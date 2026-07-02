# LoomDB

> ゲートウェイ端末向けの、**極小・埋め込み型ローカル NoSQL**。DynamoDB のデータモデルと API を再現しつつ、DynamoDB には無い **JOIN（結合）** を備える。

**loom＝機織り機。** 順序付き KV という「縦糸」を、JOIN という「横糸」で織り合わせて関係を作る、というこの DB のエンジンをそのまま名前にしている。

## これは何か

- 容量制約のあるゲートウェイ／エッジ端末で動く、**単一バイナリ・サブ MB〜1MB 台**の埋め込み DB。
- **redb（pure Rust の ACID 埋め込み KV）を土台**に、DynamoDB のデータモデル・式言語・二次索引・トランザクションを薄く載せる。クラッシュ安全な ACID ストレージという最大の難所は redb に外注する。
- ローカル単一端末前提。分散 DB の難所（パーティション・整合性モデル・マルチリージョン）は不要になり、**常に強整合**。

## DynamoDB との違い（＝存在意義）

| | DynamoDB | LoomDB |
|---|---|---|
| **JOIN** | 無い | **あり**（inner / left outer・N テーブル多段・index-nested-loop） |
| バッチ/トランザクション操作数 | 25 / 100 件上限 | **上限なし**（ローカル・メモリ次第） |
| GSI の整合性 | 結果整合 | **常に強整合**（同一 txn で索引維持） |
| 二次索引 | 事前設計が要る | 後付けしやすい（強整合） |
| 実行形態 | マネージド分散 | 埋め込み・単一バイナリ・オフライン |

条件付き書込（ConditionExpression）・原子更新（UpdateExpression の SET/ADD/REMOVE/DELETE）・TTL・楽観ロックなど、DynamoDB の主要機能は踏襲する。

## 設計の要点

- **アーキテクチャ**: ポートとアダプター（ヘキサゴナル）。ストレージ（redb）・ワイヤ（HTTP）・CLI を外側の交換可能なアダプターに、DB のドメイン（データモデル・式・索引・トランザクション意味論）を内側に隔離。
- **crate 分割でサイズ制御**: gateway への最小配布は `loom-core` + `loom-redb` のみ。JOIN（`loom-query`）・ワイヤ（`loom-wire`）・CLI（`loom-cli`）は feature / 別 crate で必要時だけ。
- **依存は厳選**: 中核は `redb` / `serde`+`rmp-serde`（MessagePack） / `thiserror`。式言語は外部パーサに依存せず手書き再帰下降で実装。

## ドキュメント

| # | 文書 | 内容 |
|---|---|---|
| 00 | [feasibility](docs/00-feasibility.md) | 実現可能性検討・中核判断（redb 土台） |
| 01 | [spec](docs/01-spec.md) | 仕様（データモデル・操作 API・式言語・JOIN §10・制限値） |
| 02 | [tech-stack](docs/02-tech-stack.md) | 技術スタック・依存方針・サイズ最適化 |
| 03 | [architecture](docs/03-architecture.md) | ヘキサゴナル構成・crate 分割・ポート定義 |
| 04 | [coding-standard](docs/04-coding-standard.md) | コーディング規約 |
| 05 | [test-standard](docs/05-test-standard.md) | テスト規約（property test・サイズ回帰） |

## ワークスペース構成（サンプル雛形）

採用アーキテクチャ（ヘキサゴナル）と技術スタック（redb / serde+rmp-serde / thiserror）を
体現する、**ビルド可能な雛形**を同梱している。

```
loomdb/
├─ Cargo.toml              # workspace（サイズ最優先の release profile 込み）
└─ crates/
   ├─ loom-core/           # ドメイン + application + ports（外部依存を持たない内側）
   │   ├─ domain/          #   attribute, key, table, key_codec, error
   │   ├─ ports/           #   StorageEngine / ReadTxn / WriteTxn / Clock
   │   └─ application/     #   usecases: put_item, get_item
   ├─ loom-redb/           # outbound adapter: StorageEngine を redb で実装
   ├─ loom-query/          # 任意: N テーブル JOIN のデータ構造（実行器は骨子）
   └─ loom-cli/            # デモ: core+redb を通す put/get の end-to-end 疎通
```

### 動かす

```bash
cd loomdb
cargo run -p loom-cli
# put   : u1/o100 amount=1200
# get   : u1/o100 -> Some({"amount": N(Number("1200")), ...})
```

## ステータス

**TDD（テスト先行・Red→Green→Refactor）で実装中**（docs/05-test-standard.md）。

| 部位 | 状態 |
|---|---|
| 順序保存キーエンコード（spec §2.3・S/N/B・複合キー） | ✅ 実装済み＋property test（順序単調性・round-trip・0x00/0xFF 耐性） |
| StorageEngine 契約スイート（rollback・MVCC スナップショット・走査） | ✅ `loom-testkit` — in-memory fake と redb アダプタの**双方に同一適用** |
| テーブル操作（§4.1 create/describe/list/delete・meta 永続化・名前検証） | ✅ DynamoDB 同様「作成してから名前で参照」する API 形 |
| 式言語 — Condition/Filter（§5.2/§5.5・手書き再帰下降＋純関数評価器） | ✅ 比較(S/N/B)・BETWEEN/IN・AND/OR/NOT・入れ子パス・#/:プレースホルダ・全関数 |
| 条件付き書込 — `put_item(condition)` / `delete_item(condition)` | ✅ attribute_not_exists イディオム・楽観ロック・失敗時ロールバックをテストで保証 |
| UpdateExpression（§5.3）＋ `update_item` | ✅ SET（+/-・if_not_exists・list_append）・REMOVE・ADD（原子カウンタ・10進厳密演算）・upsert・ALL_NEW・キー属性変更の禁止 |
| `get_item` | ✅ fake での単体テスト＋redb 経由 e2e |
| KeyCondition（§5.1）・Query/Scan・二次索引維持（§7）・JOIN 実行器（§10.3）・集合型 SS/NS/BS（ADD 集合和・DELETE） | ⏳ 次の TDD サイクル |

## ライセンス

MIT または Apache-2.0 のデュアルライセンス（利用者がいずれかを選択）。
[LICENSE-MIT](LICENSE-MIT) / [LICENSE-APACHE](LICENSE-APACHE) を参照。
