//! @spec 01-spec.md#4.4 — batch_write
//!
//! 非トランザクションの冪等ループ（put は上書き・delete は no-op 許容）。
//! ローカルは分散の部分失敗が無いため **`UnprocessedItems` は常に空**
//! （全処理 or エラー・spec §4.4）。件数無制限（spec §11）。

use super::transact_get::KeyRef;
use super::{delete_item, put_item};
use crate::domain::{DbError, Item};
use crate::ports::StorageEngine;

pub fn batch_write<E: StorageEngine>(
    engine: &E,
    puts: &[(String, Item)],
    deletes: &[KeyRef],
) -> Result<(), DbError> {
    for (table, item) in puts {
        put_item(engine, table, item, None)?;
    }
    for key in deletes {
        delete_item(engine, &key.table, &key.pk, key.sk.as_ref(), None)?;
    }
    Ok(())
}
