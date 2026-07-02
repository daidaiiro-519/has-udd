//! ユースケース（1 操作 = 1 ファイル・1 入口関数・txn を張るのはここだけ）。

pub mod create_table;
pub mod delete_table;
pub mod describe_table;
pub mod get_item;
pub mod list_tables;
pub mod put_item;

pub use create_table::create_table;
pub use delete_table::delete_table;
pub use describe_table::describe_table;
pub use get_item::get_item;
pub use list_tables::list_tables;
pub use put_item::put_item;
