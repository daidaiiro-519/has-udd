//! @spec 03-architecture.md#3 — redb アダプタが StorageEngine 契約を満たすこと。
//! in-memory fake と同一スイート（loom-testkit）を適用＝差替可能性の実証。

use loom_redb::RedbStorage;
use std::cell::RefCell;

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
