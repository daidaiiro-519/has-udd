//! LoomDB の JOIN 層（spec §10）。読取専用・N テーブル多段（left-deep）。
//!
//! この crate は **データ構造**（`JoinQuery` / `JoinStep` / …）を提供する。実行器
//! （index-nested-loop）は骨子のみで、アルゴリズムは spec §10.3 に対応させて実装する。

use loom_core::domain::attribute::AttributeValue;
use loom_core::domain::error::DbError;
use loom_core::ports::StorageEngine;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 結合種別（結合エッジごとに指定）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinKind {
    Inner,
    Left,
}

/// 等値結合の 1 条件。`"alias.attr" = "alias.attr"`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinEq {
    pub left: String,
    pub right: String,
}

/// 結合に参加する 1 入力（テーブル＋エイリアス）。エイリアスで自己結合・同名衝突を回避。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRef {
    pub table: String,
    pub alias: String,
    /// 明示的に使う索引名（省略時はアダプタが選択／scan フォールバック）。
    #[serde(default)]
    pub index: Option<String>,
    // NOTE: key_condition / filter（式言語 spec §5）はサンプルでは省略。
}

/// 1 段の結合（left-deep tree のノード）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinStep {
    pub input: InputRef,
    pub kind: JoinKind,
    /// 複合キー結合は AND（複数エントリ）で表す。
    pub on: Vec<JoinEq>,
}

/// N テーブル結合クエリ（spec §10.2）。`steps` を増やすだけで 2,3,4… テーブルへ拡張。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinQuery {
    pub root: InputRef,
    #[serde(default)]
    pub steps: Vec<JoinStep>,
    /// 射影パス（`"alias.attr"` 形式）。空なら全属性。
    #[serde(default)]
    pub select: Vec<String>,
}

/// 結合結果の 1 行（`"alias.attr"` → 値）。
pub type JoinRow = BTreeMap<String, AttributeValue>;

/// 結合結果ページ（root 走査位置でストリーミング・ページング）。
#[derive(Debug, Default)]
pub struct JoinPage {
    pub rows: Vec<JoinRow>,
    pub last_evaluated_key: Option<Vec<u8>>,
}

/// 多段 index-nested-loop の実行（spec §10.3）。
///
/// TODO(spec §10.3): root を `scan_prefix` で走査 →各 step を索引/scan で probe →
/// INNER は 0 件で打切り・LEFT は残す →post-join filter →select で射影。単一 read txn で。
pub fn execute<E: StorageEngine>(_engine: &E, _query: &JoinQuery) -> Result<JoinPage, DbError> {
    Err(DbError::Validation(
        "join executor not yet implemented (sample scaffold; see docs/01-spec.md §10.3)".into(),
    ))
}
