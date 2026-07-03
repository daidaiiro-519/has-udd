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
}
