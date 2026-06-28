# has-udd ブレスト実行タスクリスト

**方針:** 実装における依存関係順で進める（使用時の依存関係とは別物）

---

## 完了済み（CLOSED）

### brainstorm-has-udd-design.md
- [x] 論点1: has-udd の集約（documentType ごとに5集約: Skill/Spec/Knowledge/Agent/Coding。Job 集約は廃止）
- [x] 論点2: イベントとワークフロー設計
- [x] 論点3: ユビキタス言語の確定
- [x] 論点4: engine Skills の設計（実装優先順位確定）
- [x] 技術スタック確定（Python / fastmcp / jsonschema / jsonpath-ng / typer）
- [x] UsecaseSpec 設計原則（1 Actor + 1 Intent + 1 Aggregate / UDD 原則）
- [x] アーキテクチャ設計原則（ヘキサゴナル対応 / CLAUDE.md = Primary Port）

### design-schema-and-engine-skills.md
- [x] 論点1: Schema 基本構造（x-prompt-write / x-render / x-render-order 等のアノテーション体系）
- [x] 論点2: harness-query-engine アクセスパターン
- [x] 論点3: harness-render-engine 出力パターン（x-render の md/html キー・docComment キー）
- [x] 論点4: harness-audit-engine 要件（I/O トレース・構造整合性・品質ゲートの3責務）
- [x] 論点5: Knowledge Schema（blockType 7種・x-render の html キーのみ）

---

## 残タスク（依存関係順）

### Phase 1 — Skill Schema 共通定義（起点・依存なし）

- [ ] **#1** Skill Schema 共通定義
  - engine / custom 共通の blockType 構成・フィールド定義
  - x-render アノテーション構造（md/html キー）
  - invocationSpec（engine 固有 optional フィールド）の扱い
  - **→ Phase 2 全タスクの前提**

---

### Phase 2 — Engine Skill document.json 具体定義（#1 完了後）

これらが確定して初めて「document.json に何を持たせるべきか」がわかる。

エンジン構成（5種）: harness-query-engine / harness-render-engine / harness-knowledge-engine / harness-scaffold-engine / harness-audit-engine。template / coding は Phase 3 へ延期。

- [ ] **#2-1** harness-query-engine Skill document.json 具体定義
  - クエリ基盤・JsonPath クエリの仕様

- [ ] **#2-2** harness-render-engine Skill document.json 具体定義
  - レンダリング基盤・x-render（md/html キー）処理仕様

- [ ] **#2-3** harness-audit-engine Skill document.json 具体定義
  - I/O トレース・構造整合性・品質ゲートの3責務の具体仕様
  - （旧 harness-contract-engine。"audit" に確定）

- [ ] **#2-4** harness-knowledge-engine Skill document.json 具体定義
  - Knowledge 動的探索仕様（旧 _index.json Facade は廃止・動的解決に変更）

- [ ] **#2-5** harness-scaffold-engine Skill document.json 具体定義
  - （旧 harness-spec-engine）Spec 体系定義（DDD 思想込み）・UsecaseSpec/DomainModelSpec 作成ガイド

---

### Phase 3 — 成果物 document.json の Schema 具体定義（Phase 2 全完了後）

- [ ] **#3-1** UsecaseSpec Schema 具体定義
  - actor / intent / primaryAggregate / steps[] / domainEvents[] / testScenarios[]
  - x-prompt-write・x-render アノテーション
  - **→ #3-4（Coding Schema）の前提**

- [ ] **#3-2** DomainModelSpec Schema 具体定義
  - subdomainId / aggregates[] / businessRules[] / unitTestSpecs[]
  - x-prompt-write・x-render アノテーション

- [ ] **#3-3** Agent Schema 具体定義
  - agentKind（構造軸: orchestrator/subagent）/ roleKind（subagent の職種・値軸）/ persona / skillRefs[] / knowledgeRefs[]
  - Job Agent → Role に呼称変更（Job 集約は廃止・"Job" は work-execution の read-model）
  - agentKind == "subagent" のとき roleKind 必須
  - skillRefs の型は Skill Schema（#1）に依存
  - **→ #6-2（Multi-tool Agents 互換）の前提**

- [ ] **#3-4** Coding Schema 具体定義（CodingTemplate / TestTemplate）（#3-1 完了後・Phase 3 へ延期）
  - templateKind / docCommentFields[] / targetFile
  - x-prompt-template アノテーション・Jinja2 エンジン・DocComment レンダラー（旧 harness-template-engine 相当）
  - usecaseSpecRef の型は UsecaseSpec Schema（#3-1）に依存

- [ ] **#3-5** custom Skill Schema 具体定義
  - engine との差分（invocationSpec なし等）
  - ユーザーが作成する Skill document.json の blockType 構成

---

### Phase 4 — HarnessAgent 設計（Phase 3 全完了後）

- [ ] **#4** HarnessAgent 設計
  - Orchestrator（agentKind=orchestrator）の Role 委譲ロジック・インテント解釈
  - インテント × ドキュメント状態 → work-execution（Job read-model）起動判断フロー
  - **→ #5-1（Hooks 設計）の前提**

---

### Phase 5 — Hooks / FeedbackReport（#4 完了後）

- [ ] **#5-1** Hooks 設計（#4 完了後）
  - document.json 状態変化の監視
  - HarnessAgent への通知・ユーザー承認ベースの半自動連鎖
  - **→ #5-2・#6-1（Multi-tool Hooks 互換）の前提**

- [ ] **#5-2** FeedbackReport 設計（#5-1 完了後）
  - Hooks 終端の自己改善ループ
  - Knowledge / CustomSkills 改善観点の集計
  - 次 Sprint 冒頭提示フロー

---

### Phase 6 — Multi-tool 互換設計

- [ ] **#6-1** Multi-tool 互換: Skills / Hooks / SSOT 構成 / rules/（**独立・いつでも可**）
  - 論点1: Skills 互換（シンボリックリンク3本）
  - 論点3: Hooks 互換（#5-1 完了後が望ましい）
  - 論点4: SSOT フォルダ構成確定
  - 論点5: rules/ の扱い（has-udd スコープ外・確認のみ）

- [ ] **#6-2** Multi-tool 互換: Agents 互換（#3-3 完了後）
  - 論点2: 各ツール向け変換生成（Kiro=JSON / Copilot=.agent.md / Codex=TOML）

---

## 依存関係サマリー

```
#1 Skill Schema 共通
  ├─→ #2-1 harness-query-engine
  ├─→ #2-2 harness-render-engine
  ├─→ #2-3 harness-audit-engine
  ├─→ #2-4 harness-knowledge-engine
  └─→ #2-5 harness-scaffold-engine
            ↓（Phase 2 全完了）
       ├─→ #3-1 UsecaseSpec Schema ──→ #3-4 Coding Schema（template engine 相当・延期）
       ├─→ #3-2 DomainModelSpec Schema
       ├─→ #3-3 Agent Schema ──────────────────────→ #6-2 Multi-tool Agents互換
       └─→ #3-5 custom Skill Schema
                 ↓（Phase 3 全完了）
                #4 HarnessAgent 設計
                 ↓
                #5-1 Hooks 設計 ──→ #5-2 FeedbackReport
                                └──→ #6-1 Multi-tool Hooks互換

#6-1（Skills/SSOT/rules/）← 独立・いつでも可
```
