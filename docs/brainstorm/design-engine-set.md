# engine Skills セットの再導出（正しい切り出し）

## 目的

engine Skills を「per-documentType（spec / template…）」ではなく、**正しい責務単位**で切り出し直す。
集約・DDD・ヘキサゴナルの3観点から導出する。

## 背景（なぜ再導出か）

- 旧リスト: query / render / knowledge / contract / **spec** / **template**
- spec-engine の機能が「Spec 生成ガイド」と曖昧なまま放置されていた
- 分解すると、Spec の機械的工程は **scaffold / validate / render / query** という**汎用操作**に割れ、Spec 固有の機械的責務がない
- → per-documentType の切り出しが筋違い。**汎用操作（= 機械的責務）で切り出す**べき

---

## 論点 ES-1: 正しい engine セットは何か（3観点から導出）

### 観点A: document.json ライフサイクル（操作の導出）

```
Schema（ドメインモデル・パッケージ内）
  → [SCAFFOLD]  空の document.json を生成（const + 空値 + x-prompt-write ガイド。_index は持たない）   ← 機械的
  → AI が値を埋める（CREATED）                                         ← 推論（engine でない）
  → [VALIDATE]  schema で検証 → VALIDATED                              ← 機械的
  → [RENDER]    document.json → MD/HTML/コード + deploy                ← 機械的
  → [QUERY]     content を読む / index を動的計算（schema x-prompt-query × document blockType・随時） ← 機械的
横断:
  → [KNOWLEDGE] knowledge 集約への特化クエリ（Facade）                 ← 機械的
  → [TRACE]     I/O 実行トレース（リードモデル・可観測性）             ← 機械的
```

→ 機械的操作 = **scaffold / validate / query / render / knowledge / trace**。すべて**汎用**（documentType 非依存）。

### 観点B: DDD（repository / domain service / application service）

| 要素 | 対応 |
|---|---|
| Driven Port | `document_repository`（load/save document.json）/ `schema_repository`（schema 解決） |
| Domain Service | `schema_integrity`（集約横断の参照整合・状態前提の検証） |
| **Application Service（= engine）** | scaffold / validate / query / render / knowledge / trace（repo + domain service + outbound を編成） |

→ engine = Application Service（use case）。documentType ごとではなく**操作ごと**に切られる。

### 観点C: ヘキサゴナル（Secondary Port）

Orchestrator（エージェント Core）が必要とする Secondary Port:
- 作る（scaffold）/ 読む（query）/ 検証（validate）/ 出力（render）/ 知識（knowledge）/ 追跡（trace）

→ 各操作 = 1 Secondary Port = 1 engine。

### 3観点の収束 → 導出された engine セット ✅ ES-1 合意

| # | engine | 責務（汎用機械操作） | 旧リストとの対応 | 状態 |
|---|---|---|---|---|
| 1 | **query** | document.json を読む（scan / index_scan / query 等） | ✅ 既存 | ブレスト済 |
| 2 | **render** | document.json → MD/HTML/コード + deploy | ✅ 既存 | ブレスト済 |
| 3 | **knowledge** | knowledge 集約への特化クエリ（Facade・core 共有） | ✅ 既存 | ブレスト済 |
| 4 | **scaffold** | create/fill/validate（schema → 空 document.json・_index 持たない） | ← **spec-engine の正体** | ✅ ブレスト済 |
| — | ~~audit（旧 contract）~~ | I/O トレース + 契約検証 | 旧 contract-engine | **DROP（現コンセプトで不要・条件付き再導入）** |
| — | ~~template~~ | コード注入（DocComment） | ← template-engine | **Phase 3 まで延期** |

**確定: 実装する engine は query / render / knowledge / scaffold の4本。**
- spec-engine → scaffold に再構成（ES-1）
- **audit（旧 contract）は DROP**（現コンセプトでは実行状態 = document status で足り、audit は廃止 Job 集約の可観測性の再来。委譲はテキスト＝検証 N/A・engine は自検証。再導入条件: ワークフロー機構 or Phase 5 FeedbackReport が要求。詳細は design-engine-audit.md）
- template/coding は Phase 3（CodingSchema）まで延期（ES-2c）

---

## 論点 ES-2: 切り出し判断

| # | 論点 | 状態 |
|---|---|---|
| ES-2a | scaffold は独立 engine か | ✅ **CLOSED: 独立**（入力=schema/出力=空doc・render と別ライフサイクル両端・凝集が別。core は schema 走査を共有） |
| ES-2b | validate は audit-engine 内包か独立 engine か | ✅ **CLOSED: audit には入れない**（下記） |
| ES-2c | template（コード注入）は独立 engine か | ✅ **CLOSED: Phase 3 まで延期**（CodingSchema・spec↔code linkage が未設計。render 出力モード+coding 注入アダプタに畳める見込み） |
| ES-2d | knowledge は独立を維持するか | ✅ CLOSED: 維持（K-1 で Facade 合意） |
| ES-2e | contract-engine の名前は適切か | ✅ **CLOSED: `harness-audit-engine` に改名** |

### ES-2e 詳細（CLOSED）: contract → audit

- engine の責務 = I/O 記録（trace）+ Interface 契約適合の検証（verify）
- 「contract」は名詞（検証対象）で、query/render/scaffold/knowledge の動詞軸とズレる
- 「audit」= 記録 + 検証を1語で包む動詞・他 engine と軸が揃う
- 旧用語「ContractSkills（= JSON Schema = Published Language）」は廃止済み（古い Skills 分類体系の名残）。Published Language は DDD パターン名・Port はヘキサゴナル語であって、contract は一般語にすぎない
- → **harness-audit-engine** に確定

### ES-2b 詳細（CLOSED）: validation は3種類に分けて配置

| 種類 | 何を検証 | 置き場 |
|---|---|---|
| 入力 validation | 各 engine が自分の入力（operation/path/format） | 各 engine の guardrails（既存） |
| document conformance | document.json が schemaRef の schema に適合するか（CREATED→VALIDATED） | jsonschema ロジック = **shared core**（各 engine が内部利用）。明示操作は **scaffold engine に同居**（schema 適合の両端: create=誕生時 valid / validate=充填後検証） |
| contract(I/O) conformance | engine 間 I/O が Interface（契約）通りか | **audit-engine**（本来の仕事） |

- 「document validation を audit に集約」は**無意味なので撤回**（ロジックは結局 shared・各 engine が使う。contract(I/O) と関心が違う）
- **scaffold engine = { create（空骨格）, validate（適合検証） }**
- **audit engine = I/O トレース + Interface 適合（document validation を含まない）**

### ES-2a 詳細（CLOSED）

scaffold と render は文書ライフサイクルの両端:
```
schema ─[scaffold]→ 空 document.json ─(AI が値を埋める)→ document.json ─[render]→ 成果物
```
- 入力（schema vs document）・出力（空JSON vs 成果物）・タイミング（誕生 vs 出力後）・凝集が違う → 別 engine
- render は「document を入力に取る」契約。scaffold は document が無い段階で schema から作る → render の format 違いに畳めない
- Harness 原則: AI は構造を作らない（schema が固定）。空の正しい構造+prompt を機械生成する scaffold が必須
- core（schema 走査・$defs 解決）は shared/domain で共有

---

## 論点 ES-3: コードのタグ走査＋reconcile の engine 帰属（追加・CodingSchema ブレストから）

CodingSchema ブレストで、コードの `@spec`/`@stack` タグを **ripgrep で走査 → 機械で schema 適合 JSON 化 → Spec/tech-stack と reconcile**（逆引き・coverage・drift 検知）する機構が必要になった（PoC 技術検証済み・`/tmp/has-udd-poc/verify.py`）。

| # | 論点 | 状態 |
|---|---|---|
| ES-3a | この「コードのタグ走査＋reconcile」は **query 拡張**か **新 engine** か | 未 |
| ES-3b | query は document.json 対象だが、これは**ソースファイル対象**（grep）。query の責務を広げるか、別 engine（例: trace/reconcile）にするか | 未 |
| ES-3c | **audit-engine DROP（A-1）の再検討トリガになりうる**。reconcile = coverage/drift = 我々が「可観測性」として drop した領域の再来。保守ループ（design-maintenance-loop.md）/ Hooks（design-hooks.md）/ FeedbackReport（Phase5）が具体要求を出した今、再導入条件に当たる可能性 | 未 |

### AI の初期見解
- query は「document.json を schema 駆動で読む」engine。**ソース grep は対象データが異なる**ので、query に混ぜず別 engine（reconcile 専用・grep+正規表現+schema validate）が筋。汎用性も保てる
- これは audit DROP の再導入条件「Phase5 FeedbackReport が要求」に近い。**reconcile engine = 旧 audit の健全版**（実行トレースでなく、コード↔Spec の静的照合）として再設計する余地

### ユーザー見解

---

## 合意事項

（論点解決後に記録）

---

## 次のアクション

ES-1 / ES-2 解決後 → 各 engine のブレスト（scaffold / audit）→ 全 engine ブレスト完了 → 実装（template は Phase 3 まで延期）
