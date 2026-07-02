//! @spec 01-spec.md#4.2 — put_item
//!
//! item をテーブル名で参照して書き込む（定義は meta から取得）。キー属性は
//! KeySchema に従い item から取り出す。サンプルでは condition 未対応
//! （TODO: ConditionExpression, spec §5.2）。

use crate::application::meta;
use crate::domain::{key_codec, AttributeValue, DbError, Item, TableDef};
use crate::ports::StorageEngine;

pub fn put_item<E: StorageEngine>(engine: &E, table: &str, item: &Item) -> Result<(), DbError> {
    let mut txn = engine.begin_write()?;
    let def = meta::load_def_write(&*txn, table)?;
    let key = encode_item_key(&def, item)?;
    let value = rmp_serde::to_vec(item).map_err(|e| DbError::Serialization(e.to_string()))?;
    txn.put(&def.name, &key, &value)?;
    txn.commit() // commit しなければ drop でロールバック
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
