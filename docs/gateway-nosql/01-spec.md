# 仕様書 — nanodyn（ゲートウェイ向け軽量ローカル DynamoDB 風 NoSQL）

対象読者: 実装者。この文書だけで v1 を起こせる粒度を目指す。

---

## 1. 目的・スコープ

- 容量制約のあるゲートウェイ端末で動く**埋め込み KV/ドキュメント DB**。
- DynamoDB のデータモデルと操作を**可能な限り**再現。ローカル単一端末前提（分散機能は範囲外）。
- ACID トランザクション・ロールバックを提供。
- **DynamoDB との差別化＝JOIN（結合）**。ローカル特権を活かし、DynamoDB に無い結合クエリを任意 query 層で提供（§10）。

**v1 スコープ:** CRUD / Query / Scan / トランザクション / ロールバック / 式言語 / 条件付き・原子更新 / GSI・LSI / TTL / 楽観ロック / **JOIN（inner・left outer）**。
**設計原則:** ローカル単一端末の特権として、DynamoDB の**分散由来の制約は撤廃**する（バッチ操作数の上限なし・GSI も常に強整合 等）。
**v1 範囲外（後続）:** Streams、PartiQL、SQL クエリ言語、3テーブル以上の多段 JOIN、SQLite 等からの移行、ワイヤ完全互換の全 API。

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
- **操作数の上限なし**（DynamoDB の 25/100 件制限は分散由来ゆえ撤廃・§11）。トレードオフ: 巨大 `transact_write` は 1 個の大きな write txn となり、commit まで単一ライタを占有しメモリを保持する（ローカル用途では許容）。

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

## 10. 結合（JOIN）— nanodyn 拡張

DynamoDB に無い機能で、**本 DB の差別化の核**。ローカル単一端末なので、分散環境では高コストな結合を現実的に提供できる。**読み取り専用**であり、書込パス・トランザクション意味論には一切影響しない。

- **配置**: 任意 query 層（別 crate `nanodyn-query`・feature `join`）。要らない構成では丸ごと除外でき、コア常駐サイズに影響しない。

### 10.1 対応範囲（v1）
- **結合種別**: `INNER` と `LEFT OUTER`。
- **対象**: **2 テーブル間**（左＝outer / 右＝inner）。3 テーブル以上の多段は v1 範囲外（後続）。
- **結合条件**: 等値結合（equi-join）。`left.attrA = right.attrB` 形式。

### 10.2 アルゴリズム（index-nested-loop join）
```
左テーブルを query/scan で走査（左の key_condition/filter を先に適用＝プッシュダウン）
  各 左item について:
    結合キー値 v = left.attrA を取り出す
    右テーブルを v で参照:
      - 右の attrB に索引あり → 索引を点/範囲引き（高速）
      - 索引なし          → scan フォールバック（低速・警告を出す）
    右マッチが複数なら 左item × 各右item を出力（1対多を展開）
    INNER: 右マッチ 0 件なら当該左行は出力しない
    LEFT : 右マッチ 0 件でも左行を出力し、右由来の属性は欠落（射影で NULL 相当）
  結合後 filter（post-join filter）を適用
```
- **指針**: 結合キー（右 `attrB`）に**索引を貼ることを推奨**。無ければ動くが scan フォールバックで遅い。この指針は「後付け索引」の設計方針と噛み合う。

### 10.3 一貫性
- 結合全体を**単一 redb read txn 内**で実行 → 両テーブルを**同一 MVCC スナップショット**で読む。走査途中の他書込に汚されない一貫した結果を返す。

### 10.4 インターフェース（2 形態・同一 `JoinPlan` に落とす）
- **A: 型付きビルダー API**（コア利用者向け・依存ゼロ）
  ```
  join(left_table)
    .inner(right_table).on(left="attrA", right="attrB")   // or .left(...)
    .left_where(key_condition?, filter?)
    .right_index(name?)                                    // 明示指定可
    .filter(post_join_cond?)
    .select([ "L.x", "R.y", ... ])
    .page(limit?, exclusive_start_key?)
    -> JoinPage
  ```
- **B: 宣言的 `JoinSpec`**（構造体/JSON・ワイヤ層から投入可）。A と同じ内部表現へ変換。
- **SQL 文字列（C）は採用しない**（サイズ・工数が跳ね、フル SQL への入口になるため）。

### 10.5 射影と名前衝突
- 出力属性は**別名接頭辞**で区別（既定: 左=`L.`, 右=`R.`、テーブル別名指定も可）。
- LEFT で右が未マッチの行は、右由来属性を**欠落**（`attribute_exists` は偽・射影上は NULL 相当）。

### 10.6 ページング
- 左テーブルの走査位置を `last_evaluated_key` として返し、ストリーミングにページ分割。1 ページは §11 の上限（1MB or limit 件）に従う。

---

## 11. 制限値（既定・設定可能）

| 項目 | 既定 |
|---|---|
| 最大項目サイズ | 400 KB |
| 最大キー長（pk/sk 各） | 2 KB |
| Query/Scan 1 ページ最大 | 1 MB or limit 件 |
| transact_write / batch_write 最大操作数 | 無制限（ローカル・メモリ次第。DynamoDB の 25/100 件制限は撤廃） |

---

## 12. 任意ワイヤ層（別 crate）

- HTTP エンドポイント1つ。`X-Amz-Target: DynamoDB_20120810.{Op}` を見て JSON を型付き API に橋渡し。
- v1 対応 Op（サブセット）: PutItem/GetItem/UpdateItem/DeleteItem/Query/Scan/BatchWriteItem/BatchGetItem/TransactWriteItems/TransactGetItems/CreateTable/DeleteTable/DescribeTable。
- **JOIN は nanodyn 固有の拡張 Op**（DynamoDB プロトコルには存在しない）。ワイヤで公開する場合は独自ターゲット名で `JoinSpec`（§10.4 B）を受ける。既存 AWS SDK からは呼べない前提。
- サイズが要らない構成では丸ごと除外可能（feature flag）。
