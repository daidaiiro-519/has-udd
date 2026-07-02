//! @spec 01-spec.md#4.2 — delete_item
//!
//! 条件付き削除。削除した旧 item を返す（ALL_OLD 相当・無ければ None）。
//! 存在しないキーの無条件 delete は no-op（DynamoDB 準拠）。

use super::{check_condition, decode_item_or_empty, ConditionInput};
use crate::application::meta;
use crate::domain::{key_codec, AttributeValue, DbError, Item};
use crate::ports::StorageEngine;

pub fn delete_item<E: StorageEngine>(
    engine: &E,
    table: &str,
    pk: &AttributeValue,
    sk: Option<&AttributeValue>,
    condition: Option<&ConditionInput>,
) -> Result<Option<Item>, DbError> {
    let mut txn = engine.begin_write()?;
    let def = meta::load_def_write(&*txn, table)?;
    let key = key_codec::encode_key(pk, sk)?;

    let existing_bytes = txn.get(&def.name, &key)?;
    if let Some(cond) = condition {
        let current = decode_item_or_empty(existing_bytes.as_deref())?;
        check_condition(cond, &current)?; // 不成立 → txn drop = ロールバック
    }

    match existing_bytes {
        Some(bytes) => {
            let old: Item =
                rmp_serde::from_slice(&bytes).map_err(|e| DbError::Serialization(e.to_string()))?;
            txn.delete(&def.name, &key)?;
            txn.commit()?;
            Ok(Some(old))
        }
        None => Ok(None), // no-op（txn は drop）
    }
}
