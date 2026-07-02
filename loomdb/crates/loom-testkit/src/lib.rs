//! LoomDB テスト支援（配布物ではない・publish しない）。
//!
//! - [`InMemoryStorage`]: `StorageEngine` の in-memory fake（usecase 単体テスト用）
//! - [`contract`]: port の契約テストスイート（全アダプタに同一適用・test-standard §契約）

pub mod contract;
pub mod in_memory;

pub use in_memory::InMemoryStorage;
