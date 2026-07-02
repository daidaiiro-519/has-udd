//! @spec 01-spec.md#4.1 / #7 — update_table（GSI の後付け追加・削除 = 差別化の一つ）
//!
//! 追加時は既存データを全走査して索引を**バックフィル**する。定義更新・バックフィル・
//! 削除はすべて 1 write txn = 完成した索引だけが見える（巨大テーブルでは長い txn に
//! なり他の書込をブロックする点は spec §7 に明記のトレードオフ）。

use crate::application::meta;
use crate::domain::index::{index_entry_key, index_table_name};
use crate::domain::{DbError, IndexDef, Item};
use crate::ports::StorageEngine;

pub fn update_table<E: StorageEngine>(
    engine: &E,
    table: &str,
    add_indexes: &[IndexDef],
    remove_indexes: &[String],
) -> Result<(), DbError> {
    let mut txn = engine.begin_write()?;
    let mut def = meta::load_def_write(&*txn, table)?;

    // 検証: 追加は未存在・削除は存在すること
    for idx in add_indexes {
        if def.indexes.iter().any(|i| i.name == idx.name)
            || add_indexes.iter().filter(|i| i.name == idx.name).count() > 1
        {
            return Err(DbError::Validation(format!(
                "index {:?} already exists on table {table:?}",
                idx.name
            )));
        }
    }
    for name in remove_indexes {
        if !def.indexes.iter().any(|i| &i.name == name) {
            return Err(DbError::Validation(format!(
                "index {name:?} does not exist on table {table:?}"
            )));
        }
    }

    // 削除: 索引エントリを全消去して定義から外す
    for name in remove_indexes {
        let idx_table = index_table_name(table, name);
        let keys: Vec<Vec<u8>> = txn
            .scan_prefix(&idx_table, b"")?
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        for k in keys {
            txn.delete(&idx_table, &k)?;
        }
        def.indexes.retain(|i| &i.name != name);
    }

    // 追加: 既存データをバックフィル
    if !add_indexes.is_empty() {
        let entries = txn.scan_prefix(table, b"")?;
        for idx in add_indexes {
            let idx_table = index_table_name(table, &idx.name);
            for (main_key, value) in &entries {
                let item: Item = rmp_serde::from_slice(value)
                    .map_err(|e| DbError::Serialization(e.to_string()))?;
                if let Some(entry_key) = index_entry_key(idx, &item, main_key)? {
                    txn.put(&idx_table, &entry_key, &[])?;
                }
            }
            def.indexes.push(idx.clone());
        }
    }

    txn.put(
        meta::META_TABLE,
        &meta::def_key(table),
        &meta::encode_def(&def)?,
    )?;
    txn.commit()
}
