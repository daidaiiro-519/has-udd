//! @spec 01-spec.md#4.1 — delete_table
//!
//! 定義・**全項目**・**全索引エントリ**を同一 txn で削除（DynamoDB の DeleteTable 準拠）。
//! 同名で作り直しても旧データ・旧索引が蘇らないことを保証する。

use crate::application::meta;
use crate::domain::index::index_table_name;
use crate::domain::DbError;
use crate::ports::StorageEngine;

pub fn delete_table<E: StorageEngine>(engine: &E, name: &str) -> Result<(), DbError> {
    let mut txn = engine.begin_write()?;
    let key = meta::def_key(name);
    // 定義は削除より先に読む（索引一覧が要る）
    let def = meta::load_def_write(&*txn, name)?;
    txn.delete(meta::META_TABLE, &key)?;
    txn.delete(meta::META_TABLE, &meta::count_key(name))?; // item_count も消す（spec §13）

    let item_keys: Vec<Vec<u8>> = txn
        .scan_prefix(name, b"")?
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    for k in item_keys {
        txn.delete(name, &k)?;
    }

    // 索引エントリも同一 txn で削除（spec §7）
    for idx in &def.indexes {
        let idx_table = index_table_name(name, &idx.name);
        let keys: Vec<Vec<u8>> = txn
            .scan_prefix(&idx_table, b"")?
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        for k in keys {
            txn.delete(&idx_table, &k)?;
        }
    }
    txn.commit()
}
