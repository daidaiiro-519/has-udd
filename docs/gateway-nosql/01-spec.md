# 仕様書 — nanodyn（ゲートウェイ向け軽量ローカル DynamoDB 風 NoSQL）

対象読者: 実装者。この文書だけで v1 を起こせる粒度を目指す。

---

## 1. 目的・スコープ

- 容量制約のあるゲートウェイ端末で動く**埋め込み KV/ドキュメント DB**。
- DynamoDB のデータモデルと操作を**可能な限り**再現。ローカル単一端末前提（分散機能は範囲外）。
- ACID トランザクション・ロールバック・SQLite からの移行を提供。

**v1 スコープ:** CRUD / Query / Scan / トランザクション / ロールバック / 式言語 / 条件付き・原子更新 / GSI・LSI / TTL / 楽観ロック / SQLite 移行ツール。
**v1 範囲外（後続）:** Streams、PartiQL、ワイヤ完全互換の全 API。

---

## 2. データモデル

### 2.1 テーブルと項目
- **Table**: 論理的な項目集合。`name`、キースキーマ（`pk`必須・`sk`任意）、索引定義、TTL 属性名を持つ。
- **Item**: 属性名→属性値のマップ。スキーマレス（キー属性のみ必須）。
- 単一物理ストア（redb ファイル1つ）に複数テーブルを収容可（テーブル名を key に前置）。

### 2.2 属性型（DynamoDB 準拠）
| 記号 | 型 | 備考 |
|---|---|---|
| S | 文字列（UTF-8） | |
| N | 数値 | 10進・任意精度（文字列表現＋順序保存エンコード） |
| B | バイナリ | |
| BOOL | 真偽 | |
| NULL | ヌル | |
| M | マップ | 入れ子 |
| L | リスト | 入れ子 |
| SS/NS/BS | 文字列/数値/バイナリ集合 | |

### 2.3 キーとエンコード（順序保存）
- `pk`（partition key）と任意の `sk`（sort key）。型は S / N / B のいずれか。
- **順序保存エンコード** `encode(v)`:
  - S: UTF-8 バイト列（0x00 を含まない前提、含む場合はエスケープ）
  - N: 符号ビット＋指数＋仮数を辞書式順序＝数値順序になるよう変換（負数・小数対応）
  - B: そのままのバイト列
- 主キー = `encode(table) 0x00 encode(pk) 0x00 encode(sk)`（sk 無しは空）。
- **不変条件**: 同一 (table, pk, sk) は一意。

---

## 3. ストレージ配置（redb）

| redb テーブル | key | value |
|---|---|---|
| `main` | `enc(table) 0x00 enc(pk) 0x00 enc(sk)` | item（MessagePack） |
| `idx:{table}:{index}` | `enc(ipk) 0x00 enc(isk) 0x00 enc(pk) 0x00 enc(sk)` | 射影属性（射影種別に応じ） |
| `meta` | `table:{name}` | テーブル定義（キースキーマ・索引・TTL） |

- **索引維持**: 項目の書込/更新/削除時、影響する全 GSI/LSI を**同一 write txn** で更新。
- **射影**: `KEYS_ONLY`（value 空）/ `INCLUDE(attrs)` / `ALL`。

---

## 4. 操作 API（コア・型付き）

すべて `Result<T, DbError>` を返す。書込系は内部で redb write txn を張る。

### 4.1 テーブル操作
- `create_table(def) -> ()` — キースキーマ・索引・TTL 属性を登録。
- `delete_table(name) -> ()`
- `describe_table(name) -> TableDef`
- `list_tables() -> Vec<String>`

### 4.2 項目操作
- `put_item(table, item, condition?) -> ()` — condition 不成立で `ConditionalCheckFailed`。
- `get_item(table, key, consistent?, projection?) -> Option<Item>` — ローカルは常に強整合。
- `update_item(table, key, update_expr, condition?, return_values?) -> Option<Item>`
- `delete_item(table, key, condition?, return_values?) -> Option<Item>`

### 4.3 問い合わせ
- `query(table, key_condition, opts) -> Page`
  - `opts`: index?（GSI/LSI 名）、filter?、projection?、`scan_forward`、`limit`、`exclusive_start_key`。
  - 戻り: `items` ＋ `last_evaluated_key`（ページング）。
- `scan(table, opts) -> Page` — index?・filter?・segment/total_segments（並列スキャン任意）。

### 4.4 バッチ・トランザクション
- `batch_get(requests) -> ...`
- `batch_write(puts, deletes) -> ...`（非トランザクション・冪等ループ）
- `transact_write(ops) -> ()` — Put/Update/Delete/ConditionCheck を**1 txn で all-or-nothing**。
- `transact_get(keys) -> Vec<Option<Item>>` — 一貫スナップショット読取。

---

## 5. 式言語

DynamoDB の式を実装。字句: 属性パス（`a.b[0]`）、プレースホルダ `#name`（属性名）・`:val`（値）、演算子、関数。

### 5.1 KeyConditionExpression（Query）
```
pk = :v                                   （必須・等価のみ）
[ AND sk {=,<,<=,>,>=} :v
       | AND sk BETWEEN :a AND :b
       | AND begins_with(sk, :prefix) ]
```

### 5.2 FilterExpression / ConditionExpression（共通文法）
```
cond := cond OR cond | cond AND cond | NOT cond | ( cond )
      | operand cmp operand
      | operand BETWEEN operand AND operand
      | operand IN ( operand, ... )
      | function
cmp  := = | <> | < | <= | > | >=
function := attribute_exists(path) | attribute_not_exists(path)
          | attribute_type(path, :type) | begins_with(path, :v)
          | contains(path, :v) | size(path) cmp operand
```

### 5.3 UpdateExpression
```
SET  path = value [, ...]        value := operand | operand +|- operand
                                          | if_not_exists(path, :v)
                                          | list_append(op, op)
REMOVE path [, ...]
ADD    path :num | path :set      （数値加算 / 集合和）
DELETE path :set                  （集合差）
```

### 5.4 ProjectionExpression
- 取得属性をパスのリストで指定。

### 5.5 評価規則
- 式評価は**副作用なしの純関数**（AST × item × プレースホルダ → 値/真偽/新 item）。
- 型不一致は演算ごとに DynamoDB 準拠のエラー/偽。
- 実装は手書き再帰下降パーサ＋AST 評価器（外部パーサ依存を避けサイズ最小化）。

---

## 6. トランザクションとロールバック

- **1 書込操作 = 1 redb write txn**（暗黙）。例外送出・condition 不成立時は txn を破棄＝**自動ロールバック**（部分適用なし）。
- **transact_write** = 明示的に複数変更を 1 txn に束ねる。いずれか失敗で全体ロールバック。
- **分離性**: redb は単一 writer＋MVCC read。読取は書込のスナップショットを見る。ローカルなので**常に強整合**（DynamoDB の eventually consistent は再現不要）。
- **索引一貫性**: 主データと GSI/LSI を同一 txn で更新するため、索引が本体とズレることはない。

---

## 7. 二次索引（GSI / LSI）

- **定義**: 索引名・索引 pk（＋任意 sk）属性・射影種別。
- **LSI**: 主テーブルと同じ pk・異なる sk。作成後不変。
- **GSI**: 任意属性を pk/sk に。**ローカルは同一 txn 維持なので常に強整合**（DynamoDB の GSI は結果整合だが、ここでは強整合で上位互換）。
- 索引に無い属性を持つ項目は当該索引に載らない（sparse index）。

---

## 8. TTL

- テーブルに TTL 属性名を設定。値は epoch 秒（N）。
- **読取時失効**: get/query/scan で期限切れ項目は返さない（論理削除）。
- **掃引**: バックグラウンドまたは明示 `sweep_expired(table, budget)` で物理削除（txn 内・件数上限つき）。

---

## 9. エラーモデル（DynamoDB 準拠の要点）

| エラー | 契機 |
|---|---|
| `ConditionalCheckFailed` | ConditionExpression 不成立 |
| `ItemNotFound` | 対象なし（API により Option で表現） |
| `ValidationError` | 式構文誤り・型不正・キー欠落 |
| `TransactionCanceled` | transact_write のいずれか失敗（理由コード配列） |
| `ResourceNotFound` | テーブル/索引なし |
| `ItemSizeLimitExceeded` | 項目サイズ上限超過（設定可能・既定 400KB） |

---

## 10. SQLite からの移行

- **移行ツール**（別バイナリ `nanodyn-migrate`）:
  1. rusqlite で既存 `.db` を読取専用オープン。
  2. 移行設定（テーブルごとに: どの列を pk/sk にするか・除外列・型マッピング）を受ける。
  3. 各行を item に変換（列→属性、SQLite 型→DynamoDB 型: INTEGER/REAL→N, TEXT→S, BLOB→B, NULL→NULL）。
  4. redb へ txn バッチで書込（索引も構築）。
- **シームレス性**: 移行は一度きり・冪等（再実行で同一結果）。移行後は redb 単独で動作。SQLite ランタイム依存は移行ツールにのみ存在し、コア DB には持ち込まない。

---

## 11. 制限値（既定・設定可能）

| 項目 | 既定 |
|---|---|
| 最大項目サイズ | 400 KB |
| 最大キー長（pk/sk 各） | 2 KB |
| Query/Scan 1 ページ最大 | 1 MB or limit 件 |
| transact_write 最大操作数 | 無制限（ローカル・メモリ次第） |

---

## 12. 任意ワイヤ層（別 crate）

- HTTP エンドポイント1つ。`X-Amz-Target: DynamoDB_20120810.{Op}` を見て JSON を型付き API に橋渡し。
- v1 対応 Op（サブセット）: PutItem/GetItem/UpdateItem/DeleteItem/Query/Scan/BatchWriteItem/BatchGetItem/TransactWriteItems/TransactGetItems/CreateTable/DeleteTable/DescribeTable。
- サイズが要らない構成では丸ごと除外可能（feature flag）。
