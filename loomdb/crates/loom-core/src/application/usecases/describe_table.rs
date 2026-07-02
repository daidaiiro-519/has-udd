//! @spec 01-spec.md#4.1 — describe_table

use crate::application::meta;
use crate::domain::{DbError, TableDef};
use crate::ports::StorageEngine;

pub fn describe_table<E: StorageEngine>(engine: &E, name: &str) -> Result<TableDef, DbError> {
    let txn = engine.begin_read()?;
    meta::load_def_read(&*txn, name)
}
