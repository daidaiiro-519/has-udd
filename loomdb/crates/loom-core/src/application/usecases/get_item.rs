//! @spec 01-spec.md#4.2 — get_item
//!
//! 主キー（pk (+ sk)）で 1 項目を取得する。ローカルは常に強整合。

use crate::domain::{key_codec, AttributeValue, DbError, Item, TableDef};
use crate::ports::StorageEngine;

pub fn get_item<E: StorageEngine>(
    engine: &E,
    def: &TableDef,
    pk: &AttributeValue,
    sk: Option<&AttributeValue>,
) -> Result<Option<Item>, DbError> {
    let key = key_codec::encode_key(pk, sk)?;

    let txn = engine.begin_read()?;
    match txn.get(&def.name, &key)? {
        Some(bytes) => {
            let item =
                rmp_serde::from_slice(&bytes).map_err(|e| DbError::Serialization(e.to_string()))?;
            Ok(Some(item))
        }
        None => Ok(None),
    }
}
