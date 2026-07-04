# セッション引き継ぎ資料（2026-07-04時点）

このドキュメントは、**新しいセッション（別環境・リモートコントロール環境等）がこれまでの
作業経緯を全て引き継いで作業を再開できるようにする**ためのブリーフィングです。
最初にこのドキュメントを読めば、会話の全履歴を読まなくても続きから作業できることを目指します。

**作業ブランチ:** `impl/bootstrap-core-loop`（このドキュメント時点の最新コミット: `5885816`）

> コード変更の詳細な競合解決基準は `HANDOFF.md`（repo root）を参照。
> 本ドキュメントは**このセッション全体で何が起きたか・次に何をすべきか**を扱う。

---

## 1. リポジトリの全体像

`has-udd`は3つの独立した性質のディレクトリ/文書群から成る:

| 場所 | 何か | 状態 |
|---|---|---|
| `waffle/` | document.jsonスキーマ駆動エンジン「Waffle」（旧`has_udd`パッケージ）。自己完結・`loomdb/`と同じ位置付け | 実装済み・TDDで進行中 |
| `loomdb/` | ゲートウェイ向け組込NoSQL「LoomDB」。自己完結・独立プロダクト | 実装済み（JOIN実行器まで完了）・テスト拡充計画あり |
| `.has-udd/documents/` | has-udd（agent system）自身が管理するdocument.json（`specs/`・`skills/`・`coding/`） | Waffleが検証・query・renderする対象 |
| `.claude/skills/` | Claude Code用にレンダリングされたSkill定義 | Waffleのrender結果がdeployされる先 |
| `docs/brainstorm/` | ブレインストーミング文書（`brainstorm-*.md`）と、その後継の設計文書（`design-*.md`） | 本セッションで3本＋PoC計画1本を作成 |

**has-udd = Harness Agentic Scrum Usecase-Driven-Development の略**。
HAS（Harness Agentic Scrum＝agent system側）とUDD（Usecase-Driven-Development＝
Waffle engineが支える開発手法）の合成語だったことが本セッション中に判明し、
これが後述のOSS分離の判断根拠になった。

---

## 2. 本セッションで走った4本の作業ストリーム

### ストリームA: LoomDBのテスト計画（本セッション冒頭）

- ユーザーの懸念（「実運用で使えるレベルのテスト・確認がまだできていない」）を受けて、
  `loomdb/docs/06-test-plan.md`を新規作成。
- 内容: 機能面シナリオ拡張・非機能面4領域（fuzz・障害注入・CI・性能/サイズ回帰）の
  実行計画。実行順序: CI→機能面→障害注入→性能/サイズ→fuzz。
- **状態: 計画のみ。実行は未着手**（本セッションでは着手していない）。

### ストリームB: LoomDB×has-udd文書DB連携

- `docs/brainstorm/brainstorm-loomdb-has-udd-document-db.md` — 全5論点合意済み:
  1. 既存document.jsonをそのままLoomDBに入れるべきか？ → 入れない。派生的な再構築可能コピーは可
  2. 「雑多な文書を独自スキーマへ抽出しLoomDBで管理する」アプローチ → 採用
  3. 変換プロセスの再現性担保 → 経路A（AI新規執筆）と経路B（既存文書変換）を区別。経路Bのみ7ステップ手順
  4. x-prompt-queryを意味検証ゲートに転用 → 採用（経路A・B共通の第二ゲート）
  5. 変換のLLM呼び出し方式 → MCP+セッション内推論（API直接呼び出しはしない）・テキストベース共通インターフェース
- `docs/brainstorm/design-loomdb-document-conversion-poc.md` — 上記を踏まえたPoC計画（Phase1〜4）:
  - Phase1: テーブル設計＋投入・round-trip確認
  - Phase2: JOIN参照整合性チェック（正例＋負例）
  - Phase3: MCPツール設計（`get_conversion_target`/`validate_document`/`save_converted_document`）
  - Phase4: 意味検証ゲートの手動トライアル
  - 対象データ: `agg-document`・`sd-validation`・`uc-validate-document`等の実文書＋意図的に壊れた参照の合成データ
- **状態: 計画のみ。実装は未着手**。次にやるなら計画書§9の3点（PoC専用ディレクトリ作成可否・
  Phase3試作の可否・意味検証失敗ケースの合成データ作成者）をユーザーに確認してから着手。

### ストリームC: ハーネスのイベントログ解析デバッグツール

- `docs/brainstorm/brainstorm-context-observability.md` — 全4論点合意済み:
  1. 「シミュレータ」ではなく「事後解析ツール」として設計。ただしcompaction限定でなく、
     Skill起動・Hook発火・Subagent起動を含む**イベントログ全般**を扱う
  2. debug toolはhas-udd/LoomDBに構造的依存を持たない独立スキーマ（「イベントログ集約」）として設計。
     横断分析（JOIN）は利用者側の任意選択
  3. （論点2に統合）可視化UI（TUI or ローカルHTTPサーバ）は独立OSSの一部として実装時に選択
  4. 自己改善ループ（フィードバック収集→CLAUDE.md/Skill改善提案）は「検出→提示」までを自動化し、
     反映は人間承認を挟む
  - アーキテクチャ方針: この観測ツールとWaffleは、どちらも独立OSSとして提供し、has-uddは両方を
    使う「利用者」の一つという位置付け
- **状態: ブレストのみ。実装は一切着手していない**。

### ストリームD: has-uddのOSS分離（engine / agent system）→ 実際にコード変更

- `docs/brainstorm/brainstorm-has-udd-oss-separation.md` — 全5論点合意済み＋追記:
  1. 境界の事実確認: `src/has_udd/`（engine）はコード化済み、agent systemは`.claude/`層の
     規約のみ。これは開発フェーズ（engine作り込み中）の反映であり欠陥ではない
  2. 分離順序: engineを先行（現在進行中の作業の延長）
  3. 分離形式: `loomdb/`と同じ自己完結ディレクトリ（実行済み）
  4. agent systemの汎用性: Waffleの内部実装には非依存（テキストベース共通インターフェースのおかげ）。
     ただしMCP/CLIというインターフェースの利用は継続
  5. 命名: engineを**Waffle**に改名（実行済み）
  - 追記: document.jsonのパス規約を`.has-udd/`から`.waffle/`に変更・Waffle自身のdocument
    （14件）をrepo root/waffle両方の重複コピーから`waffle/.waffle/documents/`への
    一元管理に整理（実行済み）
- **状態: 全て実行済み・コミット済み・テストgreen**。詳細は`HANDOFF.md`参照。

---

## 3. 現在の技術状態（確認済み）

```bash
cd waffle && uv run pytest -q      # 15 passed
cd waffle && uv run behave         # 6 features / 65 scenarios passed
cd .. && uv run --project waffle waffle validate --path .has-udd/documents/skills/analyze-domain-model.json  # 動作確認済み
```

- `git status` はclean（このドキュメント作成時点）。
- `impl/bootstrap-core-loop` ブランチはoriginと同期済み。

---

## 4. 開発上の規約・注意点

- **loomdb/**: TDD必須（Red→Green→Refactor）。品質ゲート
  `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` /
  `cargo fmt --all --check` をコミット前に必ず通す。詳細は`loomdb/CLAUDE.md`。
- **waffle/**: `uv run pytest`・`uv run behave`を変更のたびに実行しgreenを確認。
  詳細は`waffle/CLAUDE.md`。
- **has-udd本体（agent system側）**: `docs/brainstorm/brainstorm-*.md`（アイデア発散）→
  `docs/brainstorm/design-*.md`（具体計画）という2段階のドキュメント命名規約。
- **git運用**: `impl/bootstrap-core-loop`ブランチで作業し、コミット後は
  `git push origin impl/bootstrap-core-loop`。コミットメッセージは日本語で、
  変更の「なぜ」を書く。もう一つのブランチ`claude/github-connection-check-cfrr8h`は
  ユーザー指示により**使用しない**（「後者の方はいらない」との明示指示あり）。
- **Stop hook**: `~/.claude/stop-hook-git-check.sh`が、未コミット変更・未pushコミット・
  署名関連の警告を出す。署名（GPG/SSH鍵）は本環境固有の問題で対応不可（既知の問題として無視）。
  未コミット/未push分は都度コミット・プッシュする。

---

## 5. 次にやるべきこと（優先度は特に指定なし・ユーザーと相談）

1. **ストリームB（LoomDB×has-udd PoC）の実装着手** — `design-loomdb-document-conversion-poc.md`
   §9の3点をユーザーに確認してから、Phase1（テーブル設計・投入）から着手
2. **ストリームC（イベントログ観測ツール）の設計具体化** — まだ構想段階。MCPツール設計や
   具体的なデータモデル（イベントログ集約）を詰める余地がある
3. **ストリームA（LoomDBテスト計画）の実行** — CIパイプライン構築から着手が推奨順序
4. **Waffleのさらなる整備** — PyPI公開準備（未着手）・実際の`git subtree split`実行（未実行、
   自己完結ディレクトリ化までは完了）

## 6. 読むべきドキュメント一覧

- `HANDOFF.md`（repo root） — Waffle切り出し作業の技術的詳細・並行編集時の競合解決基準
- `docs/brainstorm/brainstorm-loomdb-has-udd-document-db.md`
- `docs/brainstorm/design-loomdb-document-conversion-poc.md`
- `docs/brainstorm/brainstorm-context-observability.md`
- `docs/brainstorm/brainstorm-has-udd-oss-separation.md`
- `docs/brainstorm/brainstorm-has-udd-concept.md`（has-uddの頭字語・DDDサブドメイン分類の出典）
- `loomdb/docs/06-test-plan.md`
- `waffle/CLAUDE.md` / `loomdb/CLAUDE.md` / `CLAUDE.md`（repo root）
