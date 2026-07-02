//! テーブル定義の永続化（spec §3 `meta`）。usecase から共通利用する内部ヘルパ。

use crate::domain::{DbError, TableDef};
use crate::ports::{ReadTxn, WriteTxn};

/// メタ情報を置く予約論理テーブル。`:` は利用者のテーブル名に使えない（spec §11）ため
/// 衝突しない。
pub const META_TABLE: &str = "loom:meta";

/// テーブル定義を格納するキー（spec §3: `table:{name}`）。
pub fn def_key(name: &str) -> Vec<u8> {
    format!("table:{name}").into_bytes()
}

pub fn encode_def(def: &TableDef) -> Result<Vec<u8>, DbError> {
    rmp_serde::to_vec(def).map_err(|e| DbError::Serialization(e.to_string()))
}

pub fn decode_def(bytes: &[u8]) -> Result<TableDef, DbError> {
    rmp_serde::from_slice(bytes).map_err(|e| DbError::Serialization(e.to_string()))
}

/// read txn からテーブル定義を取得。未作成なら `ResourceNotFound`。
pub fn load_def_read(txn: &(impl ReadTxn + ?Sized), name: &str) -> Result<TableDef, DbError> {
    match txn.get(META_TABLE, &def_key(name))? {
        Some(bytes) => decode_def(&bytes),
        None => Err(DbError::ResourceNotFound(name.to_string())),
    }
}

/// write txn からテーブル定義を取得。未作成なら `ResourceNotFound`。
pub fn load_def_write(txn: &(impl WriteTxn + ?Sized), name: &str) -> Result<TableDef, DbError> {
    match txn.get(META_TABLE, &def_key(name))? {
        Some(bytes) => decode_def(&bytes),
        None => Err(DbError::ResourceNotFound(name.to_string())),
    }
}
