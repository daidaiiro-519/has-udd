# テスト規約 — nanodyn

実装方法＝ドメインモデル寄り（式・索引・トランザクション意味論が中核）→ **ピラミッド型**（単体厚め）。加えて DB 特有の**プロパティテスト**と**クラッシュ/ロールバック試験**を必須にする。

## テストの層

| 層 | 対象 | フレームワーク |
|---|---|---|
| 単体 | key_codec・式パーサ/評価・index 差分・属性型変換 | `cargo test` |
| プロパティ | 順序保存の単調性・式評価の性質・txn 原子性・移行 round-trip | `proptest` |
| 結合 | 操作 API を一時 redb ファイル上で（tempfile） | `cargo test` ＋ `tempfile` |
| 適合（conformance） | DynamoDB 挙動のサブセット一致（条件付き書込・Query 範囲・GSI 強整合） | 表駆動テスト |
| 障害 | write txn 中断→ロールバック確認・電源断シミュレーション | 専用ハーネス |
| サイズ回帰 | リリースバイナリ/ライブラリのサイズ上限 | `cargo size`/CI 閾値 |

## 必須プロパティ（proptest）

- **キーエンコード**: 任意の (a, b) について `a < b`（型内順序）⇔ `encode(a) < encode(b)`（辞書式）。round-trip `decode(encode(v)) == v`。
- **UpdateExpression**: 適用後の item が式のセマンティクスを満たす（SET/ADD/REMOVE/DELETE）。二重適用の非冪等/冪等性を明示。
- **トランザクション原子性**: `transact_write` の途中で1操作を失敗させると、**どの項目も索引も変更されていない**。
- **索引一貫性**: 任意の書込列の後、全 GSI/LSI が主データから導出した内容と一致。
- **移行 round-trip**: SQLite → 移行 → query で、元行と項目が一対一で一致。

## 障害・ロールバック試験

- write txn に mutation を積んで commit せず drop → 変更が残らないこと。
- condition 不成立の put/update/delete → 状態不変。
- transact_write の一部失敗 → 全体ロールバック＋理由コード。
- （可能なら）commit 直前/直後に強制終了を注入し、再オープンで一貫状態（redb の回復に委ねるが検証する）。

## 仕様との束ね（UDD 的）

- 各テストは対応する仕様節を参照（`@spec 01-spec.md#6` 等）。仕様変更時はテスト先行で赤にしてから実装。
- 適合テストは「DynamoDB でこう振る舞う」を1ケース=1振る舞いで固定（回帰防止）。

## カバレッジ方針

- 重点＝**式評価器・キーエンコード・トランザクション/索引維持**（バグの温床かつ中核）。ここは分岐網羅を目標。
- アダプター（redb/wire/cli）は結合テストで担保（薄いので単体は最小）。

## CI ゲート

- `cargo test` 緑・`clippy -D warnings`・`fmt --check`・サイズ閾値・proptest（固定 seed ＋一定回数）。
