//! LoomDB の JOIN 層（spec §10）。読取専用・N テーブル多段（left-deep）。
//!
//! データ構造（`JoinQuery` / `JoinStep` / …）と実行器（`execute`・多段
//! index-nested-loop）を提供する。書込パス・トランザクション意味論には影響しない。

mod exec;

pub use exec::execute;

use loom_core::application::usecases::ConditionInput;
use loom_core::domain::attribute::AttributeValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 結合種別（結合エッジごとに指定）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinKind {
    Inner,
    Left,
}

/// 等値結合の 1 条件。`"alias.attr" = "alias.attr"`（属性はトップレベル・S/N/B）。
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
    /// 明示的に使う索引名（省略時は結合キーに合う索引を自動選択／なければ scan フォールバック）。
    #[serde(default)]
    pub index: Option<String>,
}

/// 1 段の結合（left-deep tree のノード）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinStep {
    pub input: InputRef,
    pub kind: JoinKind,
    /// 複合キー結合は AND（複数エントリ）で表す。1 本目が probe キーになる。
    pub on: Vec<JoinEq>,
}

/// N テーブル結合クエリ（spec §10.2）。`steps` を増やすだけで 2,3,4… テーブルへ拡張。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinQuery {
    pub root: InputRef,
    #[serde(default)]
    pub steps: Vec<JoinStep>,
    /// 結合後フィルタ。属性パスは `alias.attr` 修飾形（spec §10.2）。
    #[serde(default)]
    pub filter: Option<ConditionInput>,
    /// 射影パス（`"alias.attr"` 形式）。空なら全属性。
    #[serde(default)]
    pub select: Vec<String>,
    /// 1 ページの最大行数（spec §10.7・filter 適用後の出力行で数える）。
    #[serde(default)]
    pub limit: Option<usize>,
    /// 前ページの `last_evaluated_key`（root キー＋展開オフセットの不透明トークン）。
    #[serde(default)]
    pub exclusive_start_key: Option<Vec<u8>>,
}

/// 結合結果の 1 行（`"alias.attr"` → 値）。LEFT 未マッチの入力の属性は欠落。
pub type JoinRow = BTreeMap<String, AttributeValue>;

/// 結合結果ページ。
#[derive(Debug, Default)]
pub struct JoinPage {
    pub rows: Vec<JoinRow>,
    /// ページング再開位置（spec §10.7）。root キー＋展開オフセットの不透明トークン。
    /// 次回の `exclusive_start_key` に渡す。
    pub last_evaluated_key: Option<Vec<u8>>,
    /// scan フォールバック等の実行時警告（spec §10.3。logging に依存しない伝達）。
    pub warnings: Vec<String>,
}
