# テスト計画 — 機能面・非機能面の拡張（TDD サイクル 20〜）

`docs/05-test-standard.md` が定める8層のうち、単体・プロパティ・契約・結合・適合は
厚く実装済み。本書はその**具体シナリオのカタログ**と、まだ手つかずの4層
（fuzz・障害・性能回帰・サイズ回帰）＋CI 基盤の**実行計画**を定める。

各シナリオは将来の TDD サイクルでそのままテスト関数名の元になる形で書く
（Red を書く → 落ちる → 実装/計測 → Green）。

## 0. 現状ベースライン（本書作成時点で実測）

| 項目 | 実測値 |
|---|---|
| ワークスペース全体のテストスイート数 | 25（core 15・bridge 2・node 1・query 2・redb 3・testkit 2） |
| 合計テスト数 | 138 |
| `loom-cli` release バイナリ | 972 KB |
| `loom-node` ネイティブ .so（release） | 1.2 MB |
| `loom-py` ネイティブ .so（release） | 1.3 MB |
| nightly toolchain | **未インストール**（stable のみ） |
| `cargo-fuzz` | **未インストール** |
| CI（GitHub Actions 等） | **存在しない**（全検証はこの開発コンテナ内のみ） |

README の「サブ MB〜1MB 台」は `loom-cli` では成立（972KB）が、これは実測に基づく
初回の閾値候補であって正式な回帰基準ではない（§4 参照）。

## 1. 機能面のシナリオ拡張

既存カバレッジの「正常系＋代表的な異常系」の外側にある、境界値・組み合わせ・規模の
拡張。既存テストファイルへの追記 or 新規ファイルとして刻む。

### 1.1 式言語の境界値（`loom-core/tests/expr_*` 拡張）
- N 型: 38 桁ちょうど（境界内）と 39 桁（境界外・ValidationError）、指数下限
  `1E-130` と上限 `9.9E+125` の境界、`-0` と `0` の等価、末尾ゼロの多いケース
  （`"1.000000000000000000000000000000000000"` が 1 桁として扱われるか）
- S 型: 空文字列・サロゲートペアを含む絵文字・NFC/NFD 正規化の異なる同一見た目文字列
  （UTF-8 バイト順比較なので正規化はしない、が明示テストで固定する）
- パス式: 深い入れ子（10段以上）・リスト×マップの混在ネスト・`#name`/`:value` の
  同時大量参照（プレースホルダ辞書が大きい場合の性能ではなく正しさ）
- BETWEEN/IN の境界（空リスト IN・BETWEEN の lo>hi 逆転）

### 1.2 GSI / JOIN の組み合わせ拡張（`loom-core/tests/indexes.rs`・`loom-query/tests/join.rs`）
- 同一テーブルに複数 GSI（3個以上）を張り、1回の update で複数索引が同時に
  付け外しされるケース
- JOIN 4段以上・各段で kind（inner/left）を混在させた組み合わせの全パターン
  （2^N 通りは網羅しないが、代表的な「途中で left → 以降 inner」等の遷移）
- 空テーブル同士の JOIN・root 0件・全 step 0件マッチ
- 自己結合を3段以上（自己結合の連鎖）

### 1.3 TTL の境界（`loom-core/tests/ttl.rs` 拡張）
- `ttl == now` ちょうど（既存でカバー済みか確認）・`now` が負値（epoch 前）
- `sweep_expired` の budget=0（no-op）・budget が実際の失効件数を超える場合

### 1.4 集合型（SS/NS/BS）の規模（`loom-core/tests/sets.rs` 拡張）
- 大きな集合（1000要素超）での ADD/DELETE の正しさと所要時間の目視
- NS で 38 桁級の要素を多数含む集合の正規化・重複除去

### 1.5 transact/batch の規模（`loom-core/tests/transact_batch.rs` 拡張）
- 既存の 150 ops 実証を 1,000 / 10,000 ops まで引き上げ、"無制限" の主張を
  現実的な規模で裏付ける（§4 の性能計測と合流できる）

### 1.6 クロス言語パリティ（新規 `loom-bridge` or 結合レベル）
- 同一シナリオ（put→get→query→update→delete の一連）を Node と Python の両方で
  実行し、**JSON として同じ結果**になることを保証するテスト（現状は各言語で別々の
  テストがあるだけで、「同じ入力に同じ出力」を横断で固定するテストが無い）

## 2. 非機能面 A: 障害注入（クラッシュ耐性）

**目的**: write txn の commit 前後で実プロセスを強制終了しても、再オープン時に
一貫状態（コミット済み or 未コミットのどちらか。中間状態はあり得ない）であることを
redb の保証に頼らず自前で検証する。

### 2.1 ハーネス設計
- 新規 crate 不要。`loom-core/tests/crash_injection.rs`（or 専用 `loom-redb/tests/`）
  に、**子プロセスを spawn** して特定の操作（例: 100件 put の途中）を行わせ、
  外部から `SIGKILL` を送るテストを書く。
- 子プロセス側は `loom-cli` に薄い専用サブコマンド（例: `loom crash-test <path>
  <n>`）を追加するか、テスト専用の小さいバイナリ crate
  （`loom-testkit` 配下に `bin/crash_worker.rs`）を用意する。
- 親プロセス（テスト本体）は: 子を起動 → 一定時間 or 特定の出力（stdout に
  "committed N" 等の目印）を待って `kill -9` → 子の終了を待つ → 同じファイルを
  再度開いて `stats()`/`get()` で一貫性を検証。

### 2.2 具体シナリオ
- **単発 put の commit 直前 kill**: item が存在しない（未コミット）ことを確認
- **単発 put の commit 直後 kill**（子プロセスが commit 完了後すぐ終了するタイミング
  を模す）: item が存在する（コミット済み）ことを確認
- **GSI 更新を伴う put の途中 kill**: 主データと索引が食い違わない
  （索引だけ書けて主データが無い、逆も無い）
- **transact_write（複数操作）の途中 kill**: 全操作が消えているか全操作が
  揃っているかのどちらか（部分適用が絶対に無いこと）
- **ランダムタイミング注入**: sleep 時間を 0〜数十 ms でランダム化して繰り返し
  実行し（例 20 回）、毎回一貫性が保たれることを見る（proptest ではなく単純ループ）

### 2.3 実現性メモ
- Linux コンテナ内でのプロセス kill・シグナルは問題なく使える（確認済み）。
- 「commit 直前/直後」を正確に狙うのは原理的に難しい（レース）。**タイミングを
  正確に制御する**より、**多数回のランダムタイミング注入で不変条件が破れないこと**
  を積み重ねる方針にする（test-standard の「(可能なら)」という書き方とも整合）。
- 依存追加は不要（`std::process::Command` のみ）。

## 3. 非機能面 B: fuzz

**目的**: 手書きパーサ・デコーダに任意バイト列を投げても panic せず、
不変条件（Err を返すか、正しい構造を返すかのどちらか）を破らないこと。

### 3.1 対象（fuzz target ごとに 1 バイナリ）
1. `expr::parse_condition` — 任意文字列
2. `expr::parse_update` — 任意文字列
3. `expr::parse_key_condition` — 任意文字列
4. `expr::parse_projection` — 任意文字列
5. `key_codec::decode_key` / `decode_first` — 任意バイト列（encode 済みでない
   壊れたキーを decode させる。「壊れた入力から復元しようとする」経路がある
   唯一の箇所なので優先度が高い）
6. `rmp_serde::from_slice::<Item>` — 任意バイト列（現状は自分で書いた値しか
   読まないので外部攻撃面は薄いが、将来ワイヤ層で外部入力になり得るため
   早めに固めておく価値はある）

### 3.2 実現性メモ（重要な制約）
- **本環境には nightly toolchain も `cargo-fuzz` も入っていない**（実測済み）。
  導入には `rustup toolchain install nightly` と `cargo install cargo-fuzz` が
  要り、後者はネットワーク経由のビルドになる。**この開発コンテナで完結できるか
  は未検証** — 着手時に最初に確認する。
- 導入できない/重い場合の代替: `proptest` の `prop_oneof!`/`Arbitrary` 相当で
  「ランダムな不正バイト列」を stable 環境の通常テストとして流す簡易 fuzz
  （cargo-fuzz ほど網羅的ではないが依存追加ゼロで今すぐ書ける）。
  test-standard の「夜間 CI」要件は cargo-fuzz 前提だが、まず簡易版で
  Red→Green を回し、本格導入は CI 基盤（§4 C）と一緒に評価する。

## 4. 非機能面 C: CI パイプライン

**目的**: これまでの検証が全部この単一コンテナ内で閉じている状態を解消し、
push/PR ごとにクリーンな環境で再現性を担保する。

### 4.1 構成案（`.github/workflows/loomdb-ci.yml`）
- トリガ: `loomdb/**` への push・PR（モノレポなので path フィルタ必須）
- ジョブ:
  1. `test` — `cargo test --workspace`
  2. `clippy` — `cargo clippy --workspace --all-targets -- -D warnings`
  3. `fmt` — `cargo fmt --all --check`
  4. `node` — `crates/loom-node` で `npm test`（build+typecheck+node --test）
  5. `python` — `crates/loom-py` で `maturin develop` 相当のビルド後 `python -m unittest`
- 将来ジョブ（本書の他セクションが Green になり次第追加）:
  6. `fuzz`（nightly cron・短時間 run）
  7. `bench`（性能回帰・§4-D）
  8. `crash`（障害注入・§2）

### 4.2 実行順序上の位置づけ
CI 自体は「TDD の Red/Green」には乗らないインフラ構築なので、他の3領域より
**先に**土台として組む。ここが無いと fuzz/crash/bench の各ジョブを追加する
先が無い。

## 5. 非機能面 D: 性能・サイズ回帰

### 5.1 性能ベンチ
- 依存方針（`docs/02-tech-stack.md`「依存を厳選」）に沿い、**`criterion` は
  追加しない**方向で検討する。まずは `std::time::Instant` ベースの簡易ハーネスを
  `loom-core/benches/`（`cargo bench` の素朴な形。nightly 不要な
  `#[test]` ベースの計測でも可）で書き、閾値は初回計測値を記録してから
  「±X% 以内」を回帰条件にする（test-standard の方針通り）。
  統計的厳密性が要る場面が出たら `criterion` を dev-dependency として再検討する。
- 計測対象:
  - `put_item`/`get_item`: 1万件投入後のランダム get の平均レイテンシ
  - `query`: 1万件パーティションからの範囲 query
  - `loom-query::execute`（JOIN）: 2テーブル×各1万件の等値結合
  - `transact_write`: 100/1000 ops のトランザクション所要時間

### 5.2 サイズ回帰
- `cargo build --release` 後の `loom-cli`・`libloom_node.so`・`libloom_py.so`
  のファイルサイズを CI で記録し、初回値（本書 §0 のベースライン: 972KB /
  1.2MB / 1.3MB）から一定割合（例 +20%）を超えたら fail。
- 閾値の具体的なパーセンテージは要決定（§6）。

## 6. 実行順序と未決定事項

### 実行順序（提案）
1. **CI パイプライン**（§4）— 土台。他の全領域の実行場所になる
2. **機能面シナリオ拡張**（§1）— 既存パターンの延長で最も着手しやすい
3. **障害注入**（§2）— 依存追加なしで書ける・DB として最も価値が高い非機能テスト
4. **性能・サイズ回帰**（§5）— ベースラインは本書で確保済みなので閾値だけ決めれば良い
5. **fuzz**（§3）— nightly/cargo-fuzz の導入可否を最初に確認してから着手

### ユーザーに確認が必要な点
- fuzz: nightly toolchain の導入を試すか、simplified proptest 版で当面代替するか
- 性能回帰の閾値（何%の劣化で CI を fail させるか）
- サイズ回帰の閾値（同上）
- CI の実行環境（GitHub Actions 前提でよいか。self-hosted runner で ARM
  ネイティブテストも要るか — README の gateway ターゲットが arm64/armv7 のため）
