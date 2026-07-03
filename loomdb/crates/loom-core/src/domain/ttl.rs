//! TTL の失効判定（spec §8）。ステートレス純関数。
//!
//! 失効規則: テーブルに TTL 属性名が設定されており、item の当該属性が N（epoch 秒）で
//! **now 以下**なら失効。属性なし・N 以外の型は対象外（DynamoDB 準拠の寛容さ）。

use super::attribute::{AttributeValue, Item, Number};
use super::number;
use super::table::TableDef;
use std::cmp::Ordering;

pub fn is_expired(def: &TableDef, item: &Item, now_epoch: i64) -> bool {
    let Some(attr) = &def.ttl_attr else {
        return false;
    };
    let Some(AttributeValue::N(ttl)) = item.get(attr) else {
        return false;
    };
    matches!(
        number::compare(ttl, &Number(now_epoch.to_string())),
        Ok(Ordering::Less | Ordering::Equal)
    )
}
