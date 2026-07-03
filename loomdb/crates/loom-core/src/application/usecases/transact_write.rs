//! @spec 01-spec.md#4.4 / #6 — transact_write
//!
//! Put / Update / Delete / ConditionCheck を **1 write txn で all-or-nothing** に適用する。
//! いずれかの条件が不成立なら `TransactionCanceled`（理由コード配列）で全体ロールバック。
//! 操作数は無制限（spec §11・分散由来の 25/100 件制限は撤廃）。
//! トレードオフ: 巨大な transact は commit まで単一ライタを占有する（spec §4.4）。

use super::{
    check_condition, decode_item_or_empty, put_item::encode_item_key, update_index_entries,
    ConditionInput, UpdateInput,
};
use crate::application::meta;
use crate::domain::expr::{self, ExprContext};
use crate::domain::{key_codec, AttributeValue, DbError, Item, TableDef};
use crate::ports::{StorageEngine, WriteTxn};
use std::collections::BTreeSet;

/// transact_write の 1 操作。
pub enum TransactWriteOp {
    Put {
        table: String,
        item: Item,
        condition: Option<ConditionInput>,
    },
    Update {
        table: String,
        pk: AttributeValue,
        sk: Option<AttributeValue>,
        update: UpdateInput,
        condition: Option<ConditionInput>,
    },
    Delete {
        table: String,
        pk: AttributeValue,
        sk: Option<AttributeValue>,
        condition: Option<ConditionInput>,
    },
    ConditionCheck {
        table: String,
        pk: AttributeValue,
        sk: Option<AttributeValue>,
        condition: ConditionInput,
    },
}

pub fn transact_write<E: StorageEngine>(
    engine: &E,
    ops: &[TransactWriteOp],
) -> Result<(), DbError> {
    let mut txn = engine.begin_write()?;

    // 同一 item への複数操作は拒否（DynamoDB 準拠）
    let mut seen: BTreeSet<(String, Vec<u8>)> = BTreeSet::new();
    for op in ops {
        let (table, key) = op_identity(&*txn, op)?;
        if !seen.insert((table, key)) {
            return Err(DbError::Validation(
                "transaction cannot include multiple operations on the same item".into(),
            ));
        }
    }

    for (i, op) in ops.iter().enumerate() {
        match apply_op(&mut *txn, op) {
            Ok(()) => {}
            Err(DbError::ConditionalCheckFailed) => {
                let mut reasons = vec!["None".to_string(); ops.len()];
                reasons[i] = "ConditionalCheckFailed".into();
                return Err(DbError::TransactionCanceled(reasons)); // txn drop = 全体ロールバック
            }
            Err(other) => return Err(other),
        }
    }
    txn.commit()
}

/// 操作対象の (テーブル名, エンコード済みキー) — 重複検査用。
fn op_identity(txn: &dyn WriteTxn, op: &TransactWriteOp) -> Result<(String, Vec<u8>), DbError> {
    match op {
        TransactWriteOp::Put { table, item, .. } => {
            let def = meta::load_def_write(txn, table)?;
            Ok((table.clone(), encode_item_key(&def, item)?))
        }
        TransactWriteOp::Update { table, pk, sk, .. }
        | TransactWriteOp::Delete { table, pk, sk, .. }
        | TransactWriteOp::ConditionCheck { table, pk, sk, .. } => {
            meta::load_def_write(txn, table)?; // 存在検証
            Ok((table.clone(), key_codec::encode_key(pk, sk.as_ref())?))
        }
    }
}

fn apply_op(txn: &mut dyn WriteTxn, op: &TransactWriteOp) -> Result<(), DbError> {
    match op {
        TransactWriteOp::Put {
            table,
            item,
            condition,
        } => {
            let def = meta::load_def_write(txn, table)?;
            let key = encode_item_key(&def, item)?;
            let existing = txn.get(&def.name, &key)?;
            if let Some(cond) = condition {
                check_condition(cond, &decode_item_or_empty(existing.as_deref())?)?;
            }
            let old = decode_old(existing.as_deref())?;
            update_index_entries(txn, &def, &key, old.as_ref(), Some(item))?;
            let bytes =
                rmp_serde::to_vec(item).map_err(|e| DbError::Serialization(e.to_string()))?;
            txn.put(&def.name, &key, &bytes)
        }
        TransactWriteOp::Update {
            table,
            pk,
            sk,
            update,
            condition,
        } => {
            let def = meta::load_def_write(txn, table)?;
            check_sk_shape(&def, sk, table)?;
            let key = key_codec::encode_key(pk, sk.as_ref())?;
            let existing = txn.get(&def.name, &key)?;
            let current = decode_item_or_empty(existing.as_deref())?;
            if let Some(cond) = condition {
                check_condition(cond, &current)?;
            }
            let ast = expr::parse_update(&update.expression)?;
            let ctx = ExprContext {
                names: &update.names,
                values: &update.values,
            };
            for root in expr::touched_roots(&ast, &ctx)? {
                if root == def.key.pk || Some(&root) == def.key.sk.as_ref() {
                    return Err(DbError::Validation(format!(
                        "cannot update key attribute {root:?}"
                    )));
                }
            }
            let mut base = current;
            if existing.is_none() {
                base.insert(def.key.pk.clone(), pk.clone());
                if let (Some(sk_name), Some(sk_val)) = (&def.key.sk, sk) {
                    base.insert(sk_name.clone(), sk_val.clone());
                }
            }
            let new_item = expr::apply_update(&ast, &base, &ctx)?;
            let old = if existing.is_some() {
                Some(&base)
            } else {
                None
            };
            update_index_entries(txn, &def, &key, old, Some(&new_item))?;
            let bytes =
                rmp_serde::to_vec(&new_item).map_err(|e| DbError::Serialization(e.to_string()))?;
            txn.put(&def.name, &key, &bytes)
        }
        TransactWriteOp::Delete {
            table,
            pk,
            sk,
            condition,
        } => {
            let def = meta::load_def_write(txn, table)?;
            let key = key_codec::encode_key(pk, sk.as_ref())?;
            let existing = txn.get(&def.name, &key)?;
            if let Some(cond) = condition {
                check_condition(cond, &decode_item_or_empty(existing.as_deref())?)?;
            }
            if let Some(old) = decode_old(existing.as_deref())? {
                update_index_entries(txn, &def, &key, Some(&old), None)?;
                txn.delete(&def.name, &key)?;
            }
            Ok(())
        }
        TransactWriteOp::ConditionCheck {
            table,
            pk,
            sk,
            condition,
        } => {
            let def = meta::load_def_write(txn, table)?;
            let key = key_codec::encode_key(pk, sk.as_ref())?;
            let current = decode_item_or_empty(txn.get(&def.name, &key)?.as_deref())?;
            check_condition(condition, &current)
        }
    }
}

fn decode_old(bytes: Option<&[u8]>) -> Result<Option<Item>, DbError> {
    match bytes {
        Some(b) => Ok(Some(
            rmp_serde::from_slice(b).map_err(|e| DbError::Serialization(e.to_string()))?,
        )),
        None => Ok(None),
    }
}

fn check_sk_shape(def: &TableDef, sk: &Option<AttributeValue>, table: &str) -> Result<(), DbError> {
    if def.key.sk.is_some() != sk.is_some() {
        return Err(DbError::Validation(format!(
            "sort key presence does not match the schema of table {table:?}"
        )));
    }
    Ok(())
}
