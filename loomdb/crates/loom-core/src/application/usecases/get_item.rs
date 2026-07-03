//! @spec 01-spec.md#4.2 — get_item
//!
//! テーブル名と主キー（pk (+ sk)）で 1 項目を取得する。ローカルは常に強整合。
//! projection（§5.4）指定時は該当パスだけを返す。

use super::{apply_projection, ProjectionInput};
use crate::application::meta;
use crate::domain::{key_codec, ttl, AttributeValue, DbError, Item};
use crate::ports::StorageEngine;

pub fn get_item<E: StorageEngine>(
    engine: &E,
    table: &str,
    pk: &AttributeValue,
    sk: Option<&AttributeValue>,
    projection: Option<&ProjectionInput>,
) -> Result<Option<Item>, DbError> {
    let now = engine.clock().now_epoch();
    let txn = engine.begin_read()?;
    let def = meta::load_def_read(&*txn, table)?;
    let key = key_codec::encode_key(pk, sk)?;
    match txn.get(&def.name, &key)? {
        Some(bytes) => {
            let item: Item =
                rmp_serde::from_slice(&bytes).map_err(|e| DbError::Serialization(e.to_string()))?;
            if ttl::is_expired(&def, &item, now) {
                return Ok(None); // 読取時失効（論理削除・spec §8）
            }
            Ok(apply_projection(vec![item], projection)?.pop())
        }
        None => Ok(None),
    }
}
