//! @spec 01-spec.md#4.2 — put_item
//!
//! item をテーブル名で参照して書き込む（定義は meta から取得）。condition は
//! **現行 item**（未存在なら空）に対して同一 txn 内で評価し、不成立なら
//! `ConditionalCheckFailed` でロールバックする。

use super::{check_condition, decode_item_or_empty, ConditionInput};
use crate::application::meta;
use crate::domain::{key_codec, AttributeValue, DbError, Item, TableDef};
use crate::ports::StorageEngine;

pub fn put_item<E: StorageEngine>(
    engine: &E,
    table: &str,
    item: &Item,
    condition: Option<&ConditionInput>,
) -> Result<(), DbError> {
    let mut txn = engine.begin_write()?;
    let def = meta::load_def_write(&*txn, table)?;
    let key = encode_item_key(&def, item)?;

    if let Some(cond) = condition {
        let current = decode_item_or_empty(txn.get(&def.name, &key)?.as_deref())?;
        check_condition(cond, &current)?; // 不成立 → txn drop = ロールバック
    }

    let value = rmp_serde::to_vec(item).map_err(|e| DbError::Serialization(e.to_string()))?;
    txn.put(&def.name, &key, &value)?;
    txn.commit()
}

/// item のキー属性（pk/sk）を KeySchema に従って取り出し、順序保存キーへエンコードする。
pub(crate) fn encode_item_key(def: &TableDef, item: &Item) -> Result<Vec<u8>, DbError> {
    let pk = require_attr(item, &def.key.pk)?;
    let sk = match &def.key.sk {
        Some(sk_name) => Some(require_attr(item, sk_name)?),
        None => None,
    };
    key_codec::encode_key(pk, sk)
}

fn require_attr<'a>(item: &'a Item, name: &str) -> Result<&'a AttributeValue, DbError> {
    item.get(name)
        .ok_or_else(|| DbError::Validation(format!("missing key attribute {name:?}")))
}
