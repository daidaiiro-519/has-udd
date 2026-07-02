//! @spec 01-spec.md#4.2 — update_item
//!
//! UpdateExpression を適用し、適用後の item 全体（ALL_NEW 相当）を返す。
//! 未存在キーへの更新は upsert（キー属性入りで新規作成・DynamoDB 準拠）。
//! condition 不成立は `ConditionalCheckFailed` でロールバック。

use super::{check_condition, decode_item_or_empty, ConditionInput, UpdateInput};
use crate::application::meta;
use crate::domain::expr::{self, ExprContext};
use crate::domain::{key_codec, AttributeValue, DbError, Item};
use crate::ports::StorageEngine;

pub fn update_item<E: StorageEngine>(
    engine: &E,
    table: &str,
    pk: &AttributeValue,
    sk: Option<&AttributeValue>,
    update: &UpdateInput,
    condition: Option<&ConditionInput>,
) -> Result<Item, DbError> {
    let mut txn = engine.begin_write()?;
    let def = meta::load_def_write(&*txn, table)?;
    if def.key.sk.is_some() != sk.is_some() {
        return Err(DbError::Validation(format!(
            "sort key presence does not match the schema of table {table:?}"
        )));
    }
    let key = key_codec::encode_key(pk, sk)?;

    let existing = txn.get(&def.name, &key)?;
    let current = decode_item_or_empty(existing.as_deref())?;
    if let Some(cond) = condition {
        check_condition(cond, &current)?; // 不成立 → txn drop = ロールバック
    }

    let ast = expr::parse_update(&update.expression)?;
    let ctx = ExprContext {
        names: &update.names,
        values: &update.values,
    };

    // キー属性の変更は禁止（DynamoDB 準拠）
    for root in expr::touched_roots(&ast, &ctx)? {
        if root == def.key.pk || Some(&root) == def.key.sk.as_ref() {
            return Err(DbError::Validation(format!(
                "cannot update key attribute {root:?}"
            )));
        }
    }

    // upsert: 未存在ならキー属性入りの新規 item を基底にする
    let mut base = current;
    if existing.is_none() {
        base.insert(def.key.pk.clone(), pk.clone());
        if let (Some(sk_name), Some(sk_val)) = (&def.key.sk, sk) {
            base.insert(sk_name.clone(), sk_val.clone());
        }
    }

    let new_item = expr::apply_update(&ast, &base, &ctx)?;
    let old_item = if existing.is_some() {
        Some(&base)
    } else {
        None
    };
    super::update_index_entries(&mut *txn, &def, &key, old_item, Some(&new_item))?;
    let bytes = rmp_serde::to_vec(&new_item).map_err(|e| DbError::Serialization(e.to_string()))?;
    txn.put(&def.name, &key, &bytes)?;
    txn.commit()?;
    Ok(new_item) // ALL_NEW 相当
}
