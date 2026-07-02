//! @spec 03-architecture.md#3 — in-memory fake 自身が契約を満たすこと（スイートの基準実装）。

#[test]
fn in_memory_satisfies_storage_contract() {
    loom_testkit::contract::run_all(loom_testkit::InMemoryStorage::new);
}
