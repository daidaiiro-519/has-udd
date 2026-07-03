//! @spec 03-architecture.md#3 — redb アダプタが StorageEngine 契約を満たすこと。
//! in-memory fake と同一スイート（loom-testkit）を適用＝差替可能性の実証。

use loom_core::ports::StorageEngine;
use loom_redb::RedbStorage;
use std::cell::RefCell;

/// spec §13 運用 API: 実ファイルでの storage_bytes（サイズ > 0）と compact（redb 委譲）
#[test]
fn redb_supports_stats_bytes_and_compact() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut engine = RedbStorage::create(dir.path().join("ops.redb")).expect("create");
    {
        let mut txn = engine.begin_write().expect("txn");
        for i in 0..100u32 {
            txn.put("t", &i.to_be_bytes(), &[0u8; 512]).expect("put");
        }
        txn.commit().expect("commit");
    }
    assert!(engine.storage_bytes().expect("bytes") > 0);
    // 全削除後の compact が Err を返さないこと（回収の有無 bool は redb の判断）
    {
        let mut txn = engine.begin_write().expect("txn");
        for i in 0..100u32 {
            txn.delete("t", &i.to_be_bytes()).expect("delete");
        }
        txn.commit().expect("commit");
    }
    engine.compact().expect("compact must not error");
}

#[test]
fn redb_satisfies_storage_contract() {
    // TempDir はエンジンより長生きさせる（drop するとファイルごと消えるため保持する）
    let keep_alive: RefCell<Vec<tempfile::TempDir>> = RefCell::new(Vec::new());
    loom_testkit::contract::run_all(|| {
        let dir = tempfile::tempdir().expect("tempdir");
        let engine = RedbStorage::create(dir.path().join("contract.redb")).expect("create");
        keep_alive.borrow_mut().push(dir);
        engine
    });
}
