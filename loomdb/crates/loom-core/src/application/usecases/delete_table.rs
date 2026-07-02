//! @spec 01-spec.md#4.1 — delete_table
//!
//! 定義と**全項目**を同一 txn で削除（DynamoDB の DeleteTable 準拠）。
//! 同名で作り直しても旧データが蘇らないことを保証する。

use crate::application::meta;
use crate::domain::DbError;
use crate::ports::StorageEngine;

pub fn delete_table<E: StorageEngine>(engine: &E, name: &str) -> Result<(), DbError> {
    let mut txn = engine.begin_write()?;
    let key = meta::def_key(name);
    if txn.get(meta::META_TABLE, &key)?.is_none() {
        return Err(DbError::ResourceNotFound(name.to_string()));
    }
    txn.delete(meta::META_TABLE, &key)?;
    let item_keys: Vec<Vec<u8>> = txn
        .scan_prefix(name, b"")?
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    for k in item_keys {
        txn.delete(name, &k)?;
    }
    // TODO(spec §7): GSI/LSI 導入後は idx テーブル群も同一 txn で削除する。
    txn.commit()
}
