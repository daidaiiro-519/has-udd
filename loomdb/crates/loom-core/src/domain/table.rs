//! テーブル定義（spec §2.1 / §7）。

use super::key::KeySchema;
use serde::{Deserialize, Serialize};

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
