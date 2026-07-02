//! 属性値（spec §2.2）。DynamoDB の型体系に準拠。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// N 型: 10 進・任意精度。文字列表現で保持する（spec §2.2）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Number(pub String);

/// 属性値。サンプルでは代表的な型のみ（SS/NS/BS 集合型は後続）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeValue {
    S(String),
    N(Number),
    B(Vec<u8>),
    Bool(bool),
    Null,
    M(BTreeMap<String, AttributeValue>),
    L(Vec<AttributeValue>),
}

/// 項目 = 属性名→属性値のマップ（スキーマレス）。
pub type Item = BTreeMap<String, AttributeValue>;
