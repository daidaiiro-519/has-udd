//! @spec 01-spec.md#4.1 — list_tables
//!
//! meta の `table:` prefix を走査。走査はキー昇順なので名前もソート済みで返る。

use crate::application::meta;
use crate::domain::DbError;
use crate::ports::StorageEngine;

pub fn list_tables<E: StorageEngine>(engine: &E) -> Result<Vec<String>, DbError> {
    let txn = engine.begin_read()?;
    let prefix = b"table:";
    txn.scan_prefix(meta::META_TABLE, prefix)?
        .into_iter()
        .map(|(key, _)| {
            String::from_utf8(key[prefix.len()..].to_vec())
                .map_err(|e| DbError::Storage(format!("corrupt table name in meta: {e}")))
        })
        .collect()
}
