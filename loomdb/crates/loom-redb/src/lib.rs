//! `StorageEngine` の redb 実装（architecture §2 outbound adapter）。
//!
//! 本サンプルは全論理テーブルを単一 redb テーブル `loom_main` に格納し、物理キーに
//! 論理テーブル名を前置する（spec §3 の「テーブル名を key に前置」と同方針）。redb 依存は
//! この crate に閉じ込め、上位（core）へ型を漏らさない（coding-standard）。

use loom_core::domain::error::DbError;
use loom_core::ports::{KvEntries, ReadTxn, StorageEngine, WriteTxn};
use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;

/// 主データを収める単一物理テーブル。
const MAIN: TableDefinition<&[u8], &[u8]> = TableDefinition::new("loom_main");

/// 下位（redb）エラーを DbError::Storage に写像する境界ヘルパ。
fn store<T, E: std::fmt::Display>(r: Result<T, E>) -> Result<T, DbError> {
    r.map_err(|e| DbError::Storage(e.to_string()))
}

/// 論理テーブル名 + 論理キー → 物理キー（`table` 0x00 `key`）。
fn phys_key(table: &str, key: &[u8]) -> Vec<u8> {
    let mut k = Vec::with_capacity(table.len() + 1 + key.len());
    k.extend_from_slice(table.as_bytes());
    k.push(0x00);
    k.extend_from_slice(key);
    k
}

pub struct RedbStorage {
    db: Database,
}

impl RedbStorage {
    /// 新規作成（既存ファイルがあれば開く）。空でも `loom_main` を作っておく。
    pub fn create(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let db = store(Database::create(path))?;
        let w = store(db.begin_write())?;
        store(w.open_table(MAIN))?; // 作成のみ（ハンドルは即 drop）
        store(w.commit())?;
        Ok(Self { db })
    }
}

impl StorageEngine for RedbStorage {
    fn begin_write(&self) -> Result<Box<dyn WriteTxn + '_>, DbError> {
        let txn = store(self.db.begin_write())?;
        Ok(Box::new(RedbWrite { txn }))
    }

    fn begin_read(&self) -> Result<Box<dyn ReadTxn + '_>, DbError> {
        let txn = store(self.db.begin_read())?;
        Ok(Box::new(RedbRead { txn }))
    }
}

struct RedbWrite {
    txn: redb::WriteTransaction,
}

impl WriteTxn for RedbWrite {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        let t = store(self.txn.open_table(MAIN))?;
        let pk = phys_key(table, key);
        let val = store(t.get(pk.as_slice()))?.map(|g| g.value().to_vec());
        Ok(val)
    }

    fn put(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<(), DbError> {
        let mut t = store(self.txn.open_table(MAIN))?;
        let pk = phys_key(table, key);
        store(t.insert(pk.as_slice(), value))?;
        Ok(())
    }

    fn delete(&mut self, table: &str, key: &[u8]) -> Result<(), DbError> {
        let mut t = store(self.txn.open_table(MAIN))?;
        let pk = phys_key(table, key);
        store(t.remove(pk.as_slice()))?;
        Ok(())
    }

    fn commit(self: Box<Self>) -> Result<(), DbError> {
        store(self.txn.commit())
    }
}

struct RedbRead {
    txn: redb::ReadTransaction,
}

impl ReadTxn for RedbRead {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        let t = store(self.txn.open_table(MAIN))?;
        let pk = phys_key(table, key);
        let val = store(t.get(pk.as_slice()))?.map(|g| g.value().to_vec());
        Ok(val)
    }

    fn scan_prefix(&self, table: &str, prefix: &[u8]) -> Result<KvEntries, DbError> {
        let t = store(self.txn.open_table(MAIN))?;
        let start = phys_key(table, prefix);
        let strip = table.len() + 1; // 物理キーから "table\0" を剥がす長さ
        let mut out = Vec::new();
        for entry in store(t.range(start.as_slice()..))? {
            let (k, v) = store(entry)?;
            let kb = k.value();
            if !kb.starts_with(&start) {
                break; // prefix 範囲を抜けたら終了
            }
            out.push((kb[strip..].to_vec(), v.value().to_vec()));
        }
        Ok(out)
    }
}
