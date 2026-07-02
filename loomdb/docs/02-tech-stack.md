# 技術スタック — LoomDB

「限りなく小さい」を最優先に、依存を厳選する。

## 1. 言語・ランタイム

| 項目 | 選択 | 理由 |
|---|---|---|
| 言語 | **Rust**（edition 2021・MSRV は固定して明記） | メモリ安全＋C 依存なしで小さい静的バイナリ＋式評価器を安全に書ける |
| 標準ライブラリ | `std` 前提（組込 Linux gateway） | `no_std` は不要・複雑化を避ける。将来必要なら alloc 境界を意識 |
| ターゲット | 例 `aarch64-unknown-linux-musl` / `armv7-unknown-linux-musleabihf` | musl 静的リンクで単一バイナリ・依存ゼロ配布 |

## 2. 中核依存（最小限）

| 能力 | クレート | 理由・代替 |
|---|---|---|
| **ストレージ（ACID KV）** | `redb` | pure Rust・ACID・単一ファイル・C 依存なし。**port で抽象化**し LMDB へ差替可能に |
| **直列化** | `serde` ＋ `rmp-serde`（MessagePack） | JSON より小さく速い。item value の格納形式 |
| **エラー** | `thiserror` | 定型エラー。マクロのみで実行時コスト小 |
| 数値（N 型・任意精度） | 自前の10進エンコード（or 軽量 `rust_decimal` を検討） | DynamoDB N は任意精度・順序保存が要る。依存を増やさないなら自前 |

## 3. 式言語

| 項目 | 選択 | 理由 |
|---|---|---|
| パーサ | **手書き再帰下降**（外部パーサ非依存） | `nom`/`chumsky` を足すとサイズ増。文法が有界なので手書きで十分・依存ゼロ |
| AST 評価 | 自前・純関数 | 副作用なし＝テスト容易 |

## 4. 任意層（feature flag で切離し）

| 層 | クレート | flag |
|---|---|---|
| ワイヤ互換サーバ | 極小 HTTP（`tiny_http` 等・要評価） | `wire` |
| 結合クエリ層（JOIN） | 追加依存なし（core の API・索引・read txn を利用） | 別 crate `loom-query`・feature `join`・コアに非混入 |
| Node.js バインディング | `napi-rs`（npm 配布・DocumentClient 風 API。JS number は f64 のため高精度 N は文字列/BigInt） | 別 crate `loom-node`・コアに非混入 |
| Python バインディング | `PyO3`（PyPI abi3 wheel・boto3 風 API） | 別 crate `loom-py`・コアに非混入 |
| CLI | 最小引数解析（`clap` は重いので `pico-args` 等を検討） | `cli` |

> **原則: コア（ライブラリ）にはワイヤ/CLI/query の依存を持ち込まない。** これらは別 crate か feature。gateway には必要な物だけ配布。

## 5. ビルド・サイズ最適化

```toml
[profile.release]
opt-level = "z"      # サイズ優先
lto = true
codegen-units = 1
panic = "abort"      # unwind テーブルを削減
strip = true
```
- 目標: コア単体バイナリ／ライブラリを **数百 KB〜1MB 台**に収める。
- `cargo bloat` / `cargo size` で継続監視（test-standard にサイズ回帰を含める）。

## 6. 非ドメイン能力（capability 一覧）

| capability | 実装 | 備考 |
|---|---|---|
| storage-engine | redb | port `StorageEngine` 経由 |
| serialization | rmp-serde | item value |
| clock（TTL 判定） | `std::time` を port `Clock` で抽象化 | テストで固定時刻に差替 |
| logging | 最小（feature `log`・既定オフ） | サイズ優先で既定は無効 |
| join（結合） | 追加依存なし（別 crate `loom-query`） | 読取専用・コア非混入 |

## 7. 依存を足す前のルール

- 反射的に足さない。①既存の std/redb/serde で代替不可か ②サイズ増（`cargo bloat`）に見合うか を必ず確認。
- 重量級（tokio・大型 HTTP・大型パーサ）はコアに禁止。ワイヤ層でも極小実装を優先。
