//! LoomDB の JSON API ブリッジ — 言語バインディング（loom-node / loom-py）と
//! ワイヤ層が共有する inbound adapter の共通部。
//!
//! - JS オブジェクト / Python dict 相当の **素の JSON がそのまま item**
//!   （DynamoDB の型記法 `{"S": ...}` は書かせない）
//! - `values` / `names` は keyCondition・filter・update・condition で共有
//!   （DocumentClient 風）
//! - LastEvaluatedKey は不透明トークン（hex 文字列）として往復させる
//!
//! 各言語シェルはこの `Bridge` を呼び、値変換と例外変換だけを行う薄い皮になる。

pub mod value;

use loom_core::application::usecases::{
    self as uc, ConditionInput, ExprInput, QueryOptions, ScanOptions, UpdateInput,
};
use loom_core::domain::{AttributeValue, DbError, IndexDef, KeySchema, Projection, TableDef};
use loom_core::ports::StorageEngine;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// 言語シェルが例外に載せる機械可読なエラーコード（DynamoDB のエラー名に対応）。
pub fn error_code(e: &DbError) -> &'static str {
    match e {
        DbError::ConditionalCheckFailed => "ConditionalCheckFailed",
        DbError::TransactionCanceled(_) => "TransactionCanceled",
        DbError::ResourceNotFound(_) => "ResourceNotFound",
        DbError::ResourceInUse(_) => "ResourceInUse",
        DbError::Validation(_) => "ValidationError",
        DbError::Serialization(_) => "SerializationError",
        DbError::Storage(_) => "StorageError",
    }
}

pub struct Bridge<E: StorageEngine> {
    engine: E,
}

impl<E: StorageEngine> Bridge<E> {
    pub fn new(engine: E) -> Self {
        Self { engine }
    }

    /// 生エンジンへの脱出口（言語シェルが Rust API と併用したい場合）。
    pub fn engine(&self) -> &E {
        &self.engine
    }

    // -- テーブル管理 --------------------------------------------------------

    /// `{ name, key: { pk, sk? }, indexes?: [{ name, key }], ttlAttr? }`
    pub fn create_table(&self, def: &Value) -> Result<(), DbError> {
        uc::create_table(&self.engine, &parse_table_def(def)?)
    }

    pub fn delete_table(&self, name: &str) -> Result<(), DbError> {
        uc::delete_table(&self.engine, name)
    }

    pub fn list_tables(&self) -> Result<Vec<String>, DbError> {
        uc::list_tables(&self.engine)
    }

    /// `{ add?: [indexDef], remove?: [name] }`
    pub fn update_table(&self, name: &str, changes: &Value) -> Result<(), DbError> {
        let obj = as_object(changes, "updateTable changes")?;
        let mut add = Vec::new();
        if let Some(list) = obj.get("add") {
            for idx in as_array(list, "add")? {
                add.push(parse_index_def(idx)?);
            }
        }
        let mut remove = Vec::new();
        if let Some(list) = obj.get("remove") {
            for v in as_array(list, "remove")? {
                remove.push(as_str(v, "remove entry")?.to_string());
            }
        }
        uc::update_table(&self.engine, name, &add, &remove)
    }

    // -- 項目操作 ------------------------------------------------------------

    /// options: `{ condition?, values?, names? }`
    pub fn put(&self, table: &str, item: &Value, options: Option<&Value>) -> Result<(), DbError> {
        let item = value::json_to_item(item)?;
        let condition = parse_condition_opt(options)?;
        uc::put_item(&self.engine, table, &item, condition.as_ref())
    }

    /// key: `{ pkAttr: value, skAttr?: value }`（属性名はテーブル定義で解決）
    pub fn get(&self, table: &str, key: &Value) -> Result<Option<Value>, DbError> {
        let (pk, sk) = self.resolve_key(table, key)?;
        let got = uc::get_item(&self.engine, table, &pk, sk.as_ref())?;
        Ok(got.map(|item| value::item_to_json(&item)))
    }

    pub fn delete(
        &self,
        table: &str,
        key: &Value,
        options: Option<&Value>,
    ) -> Result<Option<Value>, DbError> {
        let (pk, sk) = self.resolve_key(table, key)?;
        let condition = parse_condition_opt(options)?;
        let old = uc::delete_item(&self.engine, table, &pk, sk.as_ref(), condition.as_ref())?;
        Ok(old.map(|item| value::item_to_json(&item)))
    }

    /// params: `{ update: "SET ...", condition?, values?, names? }` → ALL_NEW の item
    pub fn update(&self, table: &str, key: &Value, params: &Value) -> Result<Value, DbError> {
        let (pk, sk) = self.resolve_key(table, key)?;
        let obj = as_object(params, "update params")?;
        let expression = as_str(
            obj.get("update")
                .ok_or_else(|| DbError::Validation("update expression is required".into()))?,
            "update",
        )?
        .to_string();
        let (values, names) = shared_values_names(obj)?;
        let update = UpdateInput {
            expression,
            names: names.clone(),
            values: values.clone(),
        };
        let condition = obj
            .get("condition")
            .map(|c| -> Result<ConditionInput, DbError> {
                Ok(ConditionInput {
                    expression: as_str(c, "condition")?.to_string(),
                    names,
                    values,
                })
            })
            .transpose()?;
        let new_item = uc::update_item(
            &self.engine,
            table,
            &pk,
            sk.as_ref(),
            &update,
            condition.as_ref(),
        )?;
        Ok(value::item_to_json(&new_item))
    }

    // -- 問い合わせ ----------------------------------------------------------

    /// params: `{ keyCondition, filter?, values?, names?, index?, limit?,
    ///            scanForward?, startKey? }` → `{ items, lastEvaluatedKey? }`
    pub fn query(&self, table: &str, params: &Value) -> Result<Value, DbError> {
        let obj = as_object(params, "query params")?;
        let (values, names) = shared_values_names(obj)?;
        let key_condition = ExprInput {
            expression: as_str(
                obj.get("keyCondition")
                    .ok_or_else(|| DbError::Validation("keyCondition is required".into()))?,
                "keyCondition",
            )?
            .to_string(),
            names: names.clone(),
            values: values.clone(),
        };
        let opts = QueryOptions {
            index: opt_string(obj, "index")?,
            filter: parse_filter(obj, &values, &names)?,
            scan_forward: match obj.get("scanForward") {
                Some(v) => v
                    .as_bool()
                    .ok_or_else(|| DbError::Validation("scanForward must be a bool".into()))?,
                None => true,
            },
            limit: opt_usize(obj, "limit")?,
            exclusive_start_key: opt_token(obj, "startKey")?,
        };
        let page = uc::query(&self.engine, table, &key_condition, &opts)?;
        Ok(page_to_json(page))
    }

    /// params: `{ filter?, values?, names?, limit?, startKey? }`
    pub fn scan(&self, table: &str, params: &Value) -> Result<Value, DbError> {
        let obj = as_object(params, "scan params")?;
        let (values, names) = shared_values_names(obj)?;
        let opts = ScanOptions {
            filter: parse_filter(obj, &values, &names)?,
            limit: opt_usize(obj, "limit")?,
            exclusive_start_key: opt_token(obj, "startKey")?,
        };
        let page = uc::scan(&self.engine, table, &opts)?;
        Ok(page_to_json(page))
    }

    /// JOIN（spec §10.5-B の宣言形）:
    /// `{ root: {table, alias}, steps: [{table, alias, kind, on: [{left, right}], index?}],
    ///    filter?, values?, names?, select? }` → `{ rows, warnings }`
    pub fn join(&self, params: &Value) -> Result<Value, DbError> {
        let obj = as_object(params, "join params")?;
        let (values, names) = shared_values_names(obj)?;
        let root = parse_input_ref(
            obj.get("root")
                .ok_or_else(|| DbError::Validation("join root is required".into()))?,
        )?;
        let mut steps = Vec::new();
        if let Some(list) = obj.get("steps") {
            for step in as_array(list, "steps")? {
                steps.push(parse_join_step(step)?);
            }
        }
        let filter = match obj.get("filter") {
            Some(f) => Some(ConditionInput {
                expression: as_str(f, "filter")?.to_string(),
                names,
                values,
            }),
            None => None,
        };
        let select = match obj.get("select") {
            Some(list) => as_array(list, "select")?
                .iter()
                .map(|v| Ok(as_str(v, "select entry")?.to_string()))
                .collect::<Result<Vec<_>, DbError>>()?,
            None => Vec::new(),
        };
        let query = loom_query::JoinQuery {
            root,
            steps,
            filter,
            select,
        };
        let page = loom_query::execute(&self.engine, &query)?;
        let rows: Vec<Value> = page
            .rows
            .iter()
            .map(|row| {
                Value::Object(
                    row.iter()
                        .map(|(k, v)| (k.clone(), value::attr_to_json(v)))
                        .collect(),
                )
            })
            .collect();
        Ok(json!({ "rows": rows, "warnings": page.warnings }))
    }

    // -- transact / batch / sweep（§4.4・§8） ---------------------------------

    /// ops: `[{ "put": {table, item, condition?, values?, names?} }
    ///       | { "update": {table, key, update, condition?, values?, names?} }
    ///       | { "delete": {table, key, condition?, values?, names?} }
    ///       | { "conditionCheck": {table, key, condition, values?, names?} }]`
    /// — 1 txn で all-or-nothing（件数無制限）。不成立は TransactionCanceled。
    pub fn transact_write(&self, ops: &Value) -> Result<(), DbError> {
        let list = as_array(ops, "transactWrite ops")?;
        let mut parsed = Vec::with_capacity(list.len());
        for op in list {
            parsed.push(self.parse_transact_op(op)?);
        }
        uc::transact_write(&self.engine, &parsed)
    }

    /// keys: `[{ table, key }]` → 単一スナップショットで読み、item | null の
    /// 配列を**同じ順序**で返す。
    pub fn transact_get(&self, keys: &Value) -> Result<Value, DbError> {
        let refs = self.parse_key_refs(keys)?;
        let got = uc::transact_get(&self.engine, &refs)?;
        Ok(Value::Array(
            got.iter()
                .map(|o| o.as_ref().map(value::item_to_json).unwrap_or(Value::Null))
                .collect(),
        ))
    }

    /// ローカルでは transact_get と同一意味論（UnprocessedKeys は常に空・spec §4.4）。
    pub fn batch_get(&self, keys: &Value) -> Result<Value, DbError> {
        self.transact_get(keys)
    }

    /// params: `{ puts?: [{table, item}], deletes?: [{table, key}] }`
    /// — 非トランザクションの冪等ループ（件数無制限・UnprocessedItems は常に空）。
    pub fn batch_write(&self, params: &Value) -> Result<(), DbError> {
        let obj = as_object(params, "batchWrite params")?;
        let mut puts = Vec::new();
        if let Some(list) = obj.get("puts") {
            for p in as_array(list, "puts")? {
                let p = as_object(p, "puts entry")?;
                let table = as_str(require(p, "table", "puts entry")?, "table")?.to_string();
                let item = value::json_to_item(require(p, "item", "puts entry")?)?;
                puts.push((table, item));
            }
        }
        let deletes = match obj.get("deletes") {
            Some(list) => self.parse_key_refs(list)?,
            None => Vec::new(),
        };
        uc::batch_write(&self.engine, &puts, &deletes)
    }

    /// 失効項目を budget 件まで物理削除し、削除数を返す（spec §8）。
    pub fn sweep_expired(&self, table: &str, budget: usize) -> Result<usize, DbError> {
        uc::sweep_expired(&self.engine, table, budget)
    }

    // -- 内部 -----------------------------------------------------------------

    /// transact op 1 件（`{"put": {...}}` 形の 1 キーオブジェクト）を解析する。
    fn parse_transact_op(&self, v: &Value) -> Result<uc::TransactWriteOp, DbError> {
        let obj = as_object(v, "transact op")?;
        if obj.len() != 1 {
            return Err(DbError::Validation(
                "each transact op must be an object with exactly one of \
                 put / update / delete / conditionCheck"
                    .into(),
            ));
        }
        let (kind, body) = obj.iter().next().expect("len == 1 checked above");
        let body = as_object(body, kind)?;
        let table = as_str(require(body, "table", kind)?, "table")?.to_string();
        let (values, names) = shared_values_names(body)?;
        let condition = body
            .get("condition")
            .map(|c| -> Result<ConditionInput, DbError> {
                Ok(ConditionInput {
                    expression: as_str(c, "condition")?.to_string(),
                    names: names.clone(),
                    values: values.clone(),
                })
            })
            .transpose()?;
        match kind.as_str() {
            "put" => Ok(uc::TransactWriteOp::Put {
                item: value::json_to_item(require(body, "item", "put")?)?,
                table,
                condition,
            }),
            "update" => {
                let (pk, sk) = self.resolve_key(&table, require(body, "key", "update")?)?;
                let expression = as_str(require(body, "update", "update")?, "update")?.to_string();
                Ok(uc::TransactWriteOp::Update {
                    table,
                    pk,
                    sk,
                    update: UpdateInput {
                        expression,
                        names,
                        values,
                    },
                    condition,
                })
            }
            "delete" => {
                let (pk, sk) = self.resolve_key(&table, require(body, "key", "delete")?)?;
                Ok(uc::TransactWriteOp::Delete {
                    table,
                    pk,
                    sk,
                    condition,
                })
            }
            "conditionCheck" => {
                let (pk, sk) = self.resolve_key(&table, require(body, "key", "conditionCheck")?)?;
                let condition = condition.ok_or_else(|| {
                    DbError::Validation("conditionCheck requires condition".into())
                })?;
                Ok(uc::TransactWriteOp::ConditionCheck {
                    table,
                    pk,
                    sk,
                    condition,
                })
            }
            other => Err(DbError::Validation(format!(
                "unknown transact op {other:?} (expected put / update / delete / conditionCheck)"
            ))),
        }
    }

    /// `[{ table, key }]` を KeyRef の列に解決する（transact_get / batch の deletes）。
    fn parse_key_refs(&self, v: &Value) -> Result<Vec<uc::KeyRef>, DbError> {
        as_array(v, "keys")?
            .iter()
            .map(|entry| {
                let obj = as_object(entry, "key entry")?;
                let table = as_str(require(obj, "table", "key entry")?, "table")?.to_string();
                let (pk, sk) = self.resolve_key(&table, require(obj, "key", "key entry")?)?;
                Ok(uc::KeyRef { table, pk, sk })
            })
            .collect()
    }

    /// key JSON をテーブル定義に照らして (pk, sk?) に解決する。
    fn resolve_key(
        &self,
        table: &str,
        key: &Value,
    ) -> Result<(AttributeValue, Option<AttributeValue>), DbError> {
        let def = uc::describe_table(&self.engine, table)?;
        let obj = as_object(key, "key")?;
        let pk = obj.get(&def.key.pk).ok_or_else(|| {
            DbError::Validation(format!("key is missing partition key {:?}", def.key.pk))
        })?;
        let pk = value::json_to_attr(pk)?;
        let sk = match &def.key.sk {
            Some(sk_name) => Some(value::json_to_attr(obj.get(sk_name).ok_or_else(|| {
                DbError::Validation(format!("key is missing sort key {sk_name:?}"))
            })?)?),
            None => None,
        };
        Ok((pk, sk))
    }
}

// ---------------------------------------------------------------------------
// JSON 解析ヘルパ
// ---------------------------------------------------------------------------

fn as_object<'a>(v: &'a Value, what: &str) -> Result<&'a Map<String, Value>, DbError> {
    v.as_object()
        .ok_or_else(|| DbError::Validation(format!("{what} must be a JSON object")))
}

fn as_array<'a>(v: &'a Value, what: &str) -> Result<&'a Vec<Value>, DbError> {
    v.as_array()
        .ok_or_else(|| DbError::Validation(format!("{what} must be a JSON array")))
}

fn as_str<'a>(v: &'a Value, what: &str) -> Result<&'a str, DbError> {
    v.as_str()
        .ok_or_else(|| DbError::Validation(format!("{what} must be a string")))
}

fn require<'a>(obj: &'a Map<String, Value>, key: &str, what: &str) -> Result<&'a Value, DbError> {
    obj.get(key)
        .ok_or_else(|| DbError::Validation(format!("{what} requires {key}")))
}

fn opt_string(obj: &Map<String, Value>, key: &str) -> Result<Option<String>, DbError> {
    obj.get(key)
        .map(|v| Ok(as_str(v, key)?.to_string()))
        .transpose()
}

fn opt_usize(obj: &Map<String, Value>, key: &str) -> Result<Option<usize>, DbError> {
    obj.get(key)
        .map(|v| {
            v.as_u64()
                .map(|u| u as usize)
                .ok_or_else(|| DbError::Validation(format!("{key} must be a non-negative integer")))
        })
        .transpose()
}

fn opt_token(obj: &Map<String, Value>, key: &str) -> Result<Option<Vec<u8>>, DbError> {
    obj.get(key)
        .map(|v| value::from_hex(as_str(v, key)?))
        .transpose()
}

/// values / names を共有マップとして取り出す（各式が使う分だけ参照する）。
type SharedMaps = (BTreeMap<String, AttributeValue>, BTreeMap<String, String>);

fn shared_values_names(obj: &Map<String, Value>) -> Result<SharedMaps, DbError> {
    let mut values = BTreeMap::new();
    if let Some(v) = obj.get("values") {
        for (k, val) in as_object(v, "values")? {
            values.insert(k.clone(), value::json_to_attr(val)?);
        }
    }
    let mut names = BTreeMap::new();
    if let Some(v) = obj.get("names") {
        for (k, val) in as_object(v, "names")? {
            names.insert(k.clone(), as_str(val, "names entry")?.to_string());
        }
    }
    Ok((values, names))
}

fn parse_condition_opt(options: Option<&Value>) -> Result<Option<ConditionInput>, DbError> {
    let Some(options) = options else {
        return Ok(None);
    };
    let obj = as_object(options, "options")?;
    let Some(cond) = obj.get("condition") else {
        return Ok(None);
    };
    let (values, names) = shared_values_names(obj)?;
    Ok(Some(ConditionInput {
        expression: as_str(cond, "condition")?.to_string(),
        names,
        values,
    }))
}

fn parse_filter(
    obj: &Map<String, Value>,
    values: &BTreeMap<String, AttributeValue>,
    names: &BTreeMap<String, String>,
) -> Result<Option<ConditionInput>, DbError> {
    match obj.get("filter") {
        Some(f) => Ok(Some(ConditionInput {
            expression: as_str(f, "filter")?.to_string(),
            names: names.clone(),
            values: values.clone(),
        })),
        None => Ok(None),
    }
}

fn page_to_json(page: uc::Page) -> Value {
    let items: Vec<Value> = page.items.iter().map(value::item_to_json).collect();
    let mut out = Map::new();
    out.insert("items".into(), Value::Array(items));
    if let Some(lek) = page.last_evaluated_key {
        out.insert(
            "lastEvaluatedKey".into(),
            Value::String(value::to_hex(&lek)),
        );
    }
    Value::Object(out)
}

fn parse_table_def(v: &Value) -> Result<TableDef, DbError> {
    let obj = as_object(v, "table definition")?;
    let name = as_str(
        obj.get("name")
            .ok_or_else(|| DbError::Validation("table definition requires name".into()))?,
        "name",
    )?
    .to_string();
    let key = parse_key_schema(
        obj.get("key")
            .ok_or_else(|| DbError::Validation("table definition requires key".into()))?,
    )?;
    let mut indexes = Vec::new();
    if let Some(list) = obj.get("indexes") {
        for idx in as_array(list, "indexes")? {
            indexes.push(parse_index_def(idx)?);
        }
    }
    let ttl_attr = opt_string(obj, "ttlAttr")?;
    Ok(TableDef {
        name,
        key,
        indexes,
        ttl_attr,
    })
}

fn parse_index_def(v: &Value) -> Result<IndexDef, DbError> {
    let obj = as_object(v, "index definition")?;
    Ok(IndexDef {
        name: as_str(
            obj.get("name")
                .ok_or_else(|| DbError::Validation("index definition requires name".into()))?,
            "index name",
        )?
        .to_string(),
        key: parse_key_schema(
            obj.get("key")
                .ok_or_else(|| DbError::Validation("index definition requires key".into()))?,
        )?,
        projection: Projection::KeysOnly,
    })
}

fn parse_key_schema(v: &Value) -> Result<KeySchema, DbError> {
    let obj = as_object(v, "key schema")?;
    Ok(KeySchema {
        pk: as_str(
            obj.get("pk")
                .ok_or_else(|| DbError::Validation("key schema requires pk".into()))?,
            "pk",
        )?
        .to_string(),
        sk: opt_string(obj, "sk")?,
    })
}

fn parse_input_ref(v: &Value) -> Result<loom_query::InputRef, DbError> {
    let obj = as_object(v, "join input")?;
    Ok(loom_query::InputRef {
        table: as_str(
            obj.get("table")
                .ok_or_else(|| DbError::Validation("join input requires table".into()))?,
            "table",
        )?
        .to_string(),
        alias: as_str(
            obj.get("alias")
                .ok_or_else(|| DbError::Validation("join input requires alias".into()))?,
            "alias",
        )?
        .to_string(),
        index: opt_string(obj, "index")?,
    })
}

fn parse_join_step(v: &Value) -> Result<loom_query::JoinStep, DbError> {
    let obj = as_object(v, "join step")?;
    let kind = match obj.get("kind") {
        Some(k) => match as_str(k, "kind")?.to_ascii_lowercase().as_str() {
            "inner" => loom_query::JoinKind::Inner,
            "left" => loom_query::JoinKind::Left,
            other => {
                return Err(DbError::Validation(format!(
                    "join kind must be \"inner\" or \"left\", got {other:?}"
                )))
            }
        },
        None => loom_query::JoinKind::Inner,
    };
    let mut on = Vec::new();
    if let Some(list) = obj.get("on") {
        for eq in as_array(list, "on")? {
            let eq_obj = as_object(eq, "on entry")?;
            on.push(loom_query::JoinEq {
                left: as_str(
                    eq_obj
                        .get("left")
                        .ok_or_else(|| DbError::Validation("on entry requires left".into()))?,
                    "on.left",
                )?
                .to_string(),
                right: as_str(
                    eq_obj
                        .get("right")
                        .ok_or_else(|| DbError::Validation("on entry requires right".into()))?,
                    "on.right",
                )?
                .to_string(),
            });
        }
    }
    Ok(loom_query::JoinStep {
        input: parse_input_ref(v)?,
        kind,
        on,
    })
}
