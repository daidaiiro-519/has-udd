//! ユースケース（1 操作 = 1 ファイル・1 入口関数・txn を張るのはここだけ）。

pub mod create_table;
pub mod delete_item;
pub mod delete_table;
pub mod describe_table;
pub mod get_item;
pub mod list_tables;
pub mod put_item;
pub mod query;
pub mod scan;
pub mod update_item;
pub mod update_table;

pub use create_table::create_table;
pub use delete_item::delete_item;
pub use delete_table::delete_table;
pub use describe_table::describe_table;
pub use get_item::get_item;
pub use list_tables::list_tables;
pub use put_item::put_item;
pub use query::query;
pub use scan::scan;
pub use update_item::update_item;
pub use update_table::update_table;

use crate::domain::expr::{eval, parse_condition, ExprContext};
use crate::domain::{AttributeValue, DbError, Item};
use std::collections::BTreeMap;

/// 式の入力一式（DynamoDB の *Expression / ExpressionAttributeNames /
/// ExpressionAttributeValues 相当）。Condition / Update / KeyCondition で共用。
/// serde 対応は JOIN の宣言的 `JoinQuery`（spec §10.5-B・JSON 投入）が post-join
/// filter として内包するため。
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExprInput {
    pub expression: String,
    /// キーは `#n` の完全形
    pub names: BTreeMap<String, String>,
    /// キーは `:v` の完全形
    pub values: BTreeMap<String, AttributeValue>,
}

pub type ConditionInput = ExprInput;
pub type UpdateInput = ExprInput;
pub type KeyConditionInput = ExprInput;

/// query/scan の結果ページ（spec §4.3）。
#[derive(Debug, Clone, Default)]
pub struct Page {
    pub items: Vec<Item>,
    /// limit で途中終了した場合の再開位置。次回の `exclusive_start_key` に渡す。
    pub last_evaluated_key: Option<Vec<u8>>,
}

/// query のオプション（spec §4.3）。
#[derive(Debug, Clone)]
pub struct QueryOptions {
    /// GSI/LSI 名。指定時は KeyCondition の属性名は索引のキースキーマを参照する。
    pub index: Option<String>,
    pub filter: Option<ConditionInput>,
    /// true = sk 昇順（既定）・false = 降順
    pub scan_forward: bool,
    pub limit: Option<usize>,
    pub exclusive_start_key: Option<Vec<u8>>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            index: None,
            filter: None,
            scan_forward: true,
            limit: None,
            exclusive_start_key: None,
        }
    }
}

/// scan のオプション（segment/total_segments は後続）。
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    pub filter: Option<ConditionInput>,
    pub limit: Option<usize>,
    pub exclusive_start_key: Option<Vec<u8>>,
}

/// Filter を「Limit 適用後のページ」に適用する（spec §4.3: Limit が先）。
pub(crate) fn apply_filter(
    items: Vec<Item>,
    filter: Option<&ConditionInput>,
) -> Result<Vec<Item>, DbError> {
    let Some(f) = filter else { return Ok(items) };
    let ast = parse_condition(&f.expression)?;
    let ctx = ExprContext {
        names: &f.names,
        values: &f.values,
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if eval(&ast, &item, &ctx)? {
            out.push(item);
        }
    }
    Ok(out)
}

/// condition を現行 item（未存在なら空 item）に対して評価し、
/// 不成立なら `ConditionalCheckFailed`（spec §4.2 / §9）。
pub(crate) fn check_condition(cond: &ConditionInput, current: &Item) -> Result<(), DbError> {
    let ast = parse_condition(&cond.expression)?;
    let ctx = ExprContext {
        names: &cond.names,
        values: &cond.values,
    };
    if eval(&ast, current, &ctx)? {
        Ok(())
    } else {
        Err(DbError::ConditionalCheckFailed)
    }
}

/// 格納済みバイト列から item を復元（無ければ空 item = 全属性欠落として評価される）。
pub(crate) fn decode_item_or_empty(bytes: Option<&[u8]>) -> Result<Item, DbError> {
    match bytes {
        Some(b) => rmp_serde::from_slice(b).map_err(|e| DbError::Serialization(e.to_string())),
        None => Ok(Item::new()),
    }
}

/// 主データの変化（old → new）に合わせ、全 GSI/LSI を**同一 write txn** で差分更新する
/// （coding-standard の不変: 主データと索引は必ず同一 txn）。
pub(crate) fn update_index_entries(
    txn: &mut (impl crate::ports::WriteTxn + ?Sized),
    def: &crate::domain::TableDef,
    main_key: &[u8],
    old: Option<&Item>,
    new: Option<&Item>,
) -> Result<(), DbError> {
    use crate::domain::index::{index_entry_key, index_table_name};
    for idx in &def.indexes {
        let idx_table = index_table_name(&def.name, &idx.name);
        let old_key = match old {
            Some(item) => index_entry_key(idx, item, main_key)?,
            None => None,
        };
        let new_key = match new {
            Some(item) => index_entry_key(idx, item, main_key)?,
            None => None,
        };
        if old_key == new_key {
            continue; // 索引キーが不変なら触らない（値は KEYS_ONLY で空）
        }
        if let Some(k) = old_key {
            txn.delete(&idx_table, &k)?;
        }
        if let Some(k) = new_key {
            txn.put(&idx_table, &k, &[])?;
        }
    }
    Ok(())
}
