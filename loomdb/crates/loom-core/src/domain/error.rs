//! ドメインエラー（spec §9 の要点）。`thiserror` で定型化。

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("conditional check failed")]
    ConditionalCheckFailed,

    #[error("resource not found: {0}")]
    ResourceNotFound(String),

    #[error("resource in use: {0}")]
    ResourceInUse(String),

    #[error("validation error: {0}")]
    Validation(String),

    /// transact_write のいずれかの操作が失敗（spec §9）。
    /// 各操作の理由コード配列（"None" / "ConditionalCheckFailed"）を持つ。
    #[error("transaction canceled: {0:?}")]
    TransactionCanceled(Vec<String>),

    #[error("serialization error: {0}")]
    Serialization(String),

    /// アダプタ（redb 等）由来の下位エラーを写像する境界。
    #[error("storage error: {0}")]
    Storage(String),
}
