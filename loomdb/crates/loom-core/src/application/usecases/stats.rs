//! @spec 01-spec.md#13 — stats（DescribeTable の ItemCount 相当・O(1)）。
//!
//! item_count は書込パス（put/update/delete/transact/sweep）が meta のカウンタを
//! 主データと同一 txn で維持するため走査なしで返せる。storage_bytes は
//! エンジンの物理サイズ（redb はファイルサイズ・fake は論理サイズ概算）。

use crate::application::meta;
use crate::domain::DbError;
use crate::ports::StorageEngine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableStats {
    pub item_count: u64,
    pub storage_bytes: u64,
}

pub fn stats<E: StorageEngine>(engine: &E, table: &str) -> Result<TableStats, DbError> {
    let txn = engine.begin_read()?;
    meta::load_def_read(&*txn, table)?; // 存在検証（未作成は ResourceNotFound）
    Ok(TableStats {
        item_count: meta::load_count(&*txn, table)?,
        storage_bytes: engine.storage_bytes()?,
    })
}
