//! `StorageEngine` の in-memory fake（BTreeMap ベース）。
//!
//! usecase の単体テストと契約スイートの基準実装に使う。単純さを最優先する:
//! - write txn = 全体クローンを変更し commit で差し替え（drop = ロールバック）
//! - read txn = begin 時点の全体クローン（= 自明にスナップショット一貫）
//!
//! 単一スレッドのテスト用途前提（実 DB の単一 writer 直列化は模さない）。

use loom_core::domain::error::DbError;
use loom_core::ports::{Clock, KvEntries, ReadTxn, StorageEngine, WriteTxn};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

/// (テーブル名, 論理キー) → 値。タプルキーなのでテーブル間の衝突が構造的に起きない。
type Map = BTreeMap<(String, Vec<u8>), Vec<u8>>;

/// 固定時計（TTL テスト用）。`set_now` で任意の epoch 秒に設定できる。既定は 0。
#[derive(Default)]
pub struct FakeClock(AtomicI64);

impl Clock for FakeClock {
    fn now_epoch(&self) -> i64 {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Default, Clone)]
pub struct InMemoryStorage {
    inner: Arc<Mutex<Map>>,
    clock: Arc<FakeClock>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }

    /// テスト用: 現在時刻（epoch 秒）を固定する。
    pub fn set_now(&self, epoch: i64) {
        self.clock.0.store(epoch, Ordering::Relaxed);
    }
}

impl StorageEngine for InMemoryStorage {
    fn begin_write(&self) -> Result<Box<dyn WriteTxn + '_>, DbError> {
        let view = self.inner.lock().expect("testkit: lock poisoned").clone();
        Ok(Box::new(MemWrite { store: self, view }))
    }

    fn begin_read(&self) -> Result<Box<dyn ReadTxn + '_>, DbError> {
        let snapshot = self.inner.lock().expect("testkit: lock poisoned").clone();
        Ok(Box::new(MemRead { snapshot }))
    }

    fn clock(&self) -> &dyn Clock {
        &*self.clock
    }
}

struct MemWrite<'a> {
    store: &'a InMemoryStorage,
    view: Map,
}

impl WriteTxn for MemWrite<'_> {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        Ok(self.view.get(&(table.to_string(), key.to_vec())).cloned())
    }

    fn put(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<(), DbError> {
        self.view
            .insert((table.to_string(), key.to_vec()), value.to_vec());
        Ok(())
    }

    fn delete(&mut self, table: &str, key: &[u8]) -> Result<(), DbError> {
        self.view.remove(&(table.to_string(), key.to_vec()));
        Ok(())
    }

    fn scan_prefix(&self, table: &str, prefix: &[u8]) -> Result<KvEntries, DbError> {
        Ok(scan(&self.view, table, prefix))
    }

    fn commit(self: Box<Self>) -> Result<(), DbError> {
        *self.store.inner.lock().expect("testkit: lock poisoned") = self.view;
        Ok(())
    }
}

struct MemRead {
    snapshot: Map,
}

impl ReadTxn for MemRead {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, DbError> {
        Ok(self
            .snapshot
            .get(&(table.to_string(), key.to_vec()))
            .cloned())
    }

    fn scan_prefix(&self, table: &str, prefix: &[u8]) -> Result<KvEntries, DbError> {
        Ok(scan(&self.snapshot, table, prefix))
    }
}

/// BTreeMap は (String, Vec<u8>) 順に整列済みなので、フィルタ結果もキー昇順になる。
fn scan(map: &Map, table: &str, prefix: &[u8]) -> KvEntries {
    map.iter()
        .filter(|((t, k), _)| t == table && k.starts_with(prefix))
        .map(|((_, k), v)| (k.clone(), v.clone()))
        .collect()
}
