//! ProjectionExpression の適用（spec §5.4）。純関数: パス列 × item → 部分 item。
//!
//! 適合規則（DynamoDB 準拠）:
//! - 存在しないパス・型不一致のパスは黙って省く（結果が空 item でもエラーにしない）
//! - 入れ子（M）は構造を保つ・リスト添字は指定要素だけを**昇順に詰めて**返す
//! - パス同士の重複（完全一致・プレフィックス関係）は `ValidationError`

use super::ast::{Path, PathSeg};
use super::eval::{seg_name, ExprContext};
use crate::domain::attribute::{AttributeValue, Item};
use crate::domain::error::DbError;
use std::collections::BTreeMap;

/// 選択木のセグメント。Name < Index の順に整列する（L の添字は昇順に処理される）。
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Seg {
    Name(String),
    Index(usize),
}

/// 選択木。Leaf = そのパス以下を丸ごと取る。
enum Node {
    Leaf,
    Children(BTreeMap<Seg, Node>),
}

pub fn project(paths: &[Path], item: &Item, ctx: &ExprContext) -> Result<Item, DbError> {
    // パス列 → 選択木（重複はここで検出する）
    let mut root: BTreeMap<Seg, Node> = BTreeMap::new();
    for path in paths {
        insert(&mut root, &path.0, ctx)?;
    }

    let mut out = Item::new();
    for (seg, node) in &root {
        let Seg::Name(name) = seg else {
            return Err(DbError::Validation(
                "attribute path cannot start with a list index".into(),
            ));
        };
        if let Some(v) = item.get(name) {
            if let Some(picked) = pick(v, node)? {
                out.insert(name.clone(), picked);
            }
        }
    }
    Ok(out)
}

fn overlap() -> DbError {
    DbError::Validation("projection paths overlap or are duplicated".into())
}

fn insert(
    children: &mut BTreeMap<Seg, Node>,
    segs: &[PathSeg],
    ctx: &ExprContext,
) -> Result<(), DbError> {
    let Some((head, rest)) = segs.split_first() else {
        return Err(overlap()); // 到達しない（パーサはパスを非空で返す）
    };
    let seg = match head {
        PathSeg::Index(i) => Seg::Index(*i),
        other => Seg::Name(seg_name(other, ctx)?),
    };
    if rest.is_empty() {
        // 完全重複 or 既存のより深いパスを飲み込む浅いパス
        if children.insert(seg, Node::Leaf).is_some() {
            return Err(overlap());
        }
        return Ok(());
    }
    match children
        .entry(seg)
        .or_insert_with(|| Node::Children(BTreeMap::new()))
    {
        Node::Leaf => Err(overlap()), // 既により浅いパスが全体を取っている
        Node::Children(map) => insert(map, rest, ctx),
    }
}

/// 選択木に沿って値を切り出す。何も一致しなければ None（呼び出し側で省く）。
fn pick(v: &AttributeValue, node: &Node) -> Result<Option<AttributeValue>, DbError> {
    let Node::Children(children) = node else {
        return Ok(Some(v.clone()));
    };
    match v {
        AttributeValue::M(map) => {
            let mut out = BTreeMap::new();
            for (seg, child) in children {
                let Seg::Name(name) = seg else {
                    continue; // M への添字指定は不一致 → 省く
                };
                if let Some(inner) = map.get(name) {
                    if let Some(picked) = pick(inner, child)? {
                        out.insert(name.clone(), picked);
                    }
                }
            }
            Ok((!out.is_empty()).then_some(AttributeValue::M(out)))
        }
        AttributeValue::L(list) => {
            let mut out = Vec::new();
            for (seg, child) in children {
                let Seg::Index(i) = seg else {
                    continue; // L への名前指定は不一致 → 省く
                };
                if let Some(inner) = list.get(*i) {
                    if let Some(picked) = pick(inner, child)? {
                        out.push(picked); // Seg は昇順に並ぶ = 添字順に詰まる
                    }
                }
            }
            Ok((!out.is_empty()).then_some(AttributeValue::L(out)))
        }
        _ => Ok(None), // スカラーの下のパスは不一致 → 省く
    }
}
