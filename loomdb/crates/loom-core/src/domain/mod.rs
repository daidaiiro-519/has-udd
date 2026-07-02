//! ドメイン層（最内・依存なし）。データモデル・キーエンコード・エラー。

pub mod attribute;
pub mod error;
pub mod key;
pub mod key_codec;
pub mod table;

pub use attribute::{AttributeValue, Item, Number};
pub use error::DbError;
pub use key::{Key, KeySchema};
pub use table::{IndexDef, Projection, TableDef};
