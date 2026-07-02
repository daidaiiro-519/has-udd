//! LoomDB core — ヘキサゴナルの内側（domain + application + ports）。
//!
//! この crate は外部ライブラリ（redb / HTTP / CLI）に依存しない（coding-standard）。
//! ストレージ等の外側は `ports` の抽象を通してのみ触れる。

pub mod application;
pub mod domain;
pub mod ports;

pub use domain::{AttributeValue, DbError, Item, Key, KeySchema, Number, TableDef};
