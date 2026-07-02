//! ユースケース（1 操作 = 1 ファイル・1 入口関数・txn を張るのはここだけ）。

pub mod create_table;
pub mod delete_item;
pub mod delete_table;
pub mod describe_table;
pub mod get_item;
pub mod list_tables;
pub mod put_item;
pub mod update_item;

pub use create_table::create_table;
pub use delete_item::delete_item;
pub use delete_table::delete_table;
pub use describe_table::describe_table;
pub use get_item::get_item;
pub use list_tables::list_tables;
pub use put_item::put_item;
pub use update_item::update_item;

use crate::domain::expr::{eval, parse_condition, ExprContext};
use crate::domain::{AttributeValue, DbError, Item};
use std::collections::BTreeMap;

/// ConditionExpression の入力一式（DynamoDB の
/// ConditionExpression / ExpressionAttributeNames / ExpressionAttributeValues 相当）。
#[derive(Debug, Clone, Default)]
pub struct ConditionInput {
    pub expression: String,
    /// キーは `#n` の完全形
    pub names: BTreeMap<String, String>,
    /// キーは `:v` の完全形
    pub values: BTreeMap<String, AttributeValue>,
}

/// UpdateExpression の入力一式（形は ConditionInput と同じ・意味が異なるので別型）。
#[derive(Debug, Clone, Default)]
pub struct UpdateInput {
    pub expression: String,
    /// キーは `#n` の完全形
    pub names: BTreeMap<String, String>,
    /// キーは `:v` の完全形
    pub values: BTreeMap<String, AttributeValue>,
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
