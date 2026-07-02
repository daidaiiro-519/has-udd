//! @spec 01-spec.md#4.2 — get_item
//!
//! テーブル名と主キー（pk (+ sk)）で 1 項目を取得する。ローカルは常に強整合。

use crate::application::meta;
use crate::domain::{key_codec, AttributeValue, DbError, Item};
use crate::ports::StorageEngine;

pub fn get_item<E: StorageEngine>(
    engine: &E,
    table: &str,
    pk: &AttributeValue,
    sk: Option<&AttributeValue>,
) -> Result<Option<Item>, DbError> {
    let txn = engine.begin_read()?;
    let def = meta::load_def_read(&*txn, table)?;
    let key = key_codec::encode_key(pk, sk)?;
    match txn.get(&def.name, &key)? {
        Some(bytes) => {
            let item =
                rmp_serde::from_slice(&bytes).map_err(|e| DbError::Serialization(e.to_string()))?;
            Ok(Some(item))
        }
        None => Ok(None),
    }
}
