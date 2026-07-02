//! ポート（抽象）層（architecture §3）。外側アダプタが実装する契約。

pub mod storage;

pub use storage::{Clock, KvEntries, ReadTxn, StorageEngine, WriteTxn};
