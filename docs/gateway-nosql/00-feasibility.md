# 実現可能性検討 — ゲートウェイ向け軽量ローカル NoSQL（仮称 nanodyn）

## 1. 結論

**実現性: 高い。** 方式＝**「順序付き KV の ACID 埋め込みエンジン（redb）を土台に、DynamoDB のデータモデルと API を薄く載せる」**。分散 DB の難所（パーティション・スループット・整合性モデル・障害回復）は、**ローカル単一端末では丸ごと不要**になり、常に強整合で済む。本質は「DynamoDB の API とデータモデルの再現」で、そこは有界かつ現実的。

## 2. 中核判断：ストレージは自作せず redb を土台にする

DB で最も難しく最もバグりやすいのは**クラッシュ安全な ACID ストレージ**（トランザクション・ロールバック・電源断からの回復）。ここは自作しない。

| 候補 | 言語/依存 | ACID | サイズ | DynamoDB 適合 | 判定 |
|---|---|---|---|---|---|
| **redb** | pure Rust・C 依存なし | ✓（MVCC・単一ファイル） | 小（バイナリ内包） | ◎ 順序付き KV＝(pk,sk) に素直 | **採用** |
| SQLite | C（~1MB） | ✓ | 小 | ○ だが JSON を SQL に押し込む | 非採用（構造不一致・C依存） |
| LMDB/libmdbx | C（極小 ~64KB） | ✓ | 最小 | ◎ | 次点（C 依存・mmap 制約） |
| sled | pure Rust | ✓ | 中 | ◎ | 非採用（保守停滞・肥大） |
| 自作storage | — | 要自作 | 最小可能 | — | **却下**（クラッシュ安全性の再発明＝高リスク） |

**redb を選ぶ理由:** pure Rust（gateway クロスコンパイルが容易・C ツールチェーン不要）／ACID トランザクション＋ロールバックが内蔵／順序付き KV は DynamoDB の (pk, sk) と範囲スキャンに native に対応（SQL の impedance が無い）。

## 3. 非機能要件の充足

| NFR | 実現方法 |
|---|---|
| **極小サイズ** | redb を静的リンク。重い依存を持たない（式パーサは自作）。`opt-level="z"` + LTO + strip + `panic=abort`。目標バイナリ数百 KB〜1MB 台 |
| **トランザクション** | redb の write transaction（1 write txn／複数 read txn の MVCC）。`TransactWriteItems` = 1 write txn 内の複数変更で all-or-nothing |
| **ロールバック** | write txn を commit しなければ破棄＝ロールバック。redb がクラッシュ回復も担保 |
| **DynamoDB 機能** | データモデル＋式言語をレイヤで実装（§5） |
| **JOIN（差別化）** | 任意 query 層（`nanodyn-query`）で index-nested-loop join を実装。読取専用・ローカル特権で inner/left・N テーブル多段（left-deep）を提供（spec §10） |

## 4. データモデルの KV マッピング

```
主データ（1論理テーブル）:
  key   = encode(pk) 0x00 encode(sk)      … 順序保存エンコード
  value = item 全体（MessagePack）

GSI/LSI（索引ごとに redb の別テーブル）:
  key   = encode(idxpk) 0x00 encode(idxsk) 0x00 encode(pk) 0x00 encode(sk)
  value = 射影属性（KEYS_ONLY なら空／INCLUDE/ALL なら該当属性）
  → 主データと同一 write txn で維持＝常に一貫
```

- **GetItem** = 主テーブルの点取得
- **Query** = `key` の prefix `encode(pk) 0x00` 範囲スキャン＋SK 条件
- **Scan** = 全域スキャン＋Filter
- **順序保存エンコード**が肝：数値 N 型は符号・桁を保った byte 列にして範囲比較を正しくする（後述 spec §データ型）

## 5. DynamoDB 機能の実現可否

| 機能 | 可否 | 実現法 |
|---|---|---|
| Put/Get/Update/Delete Item | ◎ | KV 操作 |
| Query（KeyConditionExpression） | ◎ | prefix 範囲スキャン |
| Scan＋FilterExpression | ◎ | 全域＋式評価 |
| ConditionExpression（条件付き書込） | ◎ | write txn 内 read→判定→書込 |
| UpdateExpression（SET/ADD/REMOVE/DELETE） | ◎ | item に式適用→書戻し |
| 原子カウンタ（ADD） | ◎ | txn 内で読み+加算 |
| TransactWriteItems / TransactGetItems | ◎ | 1 txn に集約（ローカルは 25/100 item 制限も不要） |
| BatchWriteItem / BatchGetItem | ◎ | txn 内ループ |
| LSI / GSI | ◎ | 索引テーブルを同一 txn で維持（**常に強整合**） |
| TTL | ○ | `ttl` 属性＋読取時の遅延失効＋バックグラウンド掃引 |
| 楽観ロック（version） | ◎ | item にバージョン属性・ConditionExpression で実現 |
| Streams（CDC） | △ | 変更ログテーブルを txn 内で追記（任意層・後続） |
| PartiQL | △ | サブセットを後付け（任意・後続） |
| Provisioned/パーティション/マルチリージョン/IAM | ✕(不要) | ローカル単一端末では無意味＝簡素化 |

## 6. 互換方針（確定）：コア＋任意ワイヤ層

- **コア** = 埋め込み Rust ライブラリ（型付き API）。これ単体で最小構成。
- **任意ワイヤ層** = 別 crate で DynamoDB JSON プロトコル（`X-Amz-Target`）のサブセットを実装。要れば有効化し、既存 AWS SDK コードを繋げる。サイズが要らなければ切り離す。

## 7. 主要リスクと対策

| リスク | 対策 |
|---|---|
| **式言語（Condition/Update/KeyCondition/Filter/Projection）の実装量**＝最大の作り込み | 文法は有界。手書き再帰下降パーサ＋AST 評価器。property test で網羅（§test-standard） |
| redb の成熟度・障害回復の信頼性 | 電源断シミュレーション試験・fsync 方針の検証。最悪 LMDB へ差し替え可能なよう **StorageEngine port で抽象化**（架構で疎結合） |
| 順序保存エンコードの正しさ（特に N 型範囲） | エンコード単体テスト＋property test（round-trip・順序単調性） |
| **JOIN の性能（索引なし結合が scan フォールバック・多段で累積）** | 結合キーへの索引を推奨・未索引時は警告。v1 は left-deep（宣言順）＝利用者が結合順を制御し、コストベース最適化は後続に回して複雑度を抑制 |

## 8. 実装可能性の総評

- 「ストレージは redb 再利用・DynamoDB 層は自作」で、**新規に書く難所は式評価器と索引維持と KV マッピングに限定**され、いずれも有界。
- ACID・ロールバック・クラッシュ安全は redb が担保＝**最大の技術リスクを外注**。
- サイズ・トランザクションの NFR は方式選択の時点でほぼ満たされる。JOIN は読取専用・任意層ゆえコアサイズに影響しない。
- → **実装を起こせる。** 次ページ以降に spec / tech-stack / architecture / coding-standard / test-standard。
