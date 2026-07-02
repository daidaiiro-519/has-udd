//! キー（spec §2.3）。

use super::attribute::AttributeValue;
use serde::{Deserialize, Serialize};

/// テーブルのキースキーマ定義（どの属性を pk/sk にするか）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeySchema {
    pub pk: String,
    pub sk: Option<String>,
}

/// 実キー値（item から取り出した pk/sk の実体）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    pub pk: AttributeValue,
    pub sk: Option<AttributeValue>,
}
