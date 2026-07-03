//! LoomDB の Node.js バインディング（inbound adapter・napi-rs）。
//!
//! 意味論はすべて loom-bridge / loom-core 側にあり、この層は
//! 「JS 値 ↔ serde_json::Value の受け渡し」と「DbError → JS 例外」だけを行う薄い皮。
//!
//! - v1 は同期 API（better-sqlite3 と同じ方針。ローカルファイル DB は同期の方が速く単純）
//! - 現在のバックエンドは redb（ローカルファイル）固定。`Bridge<E>` は
//!   ジェネリックなので、将来の「バックエンド切替（本物の DynamoDB へ）」は
//!   ここに別エンジンを注入する形で足せる（CLAUDE.md の将来アイデア）
//! - JS の number は f64。f64 で正確に表せない N は文字列で返る（loom-bridge の規則）

#[macro_use]
extern crate napi_derive;

use loom_bridge::{error_code, Bridge};
use loom_core::domain::DbError;
use loom_redb::RedbStorage;
use serde_json::Value;

fn js_err(e: DbError) -> napi::Error {
    napi::Error::new(
        napi::Status::GenericFailure,
        format!("{}: {e}", error_code(&e)),
    )
}

#[napi(js_name = "LoomDB")]
pub struct LoomDb {
    /// close() 後は None（redb はファイルロックを持つため、GC 任せにせず
    /// 明示的に解放できるようにする — better-sqlite3 と同じ流儀）。
    bridge: Option<Bridge<RedbStorage>>,
}

#[napi]
impl LoomDb {
    /// `new LoomDB("data.loom")` — ファイルを開く（無ければ作成）。サーバ不要。
    #[napi(constructor)]
    pub fn new(path: String) -> napi::Result<Self> {
        let engine = RedbStorage::create(&path).map_err(js_err)?;
        Ok(Self {
            bridge: Some(Bridge::new(engine)),
        })
    }

    /// DB を閉じてファイルロックを解放する。以後の操作はエラー。
    #[napi]
    pub fn close(&mut self) {
        self.bridge = None;
    }

    fn bridge(&self) -> napi::Result<&Bridge<RedbStorage>> {
        self.bridge.as_ref().ok_or_else(|| {
            napi::Error::new(
                napi::Status::GenericFailure,
                "StorageError: database is closed",
            )
        })
    }

    /// `{ name, key: { pk, sk? }, indexes?, ttlAttr? }`
    #[napi]
    pub fn create_table(&self, def: Value) -> napi::Result<()> {
        self.bridge()?.create_table(&def).map_err(js_err)
    }

    #[napi]
    pub fn delete_table(&self, name: String) -> napi::Result<()> {
        self.bridge()?.delete_table(&name).map_err(js_err)
    }

    #[napi]
    pub fn list_tables(&self) -> napi::Result<Vec<String>> {
        self.bridge()?.list_tables().map_err(js_err)
    }

    /// `{ add?: [indexDef], remove?: [name] }` — GSI の後付け追加（バックフィル）・削除
    #[napi]
    pub fn update_table(&self, name: String, changes: Value) -> napi::Result<()> {
        self.bridge()?.update_table(&name, &changes).map_err(js_err)
    }

    /// options: `{ condition?, values?, names? }`
    #[napi]
    pub fn put(&self, table: String, item: Value, options: Option<Value>) -> napi::Result<()> {
        self.bridge()?
            .put(&table, &item, options.as_ref())
            .map_err(js_err)
    }

    /// key: `{ pk属性: 値, sk属性?: 値 }` → item または null
    #[napi]
    pub fn get(&self, table: String, key: Value) -> napi::Result<Option<Value>> {
        self.bridge()?.get(&table, &key).map_err(js_err)
    }

    /// 旧 item（無ければ null）を返す。options: `{ condition?, values?, names? }`
    #[napi]
    pub fn delete(
        &self,
        table: String,
        key: Value,
        options: Option<Value>,
    ) -> napi::Result<Option<Value>> {
        self.bridge()?
            .delete(&table, &key, options.as_ref())
            .map_err(js_err)
    }

    /// params: `{ update: "SET ...", condition?, values?, names? }` → ALL_NEW の item
    #[napi]
    pub fn update(&self, table: String, key: Value, params: Value) -> napi::Result<Value> {
        self.bridge()?.update(&table, &key, &params).map_err(js_err)
    }

    /// params: `{ keyCondition, filter?, values?, names?, index?, limit?,
    ///            scanForward?, startKey? }` → `{ items, lastEvaluatedKey? }`
    #[napi]
    pub fn query(&self, table: String, params: Value) -> napi::Result<Value> {
        self.bridge()?.query(&table, &params).map_err(js_err)
    }

    /// params: `{ filter?, values?, names?, limit?, startKey? }`
    #[napi]
    pub fn scan(&self, table: String, params: Value) -> napi::Result<Value> {
        self.bridge()?.scan(&table, &params).map_err(js_err)
    }

    /// LoomDB の差別化: N テーブル JOIN。
    /// `{ root, steps, filter?, values?, names?, select? }` → `{ rows, warnings }`
    #[napi]
    pub fn join(&self, params: Value) -> napi::Result<Value> {
        self.bridge()?.join(&params).map_err(js_err)
    }

    /// ops: `[{ put } | { update } | { delete } | { conditionCheck }]` を
    /// 1 txn で all-or-nothing 適用（件数無制限）。不成立は TransactionCanceled。
    #[napi]
    pub fn transact_write(&self, ops: Value) -> napi::Result<()> {
        self.bridge()?.transact_write(&ops).map_err(js_err)
    }

    /// keys: `[{ table, key }]` → 単一スナップショットで item | null の配列（同順）
    #[napi]
    pub fn transact_get(&self, keys: Value) -> napi::Result<Value> {
        self.bridge()?.transact_get(&keys).map_err(js_err)
    }

    /// ローカルでは transactGet と同一意味論（UnprocessedKeys は常に空）。
    #[napi]
    pub fn batch_get(&self, keys: Value) -> napi::Result<Value> {
        self.bridge()?.batch_get(&keys).map_err(js_err)
    }

    /// params: `{ puts?: [{table, item}], deletes?: [{table, key}] }`（件数無制限）
    #[napi]
    pub fn batch_write(&self, params: Value) -> napi::Result<()> {
        self.bridge()?.batch_write(&params).map_err(js_err)
    }

    /// TTL 失効項目を budget 件まで物理削除し、削除数を返す。
    #[napi]
    pub fn sweep_expired(&self, table: String, budget: u32) -> napi::Result<u32> {
        self.bridge()?
            .sweep_expired(&table, budget as usize)
            .map(|n| n as u32)
            .map_err(js_err)
    }
}
