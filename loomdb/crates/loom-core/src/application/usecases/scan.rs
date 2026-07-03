//! @spec 01-spec.md#4.3 — scan
//!
//! テーブル全域をキー昇順で走査。Limit は Filter 適用「前」に効く（query と同じ）。
//! segment / total_segments（並列スキャン）は後続。

use super::{apply_filter, Page, ScanOptions};
use crate::application::meta;
use crate::domain::{ttl, DbError, Item};
use crate::ports::StorageEngine;

pub fn scan<E: StorageEngine>(
    engine: &E,
    table: &str,
    opts: &ScanOptions,
) -> Result<Page, DbError> {
    let now = engine.clock().now_epoch();
    let txn = engine.begin_read()?;
    let def = meta::load_def_read(&*txn, table)?;

    let mut matched: Vec<Item> = Vec::new();
    let mut last_key: Option<Vec<u8>> = None;
    let mut limit_hit = false;
    for (key, value) in txn.scan_prefix(&def.name, b"")? {
        if let Some(start) = &opts.exclusive_start_key {
            if key.as_slice() <= start.as_slice() {
                continue;
            }
        }
        let item: Item =
            rmp_serde::from_slice(&value).map_err(|e| DbError::Serialization(e.to_string()))?;
        if ttl::is_expired(&def, &item, now) {
            continue; // 読取時失効（spec §8）
        }
        matched.push(item);
        last_key = Some(key);
        if let Some(limit) = opts.limit {
            if matched.len() >= limit {
                limit_hit = true;
                break;
            }
        }
    }

    let last_evaluated_key = if limit_hit { last_key } else { None };
    let items = apply_filter(matched, opts.filter.as_ref())?;
    Ok(Page {
        items,
        last_evaluated_key,
    })
}
