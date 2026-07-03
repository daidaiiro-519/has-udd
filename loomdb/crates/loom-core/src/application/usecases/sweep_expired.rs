//! @spec 01-spec.md#8 — sweep_expired
//!
//! 失効した item を **budget 件まで**物理削除し、削除数を返す（1 write txn・索引も
//! 同一 txn で掃除）。常駐サービスが任意のタイミングで呼ぶ想定（バックグラウンド掃引）。

use super::update_index_entries;
use crate::application::meta;
use crate::domain::{ttl, DbError, Item};
use crate::ports::StorageEngine;

pub fn sweep_expired<E: StorageEngine>(
    engine: &E,
    table: &str,
    budget: usize,
) -> Result<usize, DbError> {
    let now = engine.clock().now_epoch();
    let mut txn = engine.begin_write()?;
    let def = meta::load_def_write(&*txn, table)?;
    if def.ttl_attr.is_none() {
        return Ok(0);
    }

    let entries = txn.scan_prefix(&def.name, b"")?;
    let mut removed = 0;
    for (key, value) in entries {
        if removed >= budget {
            break;
        }
        let item: Item =
            rmp_serde::from_slice(&value).map_err(|e| DbError::Serialization(e.to_string()))?;
        if ttl::is_expired(&def, &item, now) {
            update_index_entries(&mut *txn, &def, &key, Some(&item), None)?;
            txn.delete(&def.name, &key)?;
            removed += 1;
        }
    }
    txn.commit()?;
    Ok(removed)
}
