//! JOIN 実行器（spec §10.3）— 多段 index-nested-loop・left-deep。
//!
//! - **一貫性（§10.4）**: 結合全体を単一 read txn で実行 = 参加する全テーブルを
//!   同一 MVCC スナップショットで読む。
//! - **probe 戦略**: 結合キー（右側属性）が ①主キー → main 直引き、
//!   ②索引の pk → 索引引き（明示 `index` 指定があればそれを検証して使用）、
//!   ③どちらでもない → **scan フォールバック**（`JoinPage.warnings` で通知）。
//! - 結合キーはキーに使える型（S/N/B）。等価判定は順序保存エンコードの一致
//!   （= N は数値として等価。"1.0" と "1" は同じキー）。
//! - v1 範囲: root は全走査（key_condition プッシュダウンは後続）・ページング未対応。

use crate::{InputRef, JoinKind, JoinPage, JoinQuery, JoinRow, JoinStep};
use loom_core::application::meta;
use loom_core::domain::expr::{eval, parse_condition, ExprContext};
use loom_core::domain::index::index_table_name;
use loom_core::domain::{key_codec, ttl, AttributeValue, DbError, IndexDef, Item, TableDef};
use loom_core::ports::{ReadTxn, StorageEngine};
use std::collections::{BTreeMap, BTreeSet};

/// 中間タプル: エイリアス → その入力の item（LEFT 未マッチのエイリアスは不在）。
type Tuple = BTreeMap<String, Item>;

pub fn execute<E: StorageEngine>(engine: &E, query: &JoinQuery) -> Result<JoinPage, DbError> {
    let now = engine.clock().now_epoch(); // TTL 失効判定（spec §8: 読取時失効）
    let txn = engine.begin_read()?; // 単一スナップショット（spec §10.4）
    let mut warnings = Vec::new();

    // エイリアスの一意性検証
    let mut aliases = BTreeSet::new();
    aliases.insert(query.root.alias.clone());
    for step in &query.steps {
        if !aliases.insert(step.input.alias.clone()) {
            return Err(DbError::Validation(format!(
                "duplicate alias {:?}",
                step.input.alias
            )));
        }
    }

    // root: 全走査（v1）
    let root_def = meta::load_def_read(&*txn, &query.root.table)?;
    let mut tuples: Vec<Tuple> = Vec::new();
    for (_key, value) in txn.scan_prefix(&root_def.name, b"")? {
        let item: Item =
            rmp_serde_decode(&value).map_err(|e| DbError::Serialization(e.to_string()))?;
        if ttl::is_expired(&root_def, &item, now) {
            continue; // 失効 item は存在しない扱い
        }
        let mut t = Tuple::new();
        t.insert(query.root.alias.clone(), item);
        tuples.push(t);
    }

    // steps を宣言順に適用（left-deep）
    let mut known = BTreeSet::new();
    known.insert(query.root.alias.clone());
    for step in &query.steps {
        tuples = apply_step(&*txn, tuples, step, &known, &mut warnings, now)?;
        known.insert(step.input.alias.clone());
    }

    // 結合後フィルタ: alias 修飾パスは「alias → M(item)」の入れ子表現で §5 評価器に乗る
    if let Some(filter) = &query.filter {
        let ast = parse_condition(&filter.expression)?;
        let ctx = ExprContext {
            names: &filter.names,
            values: &filter.values,
        };
        let mut kept = Vec::with_capacity(tuples.len());
        for tuple in tuples {
            if eval(&ast, &nested_item(&tuple), &ctx)? {
                kept.push(tuple);
            }
        }
        tuples = kept;
    }

    // select 射影 → `alias.attr` の平坦な行へ
    let rows = tuples
        .iter()
        .map(|t| project(t, &query.select))
        .collect::<Result<Vec<JoinRow>, DbError>>()?;

    Ok(JoinPage {
        rows,
        last_evaluated_key: None,
        warnings,
    })
}

fn rmp_serde_decode(bytes: &[u8]) -> Result<Item, rmp_serde::decode::Error> {
    rmp_serde::from_slice(bytes)
}

// ---------------------------------------------------------------------------
// 1 段の結合
// ---------------------------------------------------------------------------

/// probe 戦略（右側テーブルの参照方法）。
enum Probe {
    /// 結合キー = 主キー（pk）→ main のパーティション直引き
    MainPk,
    /// 結合キー = 索引の pk → 索引引き＋main から item 取得
    Index(IndexDef),
    /// 索引なし → 事前に全読みした候補を線形照合（警告つき）
    Scan(Vec<Item>),
}

fn apply_step(
    txn: &(impl ReadTxn + ?Sized),
    tuples: Vec<Tuple>,
    step: &JoinStep,
    known_aliases: &BTreeSet<String>,
    warnings: &mut Vec<String>,
    now: i64,
) -> Result<Vec<Tuple>, DbError> {
    let def = meta::load_def_read(txn, &step.input.table)?;
    let ons = parse_on(step, known_aliases)?;
    let (probe_attr, probe) = choose_probe(txn, &def, &step.input, &ons[0].1, warnings)?;

    let mut out = Vec::new();
    for tuple in tuples {
        // probe キー（1 本目の on 条件の左辺）をタプルから取り出す
        let left_val = resolve_left(&tuple, &ons[0].0);
        let mut candidates: Vec<Item> = match left_val {
            None => Vec::new(), // 左値欠落 = マッチなし
            Some(v) => match &probe {
                Probe::MainPk => fetch_main_partition(txn, &def, v)?,
                Probe::Index(idx) => fetch_via_index(txn, &def, idx, v)?,
                Probe::Scan(all) => all
                    .iter()
                    .filter(|item| attr_equal(item.get(&probe_attr), v))
                    .cloned()
                    .collect(),
            },
        };
        // 失効 item は存在しない扱い（spec §8）
        candidates.retain(|cand| !ttl::is_expired(&def, cand, now));
        // 残りの on 条件（AND）で絞る
        candidates.retain(|cand| {
            ons[1..].iter().all(|((l_alias, l_attr), r_attr)| {
                match tuple.get(l_alias).and_then(|it| it.get(l_attr)) {
                    Some(lv) => attr_equal(cand.get(r_attr), lv),
                    None => false,
                }
            })
        });

        if candidates.is_empty() {
            match step.kind {
                JoinKind::Inner => {}              // 捨てる（以降の step に進めない）
                JoinKind::Left => out.push(tuple), // 残す（当該 alias は欠落）
            }
        } else {
            for cand in candidates {
                let mut expanded = tuple.clone();
                expanded.insert(step.input.alias.clone(), cand);
                out.push(expanded); // 1対多はタプル × 各マッチに展開
            }
        }
    }
    Ok(out)
}

/// on 条件を ((left_alias, left_attr), right_attr) の列に解析・検証する。
type OnCond = ((String, String), String);

fn parse_on(step: &JoinStep, known_aliases: &BTreeSet<String>) -> Result<Vec<OnCond>, DbError> {
    if step.on.is_empty() {
        return Err(DbError::Validation(format!(
            "join step {:?} requires at least one on condition",
            step.input.alias
        )));
    }
    step.on
        .iter()
        .map(|eq| {
            let (l_alias, l_attr) = eq.left.split_once('.').ok_or_else(|| {
                DbError::Validation(format!("on.left {:?} must be \"alias.attr\"", eq.left))
            })?;
            if !known_aliases.contains(l_alias) {
                return Err(DbError::Validation(format!(
                    "on.left references unknown alias {l_alias:?}"
                )));
            }
            let (r_alias, r_attr) = eq.right.split_once('.').ok_or_else(|| {
                DbError::Validation(format!("on.right {:?} must be \"alias.attr\"", eq.right))
            })?;
            if r_alias != step.input.alias {
                return Err(DbError::Validation(format!(
                    "on.right alias {r_alias:?} must be the step alias {:?}",
                    step.input.alias
                )));
            }
            Ok((
                (l_alias.to_string(), l_attr.to_string()),
                r_attr.to_string(),
            ))
        })
        .collect()
}

/// probe 戦略を選ぶ（spec §10.3: 索引を点/範囲引き・なければ scan フォールバック）。
fn choose_probe(
    txn: &(impl ReadTxn + ?Sized),
    def: &TableDef,
    input: &InputRef,
    probe_attr: &str,
    warnings: &mut Vec<String>,
) -> Result<(String, Probe), DbError> {
    if let Some(index_name) = &input.index {
        let idx = def
            .indexes
            .iter()
            .find(|i| &i.name == index_name)
            .ok_or_else(|| {
                DbError::ResourceNotFound(format!("index {index_name:?} on table {:?}", def.name))
            })?;
        if idx.key.pk != probe_attr {
            return Err(DbError::Validation(format!(
                "index {index_name:?} pk {:?} does not match join key {probe_attr:?}",
                idx.key.pk
            )));
        }
        return Ok((probe_attr.to_string(), Probe::Index(idx.clone())));
    }
    if probe_attr == def.key.pk {
        return Ok((probe_attr.to_string(), Probe::MainPk));
    }
    if let Some(idx) = def.indexes.iter().find(|i| i.key.pk == probe_attr) {
        return Ok((probe_attr.to_string(), Probe::Index(idx.clone())));
    }
    // scan フォールバック: 候補を一度だけ全読みし、警告で通知（spec §10.3）
    warnings.push(format!(
        "join on {}.{probe_attr}: no index found; falling back to a full scan of {:?}",
        input.alias, def.name
    ));
    let mut all = Vec::new();
    for (_k, v) in txn.scan_prefix(&def.name, b"")? {
        all.push(rmp_serde_decode(&v).map_err(|e| DbError::Serialization(e.to_string()))?);
    }
    Ok((probe_attr.to_string(), Probe::Scan(all)))
}

fn resolve_left<'a>(tuple: &'a Tuple, left: &(String, String)) -> Option<&'a AttributeValue> {
    tuple.get(&left.0).and_then(|item| item.get(&left.1))
}

/// 等価判定 = 順序保存エンコードの一致（N は数値として等価・S/B はバイト一致）。
/// キーに使えない型（BOOL/M/L 等）は結合キーとして不一致扱い。
fn attr_equal(a: Option<&AttributeValue>, b: &AttributeValue) -> bool {
    match (a.map(key_codec::encode_value), key_codec::encode_value(b)) {
        (Some(Ok(x)), Ok(y)) => x == y,
        _ => false,
    }
}

/// main のパーティション直引き（結合キー = pk）。
fn fetch_main_partition(
    txn: &(impl ReadTxn + ?Sized),
    def: &TableDef,
    value: &AttributeValue,
) -> Result<Vec<Item>, DbError> {
    let Ok(prefix) = key_codec::encode_value(value) else {
        return Ok(Vec::new()); // キー化できない型 = マッチなし
    };
    txn.scan_prefix(&def.name, &prefix)?
        .into_iter()
        .map(|(_k, v)| rmp_serde_decode(&v).map_err(|e| DbError::Serialization(e.to_string())))
        .collect()
}

/// 索引引き: 索引パーティションを走査し、エントリ末尾の主キーで main から item を引く。
fn fetch_via_index(
    txn: &(impl ReadTxn + ?Sized),
    def: &TableDef,
    idx: &IndexDef,
    value: &AttributeValue,
) -> Result<Vec<Item>, DbError> {
    let Ok(prefix) = key_codec::encode_value(value) else {
        return Ok(Vec::new());
    };
    let idx_table = index_table_name(&def.name, &idx.name);
    let mut out = Vec::new();
    for (entry_key, _v) in txn.scan_prefix(&idx_table, &prefix)? {
        let (_ipk, mut off) = key_codec::decode_first(&entry_key)?;
        if idx.key.sk.is_some() {
            let (_isk, used) = key_codec::decode_first(&entry_key[off..])?;
            off += used;
        }
        let main_key = &entry_key[off..];
        let bytes = txn.get(&def.name, main_key)?.ok_or_else(|| {
            DbError::Storage("index entry points to a missing main item (corruption)".into())
        })?;
        out.push(rmp_serde_decode(&bytes).map_err(|e| DbError::Serialization(e.to_string()))?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// 出力整形
// ---------------------------------------------------------------------------

/// タプルを「alias → M(item)」の入れ子 item に写す（§5 式評価器で alias.attr が引ける）。
fn nested_item(tuple: &Tuple) -> Item {
    tuple
        .iter()
        .map(|(alias, item)| (alias.clone(), AttributeValue::M(item.clone())))
        .collect()
}

/// select（`alias.attr` の列）で射影。空なら全属性を `alias.attr` に平坦化。
fn project(tuple: &Tuple, select: &[String]) -> Result<JoinRow, DbError> {
    let mut row = JoinRow::new();
    if select.is_empty() {
        for (alias, item) in tuple {
            for (attr, value) in item {
                row.insert(format!("{alias}.{attr}"), value.clone());
            }
        }
        return Ok(row);
    }
    for path in select {
        let (alias, attr) = path.split_once('.').ok_or_else(|| {
            DbError::Validation(format!("select path {path:?} must be \"alias.attr\""))
        })?;
        if let Some(value) = tuple.get(alias).and_then(|item| item.get(attr)) {
            row.insert(path.clone(), value.clone());
        }
        // 欠落（LEFT 未マッチ等）は行に含めない = attribute_exists 偽と整合
    }
    Ok(row)
}
