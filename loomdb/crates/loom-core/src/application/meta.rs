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

/// item_count（spec §13・O(1) stats 用）を格納するキー。u64 BE 固定 8 バイト。
pub fn count_key(name: &str) -> Vec<u8> {
    format!("count:{name}").into_bytes()
}

fn decode_count(bytes: Option<Vec<u8>>) -> Result<u64, DbError> {
    match bytes {
        None => Ok(0),
        Some(b) => Ok(u64::from_be_bytes(b.as_slice().try_into().map_err(
            |_| DbError::Serialization("corrupt item_count in meta".into()),
        )?)),
    }
}

/// read txn から item_count を取得（未記録は 0）。
pub fn load_count(txn: &(impl ReadTxn + ?Sized), name: &str) -> Result<u64, DbError> {
    decode_count(txn.get(META_TABLE, &count_key(name))?)
}

/// item_count に差分を適用する（write txn 内 = 主データの変更と原子的）。
pub fn adjust_count(
    txn: &mut (impl WriteTxn + ?Sized),
    name: &str,
    delta: i64,
) -> Result<(), DbError> {
    let current = decode_count(txn.get(META_TABLE, &count_key(name))?)?;
    let next = current.saturating_add_signed(delta);
    txn.put(META_TABLE, &count_key(name), &next.to_be_bytes())
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
