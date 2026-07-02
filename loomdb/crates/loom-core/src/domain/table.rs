//! テーブル定義（spec §2.1 / §7）。

use super::error::DbError;
use super::key::KeySchema;
use serde::{Deserialize, Serialize};

/// テーブル名の制約（spec §11: 3〜255 文字・`[a-zA-Z0-9_.-]`）。
/// `:` などの記号を弾くことで内部予約名（`loom:meta`）とも衝突しない。
pub fn validate_table_name(name: &str) -> Result<(), DbError> {
    let ok_len = (3..=255).contains(&name.len());
    let ok_chars = name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'));
    if ok_len && ok_chars {
        Ok(())
    } else {
        Err(DbError::Validation(format!(
            "invalid table name {name:?} (3-255 chars of [a-zA-Z0-9_.-])"
        )))
    }
}

/// 二次索引の射影種別（spec §3）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Projection {
    KeysOnly,
    Include(Vec<String>),
    All,
}

/// 二次索引（GSI/LSI）定義（spec §7）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexDef {
    pub name: String,
    pub key: KeySchema,
    pub projection: Projection,
}

/// テーブル定義。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableDef {
    pub name: String,
    pub key: KeySchema,
    #[serde(default)]
    pub indexes: Vec<IndexDef>,
    #[serde(default)]
    pub ttl_attr: Option<String>,
}
