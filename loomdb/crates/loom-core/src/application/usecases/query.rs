//! @spec 01-spec.md#4.3 — query
//!
//! 主テーブルまたは GSI/LSI を sk 順（順序保存キー）で走査し、sk 条件・ページングを
//! 適用する。**Limit は Filter 適用「前」**に効く（DynamoDB 準拠・spec §4.3）。
//! 索引経由でも item は main から全属性を返す（ローカル特権・常に強整合・spec §7）。

use super::{apply_filter, KeyConditionInput, Page, QueryOptions};
use crate::application::meta;
use crate::domain::expr::{self, key as key_expr, ExprContext, SkCond};
use crate::domain::index::index_table_name;
use crate::domain::{key_codec, DbError, Item, KeySchema};
use crate::ports::ReadTxn;
use crate::ports::StorageEngine;

pub fn query<E: StorageEngine>(
    engine: &E,
    table: &str,
    key_condition: &KeyConditionInput,
    opts: &QueryOptions,
) -> Result<Page, DbError> {
    let txn = engine.begin_read()?;
    let def = meta::load_def_read(&*txn, table)?;

    // 走査対象（主テーブル or 索引）のキースキーマと物理配置を決める
    let (schema, storage_table, via_index): (&KeySchema, String, bool) = match &opts.index {
        None => (&def.key, def.name.clone(), false),
        Some(index_name) => {
            let idx = def
                .indexes
                .iter()
                .find(|i| &i.name == index_name)
                .ok_or_else(|| {
                    DbError::ResourceNotFound(format!("index {index_name:?} on table {table:?}"))
                })?;
            (&idx.key, index_table_name(&def.name, &idx.name), true)
        }
    };

    let kc = expr::parse_key_condition(&key_condition.expression)?;
    let ctx = ExprContext {
        names: &key_condition.names,
        values: &key_condition.values,
    };

    // 属性名がキースキーマと一致することを検証
    let pk_name = key_expr::attr_name(&kc.pk_name, &ctx)?;
    if pk_name != schema.pk {
        return Err(DbError::Validation(format!(
            "key condition references {pk_name:?} but the partition key is {:?}",
            schema.pk
        )));
    }
    let sk_cond: Option<&SkCond> = match &kc.sk {
        None => None,
        Some((seg, cond)) => {
            let name = key_expr::attr_name(seg, &ctx)?;
            match &schema.sk {
                Some(sk_name) if *sk_name == name => Some(cond),
                Some(sk_name) => {
                    return Err(DbError::Validation(format!(
                        "key condition references {name:?} but the sort key is {sk_name:?}"
                    )))
                }
                None => {
                    return Err(DbError::Validation(format!(
                        "no sort key defined but the condition references {name:?}"
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

    // パーティション（または索引パーティション）はキー昇順で得られる
    let mut entries = txn.scan_prefix(&storage_table, &prefix)?;
    if !opts.scan_forward {
        entries.reverse();
    }

    let has_isk = schema.sk.is_some();
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
        // sk 条件（Limit のカウント対象を決める）
        if let Some(cond) = sk_cond {
            let sk_value = if via_index {
                decode_isk(&key, has_isk)?
            } else {
                key_codec::decode_key(&key)?.1
            };
            if !key_expr::sk_matches(cond, sk_value.as_ref(), &ctx)? {
                continue;
            }
        }
        let item = if via_index {
            fetch_via_index(&*txn, &def.name, &key, has_isk)?
        } else {
            rmp_serde::from_slice(&value).map_err(|e| DbError::Serialization(e.to_string()))?
        };
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

/// 索引エントリキー = enc(ipk) ++ enc(isk?) ++ 主キー。isk 部分を取り出す。
fn decode_isk(
    entry_key: &[u8],
    has_isk: bool,
) -> Result<Option<crate::domain::AttributeValue>, DbError> {
    let (_ipk, used) = key_codec::decode_first(entry_key)?;
    if !has_isk {
        return Ok(None);
    }
    let (isk, _) = key_codec::decode_first(&entry_key[used..])?;
    Ok(Some(isk))
}

/// 索引エントリキーの末尾（主キーのバイト列）で main から全属性の item を引く。
fn fetch_via_index(
    txn: &(impl ReadTxn + ?Sized),
    main_table: &str,
    entry_key: &[u8],
    has_isk: bool,
) -> Result<Item, DbError> {
    let (_ipk, mut off) = key_codec::decode_first(entry_key)?;
    if has_isk {
        let (_isk, used) = key_codec::decode_first(&entry_key[off..])?;
        off += used;
    }
    let main_key = &entry_key[off..];
    let bytes = txn.get(main_table, main_key)?.ok_or_else(|| {
        DbError::Storage("index entry points to a missing main item (corruption)".into())
    })?;
    rmp_serde::from_slice(&bytes).map_err(|e| DbError::Serialization(e.to_string()))
}
