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

## ステータス

設計フェーズ。実装（`loom-core` ワークスペース）はこれから。

<!-- TODO: LICENSE を選定して追加する（未定） -->
