# LoomDB — プロジェクトメモリ

ゲートウェイ向け極小・組込ローカル NoSQL。DynamoDB のデータモデル＋API を redb 上に再現し、
JOIN で差別化する。詳細は README.md と docs/00〜05 を参照。

## 確定した意思決定（ユーザー承認済み）

- **API の書き味 = DynamoDB そのまま**（式言語・ConditionExpression・#names/:values）。
  **ORM 風 facade 案は撤回済み**（再提案しない）。
- **多言語対応の本命 = 「LoomDB というライブラリ」**:
  `npm install loomdb` / `pip install loomdb` で import して使う組込形
  （better-sqlite3 的・サーバ不要・DB はファイル1個）。
  実装: `loom-node`（napi-rs）/ `loom-py`（PyO3）— inbound adapter・コア非混入。
- **ユーザーが最も魅力を感じている体験 = drop-in 互換**:
  「接続先を変えるだけで、DynamoDB や Mongo のクライアントのコードがそのまま動く」。
  - DynamoDB ワイヤ層（spec §12）はこの体験を提供する（DynamoDB Local と同じ立ち位置・
    公式 AWS SDK が endpoint 差し替えで動く）→ 任意層として維持・実装する価値が高い。
  - **Mongo ワイヤ互換は将来アイデア**（FerretDB 前例あり。ただし BSON プロトコル＋
    データモデル写像で大工事のため約束しない。要望が再燃したら feasibility から）。
  - **バックエンド切替（将来アイデア・有望）**: LoomDB ライブラリの接続先を
    ローカルファイル ↔ 本物の DynamoDB で切替可能にする（outbound adapter を1枚追加）。
    「LoomDB で書いたコードが DynamoDB でも動く」＝ユーザーの理想体験の完成形。
    パリティ操作は 1:1 写像・JOIN はクライアント側で複数 query に分解して実行・
    バッチは 25 件分割・GSI は結果整合に低下、が既知のトレードオフ。
    **バインディング設計時に backend を差替可能な形にしておく**と後で安く実現できる。
- ライセンス: **MIT OR Apache-2.0** デュアル。
- ドキュメントは日本語で進め、**公開前に英訳**（README → docs の順）。
- 公開時の名前衝突: crates.io `loom` は既存（並行性テスタ）→ `loomdb-core` 等に改名。
  npm / PyPI の `loomdb` も公開時に空き確認。
- JOIN は N テーブル多段（left-deep・inner/left・エイリアス必須）— spec §10。
- バッチ/transact 操作数は無制限・GSI 常に強整合・後付け GSI（バックフィル）が差別化。

## 開発順序（現在地: コア完成・73 テスト green）

1. **JOIN 実行器**（spec §10.3・差別化の本丸）← 次
2. `loom-node`（napi-rs → npm "loomdb"）
3. `loom-py`（PyO3 → PyPI "loomdb"）
4. ワイヤ層（任意・drop-in 互換の入口）

## 開発プロセス（must）

- **TDD 必須**: Red（テスト先行）→ Green → Refactor。テストなしの実装コミット禁止。
  docs/05-test-standard.md 参照。コミットメッセージに赤→緑の経緯を記録。
- 品質ゲート: `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` /
  `cargo fmt --all --check` をコミット前に必ず通す。
- テスト・usecase には `@spec 01-spec.md#X.Y` アンカーを付ける（docs/04-coding-standard.md）。
- OSS 公開品質を目指す（fuzz・契約テスト・CI は docs/05 に規定）。

## 構造メモ

- この `loomdb/` ディレクトリは**自己完結**しており、`git subtree split --prefix=loomdb`
  でそのまま独立リポジトリに切り出せる（has-udd 本体とは独立のプロダクト）。
- ワークスペース: crates/loom-core（domain/application/ports）・loom-redb・loom-query・
  loom-testkit（契約テスト＋in-memory fake・publish しない）・loom-cli。
