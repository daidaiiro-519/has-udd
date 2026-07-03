//! JSON 値 ↔ AttributeValue の変換規則。
//!
//! 入力（JS オブジェクト / Python dict 相当）:
//! - string → S / bool → BOOL / null → NULL / array → L / object → M
//! - number: 整数（i64/u64 に収まる）→ N を正確に。浮動小数 → 最短表現の N
//! - `{"$binary": "<hex>"}` のみからなるオブジェクト → B
//! - 集合型（JSON に集合が無いための明示表現・要素は一意に正規化される）:
//!   `{"$ss": ["a", ...]}` → SS / `{"$ns": [1, "2.5", ...]}` → NS
//!   （要素は number または数値文字列 — 精度が要る値は文字列で渡す）/
//!   `{"$bs": ["<hex>", ...]}` → BS
//!
//! 出力:
//! - N → i64/u64 に収まれば JSON number。f64 で**数値として正確に**表現できるなら
//!   JSON number。どちらも無理なら **JSON string**（精度を黙って壊さない）
//! - B → `{"$binary": "<hex>"}` / 集合 → 上記の `$ss` / `$ns` / `$bs` 形

use loom_core::domain::{number, AttributeValue, DbError, Item, Number};
use serde_json::{Map, Value};

pub fn json_to_attr(v: &Value) -> Result<AttributeValue, DbError> {
    Ok(match v {
        Value::Null => AttributeValue::Null,
        Value::Bool(b) => AttributeValue::Bool(*b),
        Value::String(s) => AttributeValue::S(s.clone()),
        Value::Number(n) => AttributeValue::N(Number(n.to_string())),
        Value::Array(items) => AttributeValue::L(
            items
                .iter()
                .map(json_to_attr)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Object(map) => {
            if map.len() == 1 {
                if let Some(Value::String(hex)) = map.get("$binary") {
                    return Ok(AttributeValue::B(from_hex(hex)?));
                }
                if let Some(list) = map.get("$ss") {
                    let elems = as_set_elems(list, "$ss")?
                        .iter()
                        .map(|e| match e {
                            Value::String(s) => Ok(s.clone()),
                            other => Err(DbError::Validation(format!(
                                "$ss elements must be strings, got {other}"
                            ))),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    return AttributeValue::string_set(elems);
                }
                if let Some(list) = map.get("$ns") {
                    let elems = as_set_elems(list, "$ns")?
                        .iter()
                        .map(|e| match e {
                            // 精度が要る値は文字列で渡せる（f64 を経由しない）
                            Value::Number(n) => Ok(Number(n.to_string())),
                            Value::String(s) => Ok(Number(s.clone())),
                            other => Err(DbError::Validation(format!(
                                "$ns elements must be numbers or numeric strings, got {other}"
                            ))),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    return AttributeValue::number_set(elems);
                }
                if let Some(list) = map.get("$bs") {
                    let elems = as_set_elems(list, "$bs")?
                        .iter()
                        .map(|e| match e {
                            Value::String(hex) => from_hex(hex),
                            other => Err(DbError::Validation(format!(
                                "$bs elements must be hex strings, got {other}"
                            ))),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    return AttributeValue::binary_set(elems);
                }
            }
            let mut out = std::collections::BTreeMap::new();
            for (k, v) in map {
                out.insert(k.clone(), json_to_attr(v)?);
            }
            AttributeValue::M(out)
        }
    })
}

fn as_set_elems<'a>(v: &'a Value, what: &str) -> Result<&'a Vec<Value>, DbError> {
    v.as_array()
        .ok_or_else(|| DbError::Validation(format!("{what} must be a JSON array")))
}

pub fn attr_to_json(v: &AttributeValue) -> Value {
    match v {
        AttributeValue::Null => Value::Null,
        AttributeValue::Bool(b) => Value::Bool(*b),
        AttributeValue::S(s) => Value::String(s.clone()),
        AttributeValue::N(n) => number_to_json(n),
        AttributeValue::B(b) => {
            let mut map = Map::new();
            map.insert("$binary".into(), Value::String(to_hex(b)));
            Value::Object(map)
        }
        AttributeValue::L(items) => Value::Array(items.iter().map(attr_to_json).collect()),
        AttributeValue::M(map) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), attr_to_json(v)))
                .collect(),
        ),
        AttributeValue::Ss(xs) => {
            wrap_set("$ss", xs.iter().map(|s| Value::String(s.clone())).collect())
        }
        AttributeValue::Ns(xs) => wrap_set("$ns", xs.iter().map(number_to_json).collect()),
        AttributeValue::Bs(xs) => {
            wrap_set("$bs", xs.iter().map(|b| Value::String(to_hex(b))).collect())
        }
    }
}

fn wrap_set(key: &str, elems: Vec<Value>) -> Value {
    let mut map = Map::new();
    map.insert(key.into(), Value::Array(elems));
    Value::Object(map)
}

/// N → JSON number（正確な場合のみ）/ それ以外は文字列フォールバック。
fn number_to_json(n: &Number) -> Value {
    if let Ok(i) = n.0.parse::<i64>() {
        return Value::from(i);
    }
    if let Ok(u) = n.0.parse::<u64>() {
        return Value::from(u);
    }
    if let Ok(f) = n.0.parse::<f64>() {
        if f.is_finite() {
            if let Some(jn) = serde_json::Number::from_f64(f) {
                // 最短表現に落とした結果が元の N と数値として等しいときだけ number にする
                let round_trip = Number(jn.to_string());
                if matches!(
                    number::compare(&round_trip, n),
                    Ok(std::cmp::Ordering::Equal)
                ) {
                    return Value::Number(jn);
                }
            }
        }
    }
    Value::String(n.0.clone())
}

pub fn json_to_item(v: &Value) -> Result<Item, DbError> {
    let Value::Object(map) = v else {
        return Err(DbError::Validation(format!(
            "expected a JSON object for an item, got {v}"
        )));
    };
    let mut out = Item::new();
    for (k, val) in map {
        out.insert(k.clone(), json_to_attr(val)?);
    }
    Ok(out)
}

pub fn item_to_json(item: &Item) -> Value {
    Value::Object(
        item.iter()
            .map(|(k, v)| (k.clone(), attr_to_json(v)))
            .collect(),
    )
}

// ---------------------------------------------------------------------------
// 不透明トークン（LastEvaluatedKey）用の hex（依存ゼロ）
// ---------------------------------------------------------------------------

pub fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

pub fn from_hex(s: &str) -> Result<Vec<u8>, DbError> {
    let err = || DbError::Validation(format!("invalid opaque token {s:?}"));
    if s.len() % 2 != 0 {
        return Err(err());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| err()))
        .collect()
}
