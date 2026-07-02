# アーキテクチャ — LoomDB

技術方式＝**ポートとアダプター（ヘキサゴナル）**。ストレージ（redb）・ワイヤ（HTTP）・CLI を外側の交換可能なアダプターにし、DB のドメイン（データモデル・式・索引・トランザクション意味論）を内側に隔離する。結合など読取専用の拡張は query 層に載せる。

## 1. レイヤーと依存方向

| レイヤー | 責務 | 依存してよい先 |
|---|---|---|
| domain | データモデル・属性値・キーエンコード・式 AST/評価・索引意味論・エラー | なし（最内） |
| application | 各操作（PutItem/Query/…）のユースケース調整・トランザクション境界 | domain・ports |
| ports | `StorageEngine` / `Clock` などの抽象 | domain 型のみ |
| adapters | redb 実装・ワイヤ・CLI | application・ports |

**規則:** 依存は内向きのみ。domain は redb / serde / HTTP を知らない。

## 2. ワークスペース構成（crate 分割＝サイズ制御の要）

```
loom/
├─ crates/
│  ├─ loom-core/        # ライブラリ本体（domain + application + ports）
│  │   ├─ domain/          #   model, attribute, key_codec, expr(parser/ast/eval), index, error
│  │   ├─ application/     #   usecases: put_item, get_item, update_item, query, scan, transact_write, ...
│  │   └─ ports/           #   StorageEngine, Clock
│  ├─ loom-redb/        # outbound adapter: StorageEngine を redb で実装
│  ├─ loom-query/       # 任意（feature "join"）: 結合/集計など読取専用クエリ層（index-nested-loop join）
│  ├─ loom-wire/        # inbound adapter（任意・feature "wire"）: DynamoDB JSON プロトコル
│  ├─ loom-node/        # inbound adapter（任意）: Node.js バインディング（napi-rs・DocumentClient 風）
│  ├─ loom-py/          # inbound adapter（任意）: Python バインディング（PyO3・boto3 風）
│  └─ loom-cli/         # inbound adapter（任意）: 端末操作
└─ Cargo.toml (workspace)
```

- **gateway への最小配布** = `loom-core` + `loom-redb` のみ。query/wire/node/py/cli は必要時。
- **多言語対応も inbound adapter** = Node.js（napi-rs）/ Python（PyO3）は application を叩く薄い変換層で、コア無改修・コアサイズに不影響。加えて wire 層経由なら公式 AWS SDK がそのまま使える（spec §12）。
- **JOIN（LoomDB 拡張）は `loom-query` に隔離** = 読取専用・application を叩く薄い層。要らない構成では除外しコア常駐サイズに影響させない。

## 3. ポート（抽象）

```rust
/// 順序付き KV の ACID ストレージ。redb / LMDB を差し替え可能に。
pub trait StorageEngine {
    type Txn<'a>: WriteTxn where Self: 'a;
    fn begin_write(&self) -> Result<Self::Txn<'_>, DbError>;
    fn begin_read(&self) -> Result<impl ReadTxn + '_, DbError>;
}
pub trait WriteTxn {
    fn get(&self, table: TableId, key: &[u8]) -> Result<Option<Vec<u8>>, DbError>;
    fn put(&mut self, table: TableId, key: &[u8], val: &[u8]) -> Result<(), DbError>;
    fn delete(&mut self, table: TableId, key: &[u8]) -> Result<(), DbError>;
    fn range(&self, table: TableId, lo: &[u8], hi: &[u8]) -> Result<KvIter, DbError>;
    fn commit(self) -> Result<(), DbError>;   // drop = rollback
}
pub trait Clock { fn now_epoch(&self) -> i64; }
```

- **トランザクション境界は application 層が持つ**：1 操作＝1 write txn を張り、成功時のみ commit（drop で自動ロールバック）。索引更新も同じ txn 内。

## 4. DDD 概念 → 配置

| 概念 | 配置 | 形 |
|---|---|---|
| value-object | `domain/`（`AttributeValue`, `Key`, `TableDef`, `IndexDef`, `Expr`） | 不変・`enum`/`struct` |
| domain-service | `domain/`（`key_codec`, `expr::eval`, `index::maintain`） | ステートレス純関数 |
| usecase | `application/usecases/{op}.rs`（`put_item`, `query`, `transact_write`…） | 入口関数・txn を調停・仕様 §4 と対応 |
| aggregate 相当 | 明示クラスは作らない。整合は txn ＋索引維持サービスで担保 | — |
| inbound-adapter | `loom-wire` / `loom-node` / `loom-py` / `loom-cli` | 変換のみ・ロジック持たない |
| outbound-adapter | `loom-redb`（StorageEngine 実装） | redb 依存はここに閉じ込め |
| query 層（拡張） | `loom-query`（`join::inner` / `join::left`） | 読取専用・複数 read を 1 スナップショットで調停 |

## 5. 主要ドメイン型（骨子）

```rust
pub enum AttributeValue { S(String), N(Number), B(Vec<u8>), Bool(bool), Null,
                          M(Map<String, AttributeValue>), L(Vec<AttributeValue>),
                          SS(Set<String>), NS(Set<Number>), BS(Set<Vec<u8>>) }
pub struct Item(pub Map<String, AttributeValue>);
pub struct Key { pub pk: AttributeValue, pub sk: Option<AttributeValue> }
pub struct TableDef { pub name: String, pub key: KeySchema,
                      pub indexes: Vec<IndexDef>, pub ttl_attr: Option<String> }
pub enum Expr { /* condition/update/key の AST */ }
```

## 6. 制御フロー（PutItem 例）

```
wire/cli/直接呼出
  → application::put_item(table, item, condition?)
       1. TableDef を meta から取得
       2. StorageEngine.begin_write()
       3. condition? を現行 item に対し評価（domain::expr::eval）→ 偽なら rollback して ConditionalCheckFailed
       4. main へ put（key = key_codec、value = rmp）
       5. 影響 GSI/LSI を index::maintain で差分更新（同一 txn）
       6. commit（失敗すれば全体ロールバック）
```

## 7. 交換可能性（設計の効き所）

- **StorageEngine port** により、redb の成熟度が懸念になれば LMDB/libmdbx へ差替可能（domain/application は無改修）。
- **inbound の複数化**：ライブラリ直呼び／ワイヤ／CLI は同じ application を叩く薄いアダプター。
- **拡張の隔離**：JOIN 等の読取専用拡張は `loom-query`（feature `join`）にのみ存在し、常駐 DB のサイズ・依存に影響しない。要らない gateway には配布しない。
