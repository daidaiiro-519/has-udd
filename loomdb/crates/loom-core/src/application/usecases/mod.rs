//! ユースケース（1 操作 = 1 ファイル・1 入口関数・txn を張るのはここだけ）。

pub mod get_item;
pub mod put_item;

pub use get_item::get_item;
pub use put_item::put_item;
