//! ストレージ抽象（architecture §3）。redb / LMDB を差し替え可能にする port。
//!
//! 注: architecture のスケッチは GAT（`type Txn<'a>`）で書かれているが、本サンプルでは
//! trait object で扱いやすいよう `Box<dyn ..>` に簡素化している。範囲スキャンも eager に
//! `Vec` 返しにしている（本実装ではストリーミングにする）。

use crate::domain::error::DbError;

/// 範囲スキャンの結果（論理キー, 値）の列。
pub type KvEntries = Vec<(Vec<u8>, Vec<u8>)>;

/// 順序付き KV の ACID ストレージ。`table` は論理テーブル名、物理配置はアダプタの責務。
pub trait StorageEngine {
    fn begin_write(&self) -> Result<Box<dyn WriteTxn + '_>, DbError>;
    fn begin_read(&self) -> Result<Box<dyn ReadTxn + '_>, DbError>;

    /// TTL 判定などに使う時刻源（Clock port）。既定は実時刻。
    /// テスト用エンジン（testkit）は固定時計に差し替える。
    fn clock(&self) -> &dyn Clock {
        &SYSTEM_CLOCK
    }

    /// 空き領域の回収（spec §13）。回収を実行したら true。
    /// 対応しないエンジン（in-memory fake 等）は false を返す。
    fn compact(&mut self) -> Result<bool, DbError> {
        Ok(false)
    }

    /// 物理ストレージのサイズ（bytes・spec §13 の stats 用）。
    fn storage_bytes(&self) -> Result<u64, DbError>;
}

/// 実時刻の Clock（既定実装・tech-stack §6: std::time を port 越しに使う）。
pub struct SystemClock;

static SYSTEM_CLOCK: SystemClock = SystemClock;

impl Clock for SystemClock {
    fn now_epoch(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}

/// 書込トランザクション。`commit` しなければ drop = ロールバック（architecture §3）。
pub trait WriteTxn {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, DbError>;
    fn put(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<(), DbError>;
    fn delete(&mut self, table: &str, key: &[u8]) -> Result<(), DbError>;
    /// 未 commit の自分の書込を含む prefix 走査（delete_table・索引バックフィルの基盤）。
    fn scan_prefix(&self, table: &str, prefix: &[u8]) -> Result<KvEntries, DbError>;
    fn commit(self: Box<Self>) -> Result<(), DbError>;
}

/// 読取トランザクション（MVCC スナップショット）。
pub trait ReadTxn {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, DbError>;
    /// 論理キーが `prefix` で始まる項目を昇順で返す（Query/Scan/JOIN の基盤）。
    fn scan_prefix(&self, table: &str, prefix: &[u8]) -> Result<KvEntries, DbError>;
}

/// 時刻源（TTL 判定など）。テストで固定時刻に差し替える（tech-stack §6）。
pub trait Clock {
    fn now_epoch(&self) -> i64;
}
