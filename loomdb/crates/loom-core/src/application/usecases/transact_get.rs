//! @spec 01-spec.md#4.4 — transact_get / batch_get
//!
//! 単一 read txn（MVCC スナップショット）で複数キーを読む。ローカルでは
//! batch_get も同じ意味論になる（DynamoDB の「batch は非トランザクション」の上位互換）。
//! `UnprocessedKeys` は常に空（spec §4.4）。

use crate::application::meta;
use crate::domain::{key_codec, ttl, AttributeValue, DbError, Item};
use crate::ports::StorageEngine;

/// 読取・削除対象のキー参照。
#[derive(Debug, Clone)]
pub struct KeyRef {
    pub table: String,
    pub pk: AttributeValue,
    pub sk: Option<AttributeValue>,
}

pub fn transact_get<E: StorageEngine>(
    engine: &E,
    keys: &[KeyRef],
) -> Result<Vec<Option<Item>>, DbError> {
    let now = engine.clock().now_epoch();
    let txn = engine.begin_read()?;
    keys.iter()
        .map(|k| {
            let def = meta::load_def_read(&*txn, &k.table)?;
            let key = key_codec::encode_key(&k.pk, k.sk.as_ref())?;
            match txn.get(&def.name, &key)? {
                Some(bytes) => {
                    let item: Item = rmp_serde::from_slice(&bytes)
                        .map_err(|e| DbError::Serialization(e.to_string()))?;
                    if ttl::is_expired(&def, &item, now) {
                        return Ok(None); // 読取時失効（spec §8）
                    }
                    Ok(Some(item))
                }
                None => Ok(None),
            }
        })
        .collect()
}

/// ローカルでは transact_get と同一（同一スナップショット読取）。
pub fn batch_get<E: StorageEngine>(
    engine: &E,
    keys: &[KeyRef],
) -> Result<Vec<Option<Item>>, DbError> {
    transact_get(engine, keys)
}
