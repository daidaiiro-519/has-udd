# 仕様書 — LoomDB（ゲートウェイ向け軽量ローカル DynamoDB 風 NoSQL）

対象読者: 実装者。この文書だけで v1 を起こせる粒度を目指す。

---

## 1. 目的・スコープ

- 容量制約のあるゲートウェイ端末で動く**埋め込み KV/ドキュメント DB**。
- DynamoDB のデータモデルと操作を**可能な限り**再現。ローカル単一端末前提（分散機能は範囲外）。
- ACID トランザクション・ロールバックを提供。
- **DynamoDB との差別化＝JOIN（結合）**。ローカル特権を活かし、DynamoDB に無い結合クエリを任意 query 層で提供（§10）。

**v1 スコープ:** CRUD / Query / Scan / トランザクション / ロールバック / 式言語 / 条件付き・原子更新 / GSI・LSI / TTL / 楽観ロック / **JOIN（inner・left outer・N テーブル多段）**。
**設計原則:** ローカル単一端末の特権として、DynamoDB の**分散由来の制約は撤廃**する（バッチ操作数の上限なし・GSI も常に強整合 等）。
**v1 範囲外（後続）:** Streams、PartiQL、SQL クエリ言語、JOIN のコストベース最適化（v1 は宣言順 left-deep）、SQLite 等からの移行、ワイヤ完全互換の全 API。

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
| N | 数値 | 10進・**38 有効桁**・範囲 1E-130〜9.9E+125（DynamoDB 準拠）。文字列表現＋順序保存エンコード。超過は `ValidationError` |
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
- `update_table(name, add_indexes, remove_indexes) -> ()` — **GSI の後付け追加・削除**（§7・差別化の一つ）。

### 4.2 項目操作
- `put_item(table, item, condition?) -> ()` — condition 不成立で `ConditionalCheckFailed`。
- `get_item(table, key, consistent?, projection?) -> Option<Item>` — ローカルは常に強整合。
- `update_item(table, key, update_expr, condition?, return_values?) -> Option<Item>`
- `delete_item(table, key, condition?, return_values?) -> Option<Item>`
- `return_values`: v1 は `NONE` / `ALL_OLD` / `ALL_NEW` に対応（`UPDATED_OLD/UPDATED_NEW` は後続）。

### 4.3 問い合わせ
- `query(table, key_condition, opts) -> Page`
  - `opts`: index?（GSI/LSI 名）、filter?、projection?、`scan_forward`、`limit`、`exclusive_start_key`。
  - 戻り: `items` ＋ `last_evaluated_key`（ページング）。
  - **`limit` は Filter 適用「前」に効く**（DynamoDB 準拠）: limit 件読んでから filter するため、結果は limit 件未満になり得る。互換の要注意点として適合テストで固定。
- `scan(table, opts) -> Page` — index?・filter?・segment/total_segments（並列スキャン任意）。

### 4.4 バッチ・トランザクション
- `batch_get(requests) -> ...`
- `batch_write(puts, deletes) -> ...`（非トランザクション・冪等ループ）
- `transact_write(ops) -> ()` — Put/Update/Delete/ConditionCheck を**1 txn で all-or-nothing**。
- `transact_get(keys) -> Vec<Option<Item>>` — 一貫スナップショット読取。
- ローカルは分散の部分失敗が無いため **`UnprocessedItems`/`UnprocessedKeys` は常に空**（batch は全処理 or エラー）。
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
- **同時実行モデル**: 複数 read txn ＋単一 write txn（redb）。書込は直列化され、競合時は**ブロック**（エラーにしない）。DB ハンドルは `Send + Sync`。**単一プロセス前提**（ファイルロックで多重オープンを拒否）。
- **耐久性（fsync）**: 既定＝commit ごとに fsync（電源断でも commit 済みは失われない）。open 時に `Durability::{Immediate(既定), Eventual}` を選択可。`Eventual` は性能優先で直近 commit を失い得る — ゲートウェイの電源断特性に応じて利用者が選ぶ。

---

## 7. 二次索引（GSI / LSI）

- **定義**: 索引名・索引 pk（＋任意 sk）属性・射影種別。
- **LSI**: 主テーブルと同じ pk・異なる sk。作成後不変。
- **GSI**: 任意属性を pk/sk に。**ローカルは同一 txn 維持なので常に強整合**（DynamoDB の GSI は結果整合だが、ここでは強整合で上位互換）。
- 索引に無い属性を持つ項目は当該索引に載らない（sparse index）。
- **v1 の格納方針**: 索引エントリの値は空（KEYS_ONLY 相当）とし、読み取りは常に main から全属性を返す（ローカルは同一ファイル・同一 txn 参照が安価かつ強整合＝DynamoDB の射影制限の上位互換）。射影属性の格納（§3 の INCLUDE/ALL）は読取最適化として後続。
- **後付け追加（差別化）**: GSI は `update_table` で**いつでも追加・削除できる**。追加時は既存データを全走査して索引を**バックフィル**する（1 write txn で構築＝完成した索引だけが見える。巨大テーブルでは長い txn になり他の書込をブロックする点は明示のトレードオフ）。LSI は DynamoDB 準拠で作成時のみ。

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
| `ResourceInUse` | `create_table` で同名テーブルが既に存在 |
| `ItemSizeLimitExceeded` | 項目サイズ上限超過（設定可能・既定 400KB） |

---

## 10. 結合（JOIN）— LoomDB 拡張

DynamoDB に無い機能で、**本 DB の差別化の核**。ローカル単一端末なので、分散環境では高コストな結合を現実的に提供できる。**読み取り専用**であり、書込パス・トランザクション意味論には一切影響しない。

- **配置**: 任意 query 層（別 crate `loom-query`・feature `join`）。要らない構成では丸ごと除外でき、コア常駐サイズに影響しない。

### 10.1 対応範囲（v1）
- **結合種別**: `INNER` と `LEFT OUTER`（結合エッジごとに指定可）。
- **対象**: **N テーブル**の多段結合。構造は最初から複数テーブルを前提とする（`JoinStep` の順序付きリスト＝**left-deep join tree**）。
- **結合条件**: 等値結合（equi-join）。各ステップは `<既出のいずれかの入力>.attrX = <新テーブル>.attrY` 形式。
- **別名（alias）必須**: 同一テーブルの自己結合や属性名衝突に対応するため、各入力にエイリアスを付ける。

### 10.2 データ構造（複数テーブル前提）
```
JoinQuery {
  root:   InputRef { table, alias, key_condition?, filter?, index? }   // 駆動表（最外）
  steps:  [ JoinStep, ... ]        // 適用順＝left-deep。0 個なら単表クエリと等価
  filter: post_join_cond?          // 結合後フィルタ（全入力の属性を参照可）
  select: [ projection_path, ... ] // "alias.attr" 形式
  page:   { limit?, exclusive_start_key? }
}
JoinStep {
  input:   InputRef { table, alias, filter?, index? }   // 追加する表
  kind:    INNER | LEFT
  on:      [ Eq { left: "existingAlias.attrX", right: "newAlias.attrY" }, ... ]  // 複合キー可（AND）
}
```
- `steps` を増やすだけで 2, 3, 4… テーブルへ自然に拡張。v1 で件数上限は設けない（§11・実用上はプラン深さで自律制御）。
- post-join `filter` / `select` の属性パスは **`alias.attr` 修飾形**を第一級で受ける（§5 の式文法にエイリアス修飾パスを拡張）。

### 10.3 アルゴリズム（index-nested-loop・多段）
```
root を query/scan で走査（root の key_condition/filter を先に適用＝プッシュダウン）
  各 中間タプル t（それまでに結合済みの全入力の束）について、steps を順に適用:
    step.on の left 側を t から評価 → 結合キー値 v
    step.input を v で参照:
      - input.attrY に索引あり → 索引を点/範囲引き（高速）
      - 索引なし              → scan フォールバック（低速・警告を出す）
    マッチ複数 → t × 各マッチ を展開（1対多）
    INNER: マッチ 0 件なら当該タプルを捨てる（以降の step に進めない）
    LEFT : マッチ 0 件でも t を残し、当該 input 由来の属性は欠落（NULL 相当）
  全 step 通過後、post-join filter を適用 → select で射影
```
- **指針**: 各ステップの結合キー（`input.attrY`）に**索引を貼ることを推奨**。無ければ動くが scan フォールバックで遅い（多段では効きが累積するので特に）。
- **警告の伝達**: scan フォールバックの発生は logging ではなく**結果メタデータ**（`JoinPage.warnings`）で返す（ライブラリの logging 既定オフ方針と整合し、呼び出し側が機械的に検知できる）。
- **プラン順序**: v1 は宣言順（left-deep）をそのまま実行＝**利用者が結合順を制御**。コストベース最適化は後続（範囲外）。

### 10.4 一貫性
- 結合全体を**単一 redb read txn 内**で実行 → 参加する全テーブルを**同一 MVCC スナップショット**で読む。走査途中の他書込に汚されない一貫結果を返す。

### 10.5 インターフェース（2 形態・同一 `JoinQuery` に落とす）
- **A: 型付きビルダー API**（コア利用者向け・依存ゼロ・`.join()` を鎖状に重ねて N テーブル）
  ```
  join(root_table).as("o").where(key_condition?, filter?)
    .inner("users").as("u").on("o.userId", "u.id")
    .left("addresses").as("a").on("u.id", "a.userId").index("byUser")
    .inner("plans").as("p").on("u.planId", "p.id")
    .filter(post_join_cond?)
    .select([ "o.id", "u.name", "a.city", "p.tier" ])
    .page(limit?, exclusive_start_key?)
    -> JoinPage
  ```
- **B: 宣言的 `JoinQuery`**（§10.2 の構造体/JSON・ワイヤ層から投入可）。A と同じ内部表現へ変換。
- **SQL 文字列（C）は採用しない**（サイズ・工数が跳ね、フル SQL への入口になるため）。

### 10.6 射影と名前衝突
- 出力属性は**エイリアス接頭辞**で一意化（`"o.id"` / `"u.name"`）。エイリアス必須ゆえ自己結合・同名属性も衝突しない。
- LEFT で未マッチの入力は、その入力由来属性を**欠落**（`attribute_exists` は偽・射影上は NULL 相当）。

### 10.7 ページング
- **root（駆動表）の走査位置**を `last_evaluated_key` として返し、ストリーミングにページ分割。1 ページは §11 の上限（1MB or limit 件）に従う。1 タプル展開の途中でページ境界に当たった場合の再開位置も root キー＋展開オフセットで表現。

---

## 11. 制限値（既定・設定可能）

| 項目 | 既定 |
|---|---|
| 最大項目サイズ | 400 KB |
| 最大キー長（pk/sk 各） | 2 KB |
| Query/Scan 1 ページ最大 | 1 MB or limit 件 |
| transact_write / batch_write 最大操作数 | 無制限（ローカル・メモリ次第。DynamoDB の 25/100 件制限は撤廃） |
| M/L の入れ子深度 | 32 段（DynamoDB 準拠） |
| N の有効桁・指数範囲 | 38 桁・1E-130〜9.9E+125（DynamoDB 準拠） |
| 空文字列・空バイナリ | 非キー属性で許容（現行 DynamoDB 準拠） |
| 空集合（SS/NS/BS） | 不可＝ `ValidationError`（DynamoDB 準拠） |
| テーブル名 | 3〜255 文字・`[a-zA-Z0-9_.-]`（DynamoDB 準拠。`:` を含む名前は内部予約） |

---

## 12. 任意ワイヤ層（別 crate）

- HTTP エンドポイント1つ。`X-Amz-Target: DynamoDB_20120810.{Op}` を見て JSON を型付き API に橋渡し。
- v1 対応 Op（サブセット）: PutItem/GetItem/UpdateItem/DeleteItem/Query/Scan/BatchWriteItem/BatchGetItem/TransactWriteItems/TransactGetItems/CreateTable/DeleteTable/DescribeTable。
- **JOIN は LoomDB 固有の拡張 Op**（DynamoDB プロトコルには存在しない）。ワイヤで公開する場合は独自ターゲット名で `JoinSpec`（§10.4 B）を受ける。既存 AWS SDK からは呼べない前提。
- **セキュリティ既定**: 認証なし・**既定バインドは 127.0.0.1**（ローカル専用）。外部公開する場合の保護（TLS・認証）は利用者責務。
- サイズが要らない構成では丸ごと除外可能（feature flag）。

---

## 13. 運用（フォーマット・保守）

- **オンディスク形式のバージョニング**: `meta` に `format_version` を保持。互換変更は自動読替え、非互換変更は crate のメジャー版数と連動して上げ、旧形式は明示エラー（将来 `migrate` を提供）。OSS として長期利用される前提の必須事項。
- **compaction**: 削除・更新で生じた空き領域は明示 API `compact()` で回収（redb の compact に委譲）。呼ぶタイミングは利用者制御。
- **バックアップ**: **open 中のファイルコピーは不可**。close 後にファイルコピー、または（後続）read txn スナップショットから複製する `backup(path)`。
- **統計**: `stats(table) -> { item_count, file_bytes }`（DescribeTable の ItemCount 相当・O(1)）。
