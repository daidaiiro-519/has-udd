//! LoomDB の Python バインディング（inbound adapter・PyO3）。
//!
//! 意味論はすべて loom-bridge / loom-core 側にあり、この層は
//! 「Python 値 ↔ serde_json::Value の受け渡し」と「DbError → Python 例外」だけの薄い皮。
//!
//! - Python の int は任意精度。i64/u64 に収まる範囲は**正確に** N になる
//!   （JS の f64 制約がない）。u64 超は OverflowError（38 桁 N の書込は文字列表現の
//!   :values でなく Rust/式経由で行う）
//! - bytes ↔ B 型（ブリッジの `$binary` 表現を介して透過変換）
//! - close() でファイルロックを明示解放（better-sqlite3 / loom-node と同じ流儀）

// pyo3 0.22 の #[pymethods] が生成するトランポリンコードが useless_conversion を
// 誤発火させる既知の相性問題のため、この薄いシェル crate 全体で抑止する。
#![allow(clippy::useless_conversion)]

use loom_bridge::{error_code, Bridge};
use loom_core::domain::DbError;
use loom_redb::RedbStorage;
use pyo3::exceptions::{PyOverflowError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDict, PyFloat, PyInt, PyList, PyString, PyTuple};
use serde_json::{json, Map, Value};

fn py_err(e: DbError) -> PyErr {
    PyRuntimeError::new_err(format!("{}: {e}", error_code(&e)))
}

// ---------------------------------------------------------------------------
// Python 値 ↔ JSON
// ---------------------------------------------------------------------------

fn py_to_json(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        return Ok(Value::Null);
    }
    // bool は int のサブクラスなので先に判定する
    if let Ok(b) = obj.downcast::<PyBool>() {
        return Ok(Value::Bool(b.is_true()));
    }
    if obj.downcast::<PyInt>().is_ok() {
        if let Ok(i) = obj.extract::<i64>() {
            return Ok(Value::from(i));
        }
        if let Ok(u) = obj.extract::<u64>() {
            return Ok(Value::from(u));
        }
        return Err(PyOverflowError::new_err(
            "integers larger than 64-bit are not supported yet (store as a string instead)",
        ));
    }
    if obj.downcast::<PyFloat>().is_ok() {
        let f = obj.extract::<f64>()?;
        return serde_json::Number::from_f64(f)
            .map(Value::Number)
            .ok_or_else(|| PyValueError::new_err("NaN/inf cannot be stored"));
    }
    if let Ok(s) = obj.downcast::<PyString>() {
        return Ok(Value::String(s.to_string()));
    }
    if let Ok(b) = obj.downcast::<PyBytes>() {
        return Ok(json!({ "$binary": loom_bridge::value::to_hex(b.as_bytes()) }));
    }
    if let Ok(list) = obj.downcast::<PyList>() {
        let mut out = Vec::with_capacity(list.len());
        for e in list.iter() {
            out.push(py_to_json(&e)?);
        }
        return Ok(Value::Array(out));
    }
    if let Ok(tuple) = obj.downcast::<PyTuple>() {
        let mut out = Vec::with_capacity(tuple.len());
        for e in tuple.iter() {
            out.push(py_to_json(&e)?);
        }
        return Ok(Value::Array(out));
    }
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut out = Map::new();
        for (k, v) in dict.iter() {
            let key = k
                .downcast::<PyString>()
                .map_err(|_| PyTypeError::new_err("dict keys must be strings"))?
                .to_string();
            out.insert(key, py_to_json(&v)?);
        }
        return Ok(Value::Object(out));
    }
    Err(PyTypeError::new_err(format!(
        "unsupported value type: {}",
        obj.get_type().name()?
    )))
}

fn json_to_py(py: Python<'_>, v: &Value) -> PyResult<PyObject> {
    Ok(match v {
        Value::Null => py.None(),
        Value::Bool(b) => b.to_object(py),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_object(py)
            } else if let Some(u) = n.as_u64() {
                u.to_object(py)
            } else {
                n.as_f64().unwrap_or(f64::NAN).to_object(py)
            }
        }
        Value::String(s) => s.to_object(py),
        Value::Array(items) => {
            let list = PyList::empty_bound(py);
            for item in items {
                list.append(json_to_py(py, item)?)?;
            }
            list.to_object(py)
        }
        Value::Object(map) => {
            // ブリッジの $binary 表現は Python の bytes へ
            if map.len() == 1 {
                if let Some(Value::String(hex)) = map.get("$binary") {
                    let bytes = loom_bridge::value::from_hex(hex).map_err(py_err)?;
                    return Ok(PyBytes::new_bound(py, &bytes).to_object(py));
                }
            }
            let dict = PyDict::new_bound(py);
            for (k, val) in map {
                dict.set_item(k, json_to_py(py, val)?)?;
            }
            dict.to_object(py)
        }
    })
}

// ---------------------------------------------------------------------------
// LoomDB クラス
// ---------------------------------------------------------------------------

#[pyclass]
struct LoomDB {
    bridge: Option<Bridge<RedbStorage>>,
}

impl LoomDB {
    fn bridge(&self) -> PyResult<&Bridge<RedbStorage>> {
        self.bridge
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("StorageError: database is closed"))
    }
}

#[pymethods]
impl LoomDB {
    /// `LoomDB("data.loom")` — ファイルを開く（無ければ作成）。サーバ不要。
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let engine = RedbStorage::create(&path).map_err(py_err)?;
        Ok(Self {
            bridge: Some(Bridge::new(engine)),
        })
    }

    /// DB を閉じてファイルロックを解放する。以後の操作はエラー。
    fn close(&mut self) {
        self.bridge = None;
    }

    fn create_table(&self, def: Bound<'_, PyAny>) -> PyResult<()> {
        self.bridge()?
            .create_table(&py_to_json(&def)?)
            .map_err(py_err)
    }

    fn delete_table(&self, name: &str) -> PyResult<()> {
        self.bridge()?.delete_table(name).map_err(py_err)
    }

    fn list_tables(&self) -> PyResult<Vec<String>> {
        self.bridge()?.list_tables().map_err(py_err)
    }

    fn update_table(&self, name: &str, changes: Bound<'_, PyAny>) -> PyResult<()> {
        self.bridge()?
            .update_table(name, &py_to_json(&changes)?)
            .map_err(py_err)
    }

    #[pyo3(signature = (table, item, options=None))]
    fn put(
        &self,
        table: &str,
        item: Bound<'_, PyAny>,
        options: Option<Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let options = options.map(|o| py_to_json(&o)).transpose()?;
        self.bridge()?
            .put(table, &py_to_json(&item)?, options.as_ref())
            .map_err(py_err)
    }

    fn get(&self, py: Python<'_>, table: &str, key: Bound<'_, PyAny>) -> PyResult<PyObject> {
        match self
            .bridge()?
            .get(table, &py_to_json(&key)?)
            .map_err(py_err)?
        {
            Some(item) => json_to_py(py, &item),
            None => Ok(py.None()),
        }
    }

    #[pyo3(signature = (table, key, options=None))]
    fn delete(
        &self,
        py: Python<'_>,
        table: &str,
        key: Bound<'_, PyAny>,
        options: Option<Bound<'_, PyAny>>,
    ) -> PyResult<PyObject> {
        let options = options.map(|o| py_to_json(&o)).transpose()?;
        match self
            .bridge()?
            .delete(table, &py_to_json(&key)?, options.as_ref())
            .map_err(py_err)?
        {
            Some(item) => json_to_py(py, &item),
            None => Ok(py.None()),
        }
    }

    fn update(
        &self,
        py: Python<'_>,
        table: &str,
        key: Bound<'_, PyAny>,
        params: Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let new_item = self
            .bridge()?
            .update(table, &py_to_json(&key)?, &py_to_json(&params)?)
            .map_err(py_err)?;
        json_to_py(py, &new_item)
    }

    fn query(&self, py: Python<'_>, table: &str, params: Bound<'_, PyAny>) -> PyResult<PyObject> {
        let page = self
            .bridge()?
            .query(table, &py_to_json(&params)?)
            .map_err(py_err)?;
        json_to_py(py, &page)
    }

    #[pyo3(signature = (table, params=None))]
    fn scan(
        &self,
        py: Python<'_>,
        table: &str,
        params: Option<Bound<'_, PyAny>>,
    ) -> PyResult<PyObject> {
        let params = match params {
            Some(p) => py_to_json(&p)?,
            None => json!({}),
        };
        let page = self.bridge()?.scan(table, &params).map_err(py_err)?;
        json_to_py(py, &page)
    }

    /// LoomDB の差別化: N テーブル JOIN → `{ "rows": [...], "warnings": [...] }`
    fn join(&self, py: Python<'_>, params: Bound<'_, PyAny>) -> PyResult<PyObject> {
        let result = self.bridge()?.join(&py_to_json(&params)?).map_err(py_err)?;
        json_to_py(py, &result)
    }

    /// ops: `[{"put": {...}} | {"update": {...}} | {"delete": {...}} |
    ///        {"conditionCheck": {...}}]` を 1 txn で all-or-nothing 適用（件数無制限）。
    /// 条件不成立は TransactionCanceled（理由コード付き）で全体ロールバック。
    fn transact_write(&self, ops: Bound<'_, PyAny>) -> PyResult<()> {
        self.bridge()?
            .transact_write(&py_to_json(&ops)?)
            .map_err(py_err)
    }

    /// keys: `[{"table": .., "key": {..}}]` → 単一スナップショットで
    /// item | None のリスト（同順）。
    fn transact_get(&self, py: Python<'_>, keys: Bound<'_, PyAny>) -> PyResult<PyObject> {
        let got = self
            .bridge()?
            .transact_get(&py_to_json(&keys)?)
            .map_err(py_err)?;
        json_to_py(py, &got)
    }

    /// ローカルでは transact_get と同一意味論（UnprocessedKeys は常に空）。
    fn batch_get(&self, py: Python<'_>, keys: Bound<'_, PyAny>) -> PyResult<PyObject> {
        let got = self
            .bridge()?
            .batch_get(&py_to_json(&keys)?)
            .map_err(py_err)?;
        json_to_py(py, &got)
    }

    /// params: `{"puts": [{table, item}], "deletes": [{table, key}]}`（件数無制限）
    fn batch_write(&self, params: Bound<'_, PyAny>) -> PyResult<()> {
        self.bridge()?
            .batch_write(&py_to_json(&params)?)
            .map_err(py_err)
    }

    /// TTL 失効項目を budget 件まで物理削除し、削除数を返す。
    fn sweep_expired(&self, table: &str, budget: usize) -> PyResult<usize> {
        self.bridge()?.sweep_expired(table, budget).map_err(py_err)
    }
}

#[pymodule]
fn loomdb(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LoomDB>()?;
    Ok(())
}
