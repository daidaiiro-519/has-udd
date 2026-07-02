//! ドメインエラー（spec §9 の要点）。`thiserror` で定型化。

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("conditional check failed")]
    ConditionalCheckFailed,

    #[error("resource not found: {0}")]
    ResourceNotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    /// アダプタ（redb 等）由来の下位エラーを写像する境界。
    #[error("storage error: {0}")]
    Storage(String),
}
