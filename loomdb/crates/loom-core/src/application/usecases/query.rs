//! @spec 01-spec.md#4.3 — query
//!
//! pk パーティションを sk 順（順序保存キー）で走査し、sk 条件・ページングを適用。
//! **Limit は Filter 適用「前」**に効く（DynamoDB 準拠・spec §4.3）。
//! index（GSI/LSI）指定は二次索引サイクルで追加する。

use super::{apply_filter, KeyConditionInput, Page, QueryOptions};
use crate::application::meta;
use crate::domain::expr::{self, key as key_expr, ExprContext};
use crate::domain::{key_codec, DbError, Item};
use crate::ports::StorageEngine;

pub fn query<E: StorageEngine>(
    engine: &E,
    table: &str,
    key_condition: &KeyConditionInput,
    opts: &QueryOptions,
) -> Result<Page, DbError> {
    let txn = engine.begin_read()?;
    let def = meta::load_def_read(&*txn, table)?;

    let kc = expr::parse_key_condition(&key_condition.expression)?;
    let ctx = ExprContext {
        names: &key_condition.names,
        values: &key_condition.values,
    };

    // 属性名がテーブルのキースキーマと一致することを検証
    let pk_name = key_expr::attr_name(&kc.pk_name, &ctx)?;
    if pk_name != def.key.pk {
        return Err(DbError::Validation(format!(
            "key condition references {pk_name:?} but the partition key is {:?}",
            def.key.pk
        )));
    }
    let sk_cond = match &kc.sk {
        None => None,
        Some((seg, cond)) => {
            let name = key_expr::attr_name(seg, &ctx)?;
            match &def.key.sk {
                Some(sk_name) if *sk_name == name => Some(cond),
                Some(sk_name) => {
                    return Err(DbError::Validation(format!(
                        "key condition references {name:?} but the sort key is {sk_name:?}"
                    )))
                }
                None => {
                    return Err(DbError::Validation(format!(
                        "table {table:?} has no sort key but the condition references {name:?}"
                    )))
                }
            }
        }
    };

    let pk_value = key_condition
        .values
        .get(&format!(":{}", kc.pk_value))
        .ok_or_else(|| {
            DbError::Validation(format!(
                "unknown expression attribute value :{}",
                kc.pk_value
            ))
        })?;
    let prefix = key_codec::encode_value(pk_value)?;

    // パーティション全体はキー昇順で得られる（順序保存エンコード）
    let mut entries = txn.scan_prefix(&def.name, &prefix)?;
    if !opts.scan_forward {
        entries.reverse();
    }

    let mut matched: Vec<Item> = Vec::new();
    let mut last_key: Option<Vec<u8>> = None;
    let mut limit_hit = false;
    for (key, value) in entries {
        // 再開位置より後ろだけを見る（走査方向に応じて比較を反転）
        if let Some(start) = &opts.exclusive_start_key {
            let passed = if opts.scan_forward {
                key.as_slice() > start.as_slice()
            } else {
                key.as_slice() < start.as_slice()
            };
            if !passed {
                continue;
            }
        }
        // sk 条件（KeyCondition の一部＝Limit のカウント対象を決める）
        if let Some(cond) = sk_cond {
            let (_, sk) = key_codec::decode_key(&key)?;
            if !key_expr::sk_matches(cond, sk.as_ref(), &ctx)? {
                continue;
            }
        }
        let item: Item =
            rmp_serde::from_slice(&value).map_err(|e| DbError::Serialization(e.to_string()))?;
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
    let items = apply_filter(matched, opts.filter.as_ref())?; // Filter は Limit の後
    Ok(Page {
        items,
        last_evaluated_key,
    })
}
