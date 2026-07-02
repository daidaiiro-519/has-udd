# コーディング規約 — nanodyn

## 技術方式
ポートとアダプター（ヘキサゴナル）。domain は外部クレート（redb/serde/http）を知らない。

## レイヤーと依存方向

| レイヤー | 責務 | 依存してよい先 |
|---|---|---|
| domain | データモデル・キーエンコード・式・索引・エラー | なし |
| application | 各操作・トランザクション境界 | domain・ports |
| ports | StorageEngine・Clock | domain 型のみ |
| adapters | redb・wire・cli | application・ports |
| query（拡張） | 結合など読取専用クエリ（`nanodyn-query`） | application・ports |

**規則**
- 依存は内向きのみ。`nanodyn-core/domain` は `redb`/`serde`/`hyper` を import しない。
- 外部ライブラリはアダプター（`nanodyn-redb` 等）に閉じ込める。
- `println!`/`eprintln!` をライブラリに書かない（logging capability・既定オフ）。

## 概念 → 実現形

| 概念 | 規約 |
|---|---|
| value-object | `domain/` に不変 `struct`/`enum`。`Clone` は許容、内部可変は禁止 |
| domain-service | `domain/` にステートレス純関数（`key_codec`, `expr::eval`, `index::maintain`） |
| usecase | `application/usecases/{op}.rs`・入口は1関数・txn を張るのはここだけ |
| outbound-adapter | `StorageEngine` 実装は `nanodyn-redb` のみ。redb 型を外に漏らさない |

## エラー・結果

- 公開 API は `Result<T, DbError>`。`DbError` は `thiserror`。
- **ライブラリコードで `unwrap`/`expect`/`panic!` 禁止**（テスト・不変条件の内部検証を除く）。回復不能は `DbError` に写像。
- `transact_write` は失敗理由コード配列を返す（DynamoDB `TransactionCanceled` 準拠）。

## 安全性

- `#![forbid(unsafe_code)]` を原則。順序保存エンコード等で必要なら該当モジュールに限定し理由をコメント＋テストで固める。
- `clippy` を CI で deny（`-D warnings`）。

## 命名・レイアウト

- モジュール = スネークケース、型 = パスカルケース、関数 = スネークケース。
- 操作名は DynamoDB に合わせる（`put_item`, `query`, `transact_write`）。
- 1 usecase = 1 ファイル。

## 仕様との紐付け（アンカー）

- 各 usecase の docstring 先頭に **`@spec 01-spec.md#4.2`** の形で仕様節を指す（探索・重複防止のため）。
- 式評価は `@spec 01-spec.md#5` を指す。
- 実装本体は再生成保護のため境界コメントで囲う: `// nanodyn:impl-start` / `// nanodyn:impl-end`。

## サイズ規律

- 依存追加は PR で `cargo bloat --release` の差分を添付。理由なきサイズ増は却下。
- `panic = "abort"` 前提のコード（unwind に依存しない）。

## 決定ルール（守るべき不変）

- 主データと索引は**必ず同一 write txn** で更新（別 txn 禁止＝索引ズレ防止）。
- キーは必ず `key_codec` を通す（生バイト直書き禁止＝順序保証）。
- item value の直列化は rmp-serde に一本化（形式分岐禁止）。
