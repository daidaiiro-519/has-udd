//! 属性値（spec §2.2）。DynamoDB の型体系に準拠。

use crate::domain::error::DbError;
use crate::domain::number;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// N 型: 10 進・任意精度。文字列表現で保持する（spec §2.2）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Number(pub String);

/// 属性値。
///
/// 集合型（Ss/Ns/Bs）の不変条件: **整列済み・重複なし・非空**
/// （Ns は数値として一意）。必ず `string_set` / `number_set` / `binary_set`
/// で構築すること — 構造比較（derive の Eq）が集合等価に一致するのは
/// この正規化があるため。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeValue {
    S(String),
    N(Number),
    B(Vec<u8>),
    Bool(bool),
    Null,
    M(BTreeMap<String, AttributeValue>),
    L(Vec<AttributeValue>),
    Ss(Vec<String>),
    Ns(Vec<Number>),
    Bs(Vec<Vec<u8>>),
}

impl AttributeValue {
    /// SS を正規化（整列・重複除去）して構築する。空集合は不可（spec §2.2）。
    pub fn string_set(mut elems: Vec<String>) -> Result<Self, DbError> {
        elems.sort();
        elems.dedup();
        non_empty(elems.len())?;
        Ok(Self::Ss(elems))
    }

    /// NS を正規化して構築する。要素は**数値として**一意（"1.0" と "1" は同一）・
    /// canonical 表記（"1.0"→"1"）・数値順に整列。不正な数値表現は ValidationError。
    pub fn number_set(elems: Vec<Number>) -> Result<Self, DbError> {
        let mut out: Vec<Number> = Vec::with_capacity(elems.len());
        'next: for e in &elems {
            let e = number::canonicalize(e)?;
            let mut at = out.len();
            for (i, x) in out.iter().enumerate() {
                match number::compare(&e, x)? {
                    Ordering::Equal => continue 'next,
                    Ordering::Less => {
                        at = i;
                        break;
                    }
                    Ordering::Greater => {}
                }
            }
            out.insert(at, e);
        }
        non_empty(out.len())?;
        Ok(Self::Ns(out))
    }

    /// BS を正規化（整列・重複除去）して構築する。空集合は不可。
    pub fn binary_set(mut elems: Vec<Vec<u8>>) -> Result<Self, DbError> {
        elems.sort();
        elems.dedup();
        non_empty(elems.len())?;
        Ok(Self::Bs(elems))
    }
}

fn non_empty(len: usize) -> Result<(), DbError> {
    if len == 0 {
        return Err(DbError::Validation(
            "sets (SS/NS/BS) cannot be empty".into(),
        ));
    }
    Ok(())
}

/// 項目 = 属性名→属性値のマップ（スキーマレス）。
pub type Item = BTreeMap<String, AttributeValue>;
