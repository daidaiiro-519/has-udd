//! @spec 01-spec.md#4.1 — create_table
//!
//! キースキーマ・索引・TTL 属性を meta に登録する。同名が既にあれば `ResourceInUse`。

use crate::application::meta;
use crate::domain::table::validate_table_name;
use crate::domain::{DbError, TableDef};
use crate::ports::StorageEngine;

pub fn create_table<E: StorageEngine>(engine: &E, def: &TableDef) -> Result<(), DbError> {
    validate_table_name(&def.name)?;
    let mut txn = engine.begin_write()?;
    let key = meta::def_key(&def.name);
    if txn.get(meta::META_TABLE, &key)?.is_some() {
        return Err(DbError::ResourceInUse(def.name.clone())); // txn は drop = ロールバック
    }
    txn.put(meta::META_TABLE, &key, &meta::encode_def(def)?)?;
    txn.commit()
}
