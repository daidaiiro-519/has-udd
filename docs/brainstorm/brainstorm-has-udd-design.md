# 設計ブレインストーミング: has-udd ドメインモデル設計

**目的:** has-udd システム自体のドメインモデルを設計し、engine Skills（harness-*-engine）・Orchestrator の実装根拠を固める
**モード:** アイデア発散 → 設計収束
**前提:** コンセプトブレスト（`brainstorm-has-udd-concept.md`）で確定した方針に基づく

---

## なぜドメインモデルから始めるか

engine Skills（harness-*-engine）と Orchestrator を設計するには has-udd 自体のドメインモデルが先に必要。ドメインコア = JSON Schema（DATA であり Python コードではない。schema = ドメインモデル）。

| 設計対象 | 依存するドメイン知識 |
|---|---|
| schema（JSON Schema, ドメインモデル） | has-udd の集約・値オブジェクトを型として表現する |
| harness-query-engine | has-udd のドメインデータ構造をナビゲートする |
| harness-render-engine | has-udd の集約状態をドキュメント・コードに変換する |
| Orchestrator（オーケストレーション） | has-udd のドメインイベント・状態遷移が根拠になる |

つまり **ドメインモデル（schema）→ engine Skills → Orchestrator → Role（職種）** の順序が正しい。

> **語彙（最新合意）**: 「Job Agent」→ **Role**（職種: PO/SM/Dev/QA/Backend/Frontend…）。「Job」は **作業実行を表す1レコード（audit-engine トレース内のリードモデル）** に限定し、独立集約ではない。実行状態は document.json の `status`（SSOT）が表現する。subagent は Role を体現する agentKind として維持。engine 認識は Orchestrator のみ（Role / Custom Skill は engine を呼ばない）。

---

## has-udd 自体のサブドメイン（確認）

コンセプトブレストで確定済みの分類を再確認する。

| サブドメイン | 分類 | 備考 |
|---|---|---|
| Usecase-Driven仕様管理 | 🔴 中核 | has-uddの差別化の核心 |
| Agentic Scrumオーケストレーション | 🔴 中核 | Orchestrator の領域 |
| Skill/Knowledge DI基盤 | ⚪ 補完 | engine Skills（harness-*-engine）の領域 |
| Harness実行基盤 | ⚪ 補完 | 既存ツールを活用 |
| スクラムセレモニー定義 | 🔵 一般 | スクラムガイドをKnowledge注入 |

---

## 論点 1: has-udd の集約（Aggregate）は何か？ ✅ CLOSED

### 合意決定

**集約は documentType ごとに分ける（Skill集約 / Spec集約 / Knowledge集約 / Agent集約 / Coding集約 の5集約）。Job 集約は持たない。**

> **集約は5つ**: Skill / Spec / Knowledge / Agent / Coding。**Job 集約は廃止**。実行の状態は document.json の `status`（SSOT）が表現し、Job は has-udd が強制する不変条件を持たない（= 集約ではなくリードモデル）。実行の可観測性（失敗・I/O・委譲チェーン）が必要になった場合は harness-audit-engine のトレース（リードモデル）が担う。「Job」は作業実行を指す語彙としては維持するが、独立集約としては持たない。

```
ドメインコア = JSON Schema（DATA・OSS チームが管理）
└── src/has_udd/domain/model/{SchemaName}.json
      AgentSchema / SkillSchema / UsecaseSpecSchema / DomainModelSpecSchema / CodingSchema
      schema = ドメインモデル（class）。document.json = instance。
      schema はパッケージ内に存在し、ユーザープロジェクトには配布しない。
      engine は importlib.resources / schema_repository 経由で解決する。
      → .has-udd/ に schemas/ は存在しない。

has-udd ランタイムが管理する集約（documentType ごとに分離・SOURCE は document.json）

├── Agent 集約 ← AgentSchema
│   ├── documentId / schemaRef / status / content
│   ├── agentKind: "orchestrator" | "subagent"（技術・構造軸。Role は subagent が体現するドメイン概念）
│   ├── Orchestrator: engine routing 知識・engine（Secondary Adapter）を呼ぶ唯一の責務・knowledgeRefs を機械解決
│   └── subagent = Role（職種: PO / SM / Dev / QA / Backend / Frontend…）を体現: Custom Skills を呼ぶ・engine を直接呼ばない
│         Role は roleKind（職種 identity・値軸）＋ Persona（レンズ）＋ skillRefs（担当 Custom Skill）＋ knowledgeRefs（不変 knowledge）を持つ
│         ※ agentKind = 構造ディスクリミネーター（if/then/else 分岐）／ roleKind = subagent 内の職種 identity（値軸・構造は不変）
│         ※ 1 Role = 1 subagent document（Role 集約は作らない・同一実体の2レンズ）
│
├── Skill 集約 ← SkillSchema
│   ├── documentId / schemaRef / status / content
│   ├── skillKind: "engine" | "custom"
│   ├── engine Skill（OSS 提供・Secondary Adapter）
│   │     harness-query-engine / harness-render-engine / harness-knowledge-engine
│   │     harness-scaffold-engine / harness-audit-engine
│   │     ※ harness-template-engine（テンプレート/コーディング）は CodingSchema 依存のため Phase 3 に延期
│   └── Custom Skill（ユーザー作成・Application Layer の手順定義。Role が実行する）
│
├── Spec 集約 ← UsecaseSpecSchema or DomainModelSpecSchema
│   ├── documentId / schemaRef / status / content
│   ├── UsecaseSpec: CREATED（リファインメント段階）→ VALIDATED（詳細設計完了）
│   │     refs.pbiRef で外部 PBI を参照（PBI は Jira / Linear で管理・スコープ外）
│   │     受け入れ条件・ドメインイベント・テストシナリオを持つ
│   └── DomainModelSpec: subdomain の集約・業務ルール・ユニットテスト仕様
│         ※ SBI は外部 Sprint Planning ツールで管理（has-udd スコープ外）
│
├── Knowledge 集約 ← KnowledgeSchema（独立集約）
│   ├── documentId / schemaRef / status / content
│   ├── ドメイン知識・業務ルール・Scrum 規約・手順を document.json として管理
│   ├── harness-knowledge-engine が動的クエリで集約（Facade パターン）
│   └── ユーザーは Knowledge document を作るだけでよい
│
└── Coding 集約 ← CodingSchema
        ├── documentId / schemaRef / status / content
        ├── CodingTemplate: 実装ファイル向けテンプレート
        └── TestTemplate: テストファイル向けテンプレート
              ※ UsecaseSpec（VALIDATED）→ harness-render-engine → DocComment 値注入 → コードファイル

※ Job 集約は廃止。実行状態は各 document の status（SSOT）が表現する。
  実行トレース（失敗・I/O・委譲チェーン）が必要なら harness-audit-engine のリードモデルが担う。

業務サービス（ステートレス）
└── schema_integrity（Domain Service）
      単一集約の不変条件では検証できない「集約をまたぐルール」を担う。
      （document-validate オペレーションは harness-scaffold-engine が持つ）

      主な検証責務:
      ・document.json 生成時  → schemaRef の schema が存在するか
                              → documentType と schemaRef の組み合わせは正しいか
      ・document.json 参照時  → 参照先が VALIDATED 状態か・documentType が正しいか
      ・Coding 生成時         → 参照する UsecaseSpec が VALIDATED 状態か
      ・Knowledge 参照時      → 対象 Knowledge が存在するか（harness-knowledge-engine が動的解決）

外部参照
└── Sprint → SprintId を保持するだけ（Scrum Team が管理）
```

**決定理由:**
- documentType ごとに業務ロジック・status lifecycle・スキーマが異なる → 集約の境界は documentType で引く
- フォルダ・schema・lifecycle が独立しているなら集約も分けるのが DDD 原則に沿う
- Job は has-udd が強制する不変条件を持たない（状態機械を has-udd が制御しない）→ 集約ではなくリードモデル。独立集約として持たず廃止。実行状態は document status が表現する
- PBI / SBI は外部ツールで管理。has-udd は外部参照（refs.pbiRef）のみ保持

**次のアクション:** 論点2（ドメインイベント）を集約設計ベースで設計する

---

## 論点 2: イベントとワークフローの設計

**前提の更新:** has-udd はチャットベース CLI（Claude Code と同様のターミナル操作）として動作する。
重厚な EDA インフラは不要。イベントは2層で表現される。

### AI 初期見解

**イベントの2層構造:**

```
外部イベント（UserInputEvent）
  ユーザーのチャット入力 = Command（DDD のコマンドに相当）
  Orchestrator がインテントを解釈して Role に作業を委譲する

内部イベント（Document 状態変化）
  document.json の status フィールドに記録される
  Orchestrator が次回の UserInputEvent 処理時に参照する

  ※ メッセージブローカー・イベントバスは不要
    状態はすべて document.json として永続化される
```

**Orchestrator の判断フロー:**

```
UserInputEvent（チャット入力）
  + harness-query-engine で関連 document.json を読み取る
  → インテント × ドキュメント状態 で委譲先 Role を決定
  → Role を起動（Claude / Copilot / Codex API 呼び出し）
  → 結果を document.json に書き込む（status 更新）
  → ユーザーに返答
```

**インテント × ドキュメント状態 → Role 起動の対応表（案）:**

| UserInputEvent（インテント） | 読む Document | 起動する Role | 生成する Document |
|---|---|---|---|
| Usecase 分解（リファインメント） | refs.pbiRef（外部 PBI を参照） | Dev Role | UsecaseSpec[]（CREATED） |
| Usecase 詳細設計（スプリント） | UsecaseSpec（CREATED） | Dev Role | UsecaseSpec（VALIDATED） |
| ドメインモデル整理 | UsecaseSpec[] | Dev Role | DomainModelSpec |
| 影響範囲検知 | UsecaseSpec / DomainModelSpec（変更対象） | Dev Role | 影響ドキュメント一覧 |
| 既存実装への JavaDoc 追加 | UsecaseSpec + 既存コード | Dev Role | CodingTemplate（修正モード） |
| Sprint 計画支援 | UsecaseSpec[] | SM Role | 外部 Sprint ツールへの出力（has-udd スコープ外） |
| スプリント状況確認 | UsecaseSpec[]（status 集計） | SM Role | — |
| レトロスペクティブ支援 | 完了 UsecaseSpec[] | SM Role | — |
| 実装支援 | UsecaseSpec | Dev Role | CodingTemplate |
| テスト支援 | UsecaseSpec + CodingTemplate | QA Role | TestTemplate |

**SM Role の has-udd 上での表現（すべて同じパターン）:**

```
harness-query-engine（読む）→ harness-knowledge-engine（knowledge 動的クエリ）→ Custom Skill（処理）→ Document 生成 → harness-render-engine（出力）
（engine は Orchestrator が呼ぶ。Role は Custom Skill を実行する）

スプリント状況確認の例:
  harness-query-engine: UsecaseSpec[] を JsonPath でステータス集計
  SM Role:              進捗をスクラムガイドの観点で解釈
  出力:                 状況スナップショット（document.json にはしない・後述）
```

**Hooks（AI 自動トリガー）:**

```
.has-udd/hooks/ に定義（.gitignore）。document.json の状態変化を監視し
Orchestrator に通知することでユーザー承認ベースの半自動連鎖を実現する。

発火条件の例:
  UsecaseSpec.status → "VALIDATED"
    → Orchestrator 提案: "実装支援を開始しますか？"

  Role 実行が失敗した（セッション内イベント・audit-engine トレース）
    → Orchestrator 提案: "実行が失敗しました。原因を分析しますか？"

設計方針:
  自動実行はしない（支援ツールの思想を維持）
  Orchestrator が提案 → ユーザーが承認 → 実行
```

**疎結合のインターフェース設計（DDD 公開ホストサービス＋共通言語に対応）:**

```
schema（JSON Schema, ドメインモデル）= Published Language
  → Agent 間・委譲間の通信規約
  → schema が変わらない限り内部実装は独立して変更できる

harness-query-engine（JsonPath）= 疎結合な情報アクセス
  → Document の内部構造を知らなくても必要な値だけ取得できる

delegation prompt（契約）= 疎結合な委譲
  → Role は「何を渡されたか」だけ知ればよく、上位の実装詳細を知る必要がない
  → 委譲チェーン・I/O の記録が必要なら harness-audit-engine のトレース（リードモデル）が担う
  → 再現性は schema + document status（SSOT）で担保する
```

**Document status（内部イベントの記録）:**

```
document.json
  status: "CREATED"    ← AI が生成した直後
         "VALIDATED"   ← schema_integrity（Domain Service）が通過
         "RENDERED"    ← docs/ フォルダへの出力が完了
         "SUPERSEDED"  ← より新しいバージョンに置き換えられた
```

### 合意決定

**決定:** 論点2の設計方針を以下の通り確定する。

**① document.json を作る価値があるアウトプットは Spec（UsecaseSpec / DomainModelSpec）のみ**

スプリント状況・レトロスペクティブの成果物は document.json にしない（次の作業の入力ゲートにならないため）。
SBI も has-udd のスコープ外（外部 Sprint Planning ツールで管理）。PBI は外部（Jira / Linear）で管理し、has-udd は refs.pbiRef で参照のみ保持する。
document.json にすべきかどうかの判断基準 = **次の作業（Role 実行）への入力ゲートになるか否か**。
UsecaseSpec は PBI（外部）を入力とし、スプリント内で CREATED → VALIDATED へ進化する。
リファインメントの「完了」は UsecaseSpec の状態遷移（CREATED → VALIDATED）として表現する。

**② Harness レトロ（自己改善ループ）は FeedbackReport として定義する**

```
スコープ: Spec作成 → Coding 完了までの一連ワークフローが終わった後に蓄積
観点:     ユーザーが改善できるレイヤーに限定する

Knowledge 改善観点:
  - AI が知識不足で誤った解釈をした箇所（Knowledge document の追加・更新で対応）
  - 業務ルールの変化で Knowledge が古くなった箇所

Custom Skills 改善観点:
  - Role 実行が繰り返し失敗したパターン
  - 同じ作業を何度も再実行した箇所（出力が期待値とずれ続けた）

含めない観点:
  - schema（ドメインモデル）の問題 → OSS チーム担当
  - AI モデルの推論精度 → モデル側の問題

活用: Hooks と連携し「前回の改善提案があります、反映しますか？」として次 Sprint 冒頭に提示
```

**③ エンドユーザーがカスタムできる範囲と OSS 提供側が固定する範囲の境界**

```
OSS 提供側が固定する（ユーザーは変更できない）:
  schema（JSON Schema, ドメインモデル）  定義・バージョン管理（パッケージ内に存在）
  engine Skills（harness-*-engine）      query / render / knowledge / scaffold / audit の基盤実装
  各集約（Skill/Spec/Knowledge/Agent/Coding）  スキーマ・status 遷移・schema_integrity
  ※ Job 集約は廃止（実行状態は document status が SSOT。トレースは audit-engine リードモデル）

エンドユーザーがカスタムできる（document.json の編集で対応）:
  Knowledge documents  ドメイン知識・業務ルール・Scrum 解釈の追加・更新（document.json を作るだけ）
  Custom Skills        実行手順・参照定義・プロンプトテンプレートの調整
  Agent document       Persona・担当 Skills の組み合わせ定義
  各種 Spec の content  schema に沿った範囲でのフィールド値
```

この境界こそが **Harness の本質**:
「AI に構造を推論させない」= OSS チームが構造を固定し、ユーザーは知識と手順だけを渡す。

**理由:**
- document.json のゲート判断基準を明確にしないと何でも Spec 化して形骸化する
- FeedbackReport の観点をユーザー改善可能レイヤーに絞ることで actionable になる
- カスタマイズ境界を設計段階で確定しないと OSS として配布できない

**次のアクション:** 論点3（ユビキタス言語の確定）へ進む

---

## 論点 3: ユビキタス言語の確定

has-udd の中で一貫して使う言葉を確定する。
schema（JSON Schema, ドメインモデル）のフィールド名がここから決まる。

### ユーザー見解

> Usecase は OSS 提供側。ユーザー拡張用語という概念自体が不要。
> has-udd が認識できる語彙はすべて has-udd 側で定義する。
> 変にユーザー独自の語彙が混ざるとノイズになり、解釈の違いが生まれる。

### AI 再考見解

**見解:** Layer 1 / 2 の区分を廃止。全用語を has-udd のユビキタス言語として一元定義する。

ユーザーが渡すのは**値（value）のみ**。用語（vocabulary）は変えられない。

**has-udd ユビキタス言語（全量）:**

**▼ ランタイム集約**

| 用語 | 定義 |
|---|---|
| **Document** | has-udd が管理するすべての成果物の単位。schemaRef・status・content を持つ。`.has-udd/documents/{type}/<id>.json` が SOURCE |
| **documentType** | Document の種別識別子。`Agent` / `Skill` / `Spec` / `Coding` / `Knowledge` の5値 |
| **Job** | 作業実行を表す1レコード（audit-engine トレース内のリードモデル）。集約ではない。状態機械は持たず、実行状態は対象 document.json の status が SSOT |
| **status** | Document の状態。`CREATED` / `VALIDATED` / `RENDERED` / `SUPERSEDED`（実行状態の SSOT） |
| **schemaRef** | Document が準拠する schema への参照 |
| **parentJobRef** | 委譲元 Job の ID。委譲チェーンを追跡する（audit-engine トレース内） |
| **skillsInvoked** | Job が実行時に使用した Skill の ID リスト（audit-engine トレース内） |

**▼ Agent / Skill**

| 用語 | 定義 |
|---|---|
| **Agent** | agentKind（orchestrator/subagent）で分かれる。subagent は roleKind + Persona + skillRefs + knowledgeRefs を持つ |
| **Orchestrator** | engine routing 知識を持ち Role へ作業を委譲・knowledge を機械解決する唯一の主体。engine を呼ぶ唯一の主体 |
| **subagent** | Role（職種: PO/SM/Dev/QA/Backend/Frontend…）を体現する Agent（agentKind: "subagent"）。1 Role = 1 subagent document |
| **Role** | subagent が体現する職種。Custom Skill を実行する。engine を呼ばない |
| **roleKind** | subagent の職種 identity（"backend"/"qa"/"po"/"sm"…）。OSS カタログ・拡張可能。構造ではなく値の軸 |
| **Persona** | Role の役割・観点（レンズ）を定義する自然言語の記述。Agent document の content 内 |
| **Skill** | Agent に注入される能力単位。skillKind で engine / custom に分かれる |
| **Custom Skill** | ユーザーが自プロジェクト用に定義する Skill。手順・参照定義・ドメイン固有の指示を持つ（skillKind: "custom"）。Role が実行する |
| **engine Skill** | OSS 提供の処理基盤 Skill（harness-*-engine）。query / render / knowledge / scaffold / audit。Orchestrator のみ呼ぶ（skillKind: "engine"） |
| **Knowledge** | ドメイン知識・業務ルール・Scrum 規約・手順を document.json として管理する知識文書。harness-knowledge-engine が動的クエリする独立した集約。ユーザーが作成する唯一の知識成果物 |

**▼ Document サブタイプ（Spec 系）**

| 用語 | 定義 |
|---|---|
| **Subdomain** | ビジネス業務領域の単位。docs/ フォルダ構成に対応し Spec の管理スコープになる |
| **PBI** | Product Backlog Item。外部ツール（Jira / Linear 等）で管理。has-udd は refs.pbiRef で外部参照のみ保持する |
| **Usecase** | 業務操作の単位。DDD の AppService メソッドレベル。has-udd が SSOT として管理する |
| **UsecaseSpec** | Usecase を document.json として表現したもの。CREATED（リファインメント段階・粗い）→ VALIDATED（スプリント内・詳細設計完了）。受け入れ条件・ドメインイベント・テストシナリオを持つ |
| **SBI** | Sprint Backlog Item。外部 Sprint Planning ツールで管理。has-udd のスコープ外 |
| **DomainModelSpec** | Subdomain の集約・業務ルール・ユニットテスト仕様を document.json として表現したもの |
| **Sprint** | 1〜2週間の開発サイクル。has-udd は SprintId として外部参照のみ保持する |

**▼ Document サブタイプ（Coding 系）**

| 用語 | 定義 |
|---|---|
| **CodingTemplate** | 実装ファイル向けテンプレート。UsecaseSpec から DocComment 値を注入して生成される |
| **TestTemplate** | テストファイル向けテンプレート。UsecaseSpec のテストシナリオをもとに生成される |

**▼ engine Skills（harness-*-engine, OSS 固定）**

| 用語 | 定義 |
|---|---|
| **schema（JSON Schema, ドメインモデル）** | Agent 間・委譲間の通信規約。Published Language。パッケージ内に存在し配布しない |
| **harness-query-engine** | JsonPath で document.json から必要な値を取得する疎結合アクセス基盤 |
| **harness-render-engine** | document.json を MD / HTML / ソースコード（DocComment 付き）に変換し deploy する出力基盤 |
| **harness-knowledge-engine** | Knowledge document を動的クエリする Facade |
| **harness-scaffold-engine** | schema から空の document.json を生成（_index は持たない・動的計算）。document-validate オペレーションも担う |
| **harness-audit-engine** | I/O トレース ＋ Interface 契約適合性（旧 contract-engine をリネーム）。Job リードモデルを保持 |
| **Hooks** | document.json の status 変化を監視し Orchestrator に提案をトリガーする仕組み（`.has-udd/hooks/`） |
| **FeedbackReport** | Workflow 完了後に蓄積する自己改善レポート。Knowledge document / Custom Skills 改善提案を持つ |

※ harness-template-engine（テンプレート/コーディング）は CodingSchema 依存のため Phase 3 に延期。

**▼ 命名規約**

```
has-udd ユビキタス言語: PascalCase（Document / Job / UsecaseSpec）
JSON フィールド名:      camelCase（documentType / schemaRef / parentJobRef / skillsInvoked）
engine Skill 名:        kebab-case（harness-query-engine 等）
ユーザーが渡すもの:     値のみ（Subdomain の名前文字列 / PBI のテキスト / Usecase の説明）
                        用語（フィールド名・型名）はユーザーが変更できない
```

### 合意決定 ✅ CLOSED

**決定:** has-udd のユビキタス言語はすべて OSS 側が定義する。ユーザー拡張用語という概念は持たない。ユーザーが渡すのは値のみ。用語・フィールド名は変更不可。

**確定用語一覧（全量）:**

| カテゴリ | 用語 |
|---|---|
| 集約 | documentType / status / schemaRef（Skill/Spec/Knowledge/Agent/Coding の5集約）※ Job 集約は廃止 |
| 実行トレース（リードモデル） | Job / jobId / parentJobRef / skillsInvoked（audit-engine の trace・集約ではない） |
| Agent / Skill | Agent / agentKind / Orchestrator / subagent / Role / roleKind / Persona / skillRefs / knowledgeRefs / Skill / Custom Skill / engine Skill |
| Knowledge | Knowledge / KnowledgeContent |
| Spec 系 | Subdomain / PBI / Usecase / UsecaseSpec / SBI / DomainModelSpec / Sprint |
| Coding 系 | CodingTemplate / TestTemplate |
| engine Skills | harness-query-engine / harness-render-engine / harness-knowledge-engine / harness-scaffold-engine / harness-audit-engine |
| schema / 仕組み | schema（JSON Schema, ドメインモデル）/ Hooks / FeedbackReport |

**命名規約:**
- has-udd 用語: PascalCase
- JSON フィールド名: camelCase
- ユーザーが渡すもの: 値のみ（Subdomain の名前文字列・PBI のテキスト等）

**理由:** 語彙がユーザーごとに揺れると AI の解釈にノイズが生まれる。Harness の本質は「AI に構造を推論させない」ことであり、語彙の固定はその前提条件。

**次のアクション:** 論点4（engine Skills 設計）へ進む。schema（ドメインモデル）の構造定義から着手する。

---

## 論点 4: engine Skills の設計

has-udd の基盤となる engine Skills（harness-*-engine）の具体的なインターフェースを設計する。
engine Skills 間には明確な依存関係があるため、設計順序を先に確定する。

### AI 初期見解

**engine Skills の依存関係:**

```
schema（JSON Schema, ドメインモデル）
  ↓ 全 document.json の構造を定義する。ここが確定しないと他は何も設計できない
  ↓
harness-query-engine                 harness-render-engine
  document.json を読む基盤               document.json を出力する基盤
  全 Agent の Input 側を担う             全 Agent の Output 側を担う
  ↓                                    ↓
  ← すべての成果物の I/O がここを通る →
  ↓
Hooks
  status 変化を検知し Orchestrator に通知する
  ↓
FeedbackReport（Hooks の終端）
  Workflow 完了後に audit-engine の Job 実行記録を集計し生成する
```

**設計順序: schema → harness-query-engine / harness-render-engine（同時）→ Hooks → FeedbackReport**

---

**① schema（JSON Schema, ドメインモデル）— 全 engine Skills の土台**

```
すべての document.json はいずれかの schema に準拠しなければならない。
schema のバージョン管理は schemaRef で行う（例: "UsecaseSpecSchema/v1"）。
schema はパッケージ内（src/has_udd/domain/model/）に存在し、ユーザーには配布しない。
harness-scaffold-engine が schema から空の document.json を生成する。

共通エンベロープ（全 documentType 共通）:
{
  "documentId":   string,          // UUID
  "documentType": "Agent" | "Skill" | "Spec" | "Coding" | "Knowledge",
  "schemaRef":    string,          // 例: "UsecaseSpecSchema/v1"
  "status":       "CREATED" | "VALIDATED" | "RENDERED" | "SUPERSEDED",
  "content":      object,          // documentType ごとに異なる本体
  "createdAt":    string,
  "updatedAt":    string
}

content の構造は schemaRef が指す個別 schema で定義される。
エンベロープは変えない。content だけが documentType ごとに異なる。
```

| schema | documentType | 主なフィールド（content 内） |
|---|---|---|
| AgentSchema | Agent | agentKind（構造軸）/ roleKind（subagent の職種・値軸）/ persona / skillRefs[] / knowledgeRefs[] |
| SkillSchema | Skill | skillKind（"engine" / "custom"）/ purpose / invocationSpec（engine のみ） |
| UsecaseSpecSchema | Spec | subdomainId / refs.pbiRef（外部参照）/ actor / intent / primaryAggregate / steps[] / domainEvents[] / testScenarios[] |
| DomainModelSpecSchema | Spec | subdomainId / aggregates[] / businessRules[] / unitTestSpecs[] |
| CodingSchema（CodingTemplate） | Coding | usecaseSpecRef / templateKind / docCommentFields[] |
| CodingSchema（TestTemplate） | Coding | usecaseSpecRef / testScenarioRefs[] |

---

**② harness-query-engine — 全成果物の Input 基盤**

```
すべての Agent が document.json を読む唯一の経路（Orchestrator が呼ぶ）。
Document の内部構造を呼び出し元が直接知らなくていい設計にする。

インターフェース（案）:
  find(jsonPath, { schemaRef })
    例: find("$.content.acceptanceCriteria[*]",
             { schemaRef: "UsecaseSpecSchema/v1" })

  findByStatus(status, { schemaRef })
    例: findByStatus("VALIDATED", { schemaRef: "UsecaseSpecSchema/v1" })

  findByRef(fieldName, value)
    例: findByRef("sprintId", "sprint-03")

設計方針:
  - schemaRef を必ず指定する → 型安全。異なる documentType を混在させない
  - 結果は常に JSON（AI が次の処理でそのまま使える）
  - document.json を動的に集約する（_index.json は廃止）
```

---

**③ harness-render-engine — 全成果物の Output 基盤**

```
すべての Agent が成果物を出力する唯一の経路（Orchestrator が呼ぶ）。
出力先パスは schemaRef から自動決定（ユーザーが指定しない）。
render は canonical（.has-udd/）に書き込む。別フォーマットツールへの deploy は transform（Phase 6）。

出力先の規約（OSS 固定）:
  UsecaseSpec     → .has-udd/specs/{id}.html（human 向け RENDERED）
  DomainModelSpec → .has-udd/specs/{id}.html
  Knowledge       → .has-udd/knowledge/{id}.html
  Skill           → .has-udd/skills/{name}/SKILL.md（tool 認識・RENDERED）
  Agent           → .has-udd/agents/{name}.md（tool 認識・RENDERED）
  CodingTemplate  → コードファイル（DocComment 注入済み・Phase 3）
  TestTemplate    → テストファイル（Phase 3）

インターフェース（案）:
  render(documentId)
    → document.json を読み、schemaRef から出力先・テンプレートを決定して書き出す

設計方針:
  - render 後に document.json の status を RENDERED に更新する
  - テンプレートは OSS が提供するが、プロジェクトルートに上書きファイルがあれば優先する
  - docCommentFields は UsecaseSpec の対応フィールドから自動マッピングして注入する
```

---

**④ Hooks — status 変化を監視**

```
document.json の status を定期チェックし、
変化を検知したら Orchestrator に通知する（`.has-udd/hooks/`）。

発火条件:
  UsecaseSpec.status → VALIDATED  : "実装支援（CodingTemplate 生成）を開始しますか？"
  Job → FAILED（audit トレース）   : "実行が失敗しました。原因を分析しますか？"
  Workflow 完了                    : "FeedbackReport を生成しますか？"

動作モデル:
  Hooks（status 検知）→ Orchestrator に通知 → ユーザーに提案（テキスト）
  → 承認 → 通常の UserInputEvent と同じパスで次の作業を起動
  → 拒否 → 何もしない

設計方針:
  - 自動実行は絶対にしない（支援ツールの思想を維持）
  - 提案は1つずつ（複数同時はノイズ）
```

---

**⑤ FeedbackReport — Hooks の終端・自己改善ループの出力**

```
Workflow 完了の Hooks をトリガーに harness-audit-engine の Job 実行記録を集計する。

集計対象:
  - Job → FAILED のパターン（Custom Skills 改善候補）
  - 同一 Job の再起動回数（出力が期待値とずれ続けた箇所）
  - harness-knowledge-engine が Knowledge を参照できなかった箇所（Knowledge document 不足の候補）

出力: FeedbackReport
  → Hooks が次 Sprint 冒頭に「前回の改善提案があります、反映しますか？」と提示
```

### ユーザー見解

> schema は土台ではない。harness-query-engine / harness-render-engine の方が土台。

### AI 再考見解

**見解:** 「依存関係の論理順」と「実装の優先順位」を混同していた。修正する。

```
依存関係（論理）:
  schema → harness-query-engine / harness-render-engine
  （schema が決まっていないと正しく読み書きできない）

実装優先順位（実際に作る順）:
  harness-query-engine / harness-render-engine が先
  → has-udd の実行基盤。これがなければ何も動かない
  → 実装する中で「どういう構造の document.json が扱いやすいか」が見えてくる
  → schema はその要件を逆算して確定する
```

**schema の正しい位置づけ:**

schema は「ドメインモデル（DATA）」であって実行コンポーネントではない。
query/render engine が動いて初めて「schema はこの形が必要だ」という要件が確定する。
schema は実装の**起点**ではなく**実装から逆算して固まるもの**。

**修正後の実装優先順位:**

```
① harness-query-engine / harness-render-engine（実行基盤。has-udd の I/O そのもの）
    ↓ これを作ることで document.json の必要な構造が見えてくる
② schema（ドメインモデルの確定。query/render engine の要件から逆算）
    ↓
③ Hooks（status 変化を監視する）
    ↓
④ FeedbackReport（Hooks の終端。audit-engine で Job 記録を集計）
```

### 合意決定

**決定:** engine Skills の実装優先順位は harness-query-engine / harness-render-engine → schema → Hooks → FeedbackReport。

schema は論理的な依存の起点だが、実装の起点は query/render engine。この2つが has-udd の真の基盤であり、全 Agent の I/O がここを通る。

**理由:** schema は実行しないドメインモデル（DATA）。query/render engine を実装して初めて「どんな schema が必要か」の要件が確定する。実装順序を論理依存順にすると、使われない schema を先に作り込む無駄が生まれる。

**次のアクション:** 論点5（Orchestrator 設計）へ進む。または query/render engine の具体的なインターフェース設計を深掘りする。

---

## 技術スタック確定 ✅ CLOSED

### 実装言語・配布

| 項目 | 選定 | 理由 |
|---|---|---|
| 言語 | **Python** | MCP 公式 SDK あり・jsonschema 充実・開発速度が速い |
| 配布 | **`uvx has-udd`** | npx 相当。インストール不要で実行可能 |

### ライブラリ

| 用途 | ライブラリ |
|---|---|
| MCP サーバー | `fastmcp`（Anthropic 公式 MCP Python SDK） |
| JSON Schema バリデーション | `jsonschema` |
| JsonPath クエリ | `jsonpath-ng` |
| CLI / init wizard | `typer` + `rich` |

### 動作モード

**モード1: Skills 配置モード（MCP なし）**
```
has-udd init
  → .has-udd/ を生成（document.json・rendered Skills/Agents を展開）
      ※ schema はパッケージ内に残す（.has-udd/schemas/ は作らない）
  → 同フォーマットツールへは init で symlink
      .claude/skills/<name>/SKILL.md ← .has-udd/skills/<name>/SKILL.md
      .claude/agents/<name>.md       ← .has-udd/agents/<name>.md
      （別フォーマットツールへは transform deploy・Phase 6）
  → CLAUDE.md / AGENTS.md（Primary Port）を生成（engine routing は持たない）

Claude Code / Kiro が Skills を認識し AI が直接 document.json を操作する
（has-udd バイナリは init 時のみ使用）
```

**モード2: MCP サーバーモード**
```
has-udd serve
  → fastmcp で MCP サーバーとして起動
  → Claude Code / Kiro が MCP 経由で以下を呼び出す
      query_document   document.json のクエリ
      validate_document JSON Schema バリデーション
      render_document  MD / HTML レンダリング
```

### has-udd CLI コマンド体系

```
has-udd init      プロジェクト初期化（document.json・rendered 展開・symlink・MCP 登録。schema は配布しない）
has-udd serve     MCP サーバー起動
has-udd validate  document.json を手動バリデーション
has-udd render    手動レンダリング実行
```

---

---

## アーキテクチャ設計原則（確定）

> **TODO:** このセクションの内容は将来 Knowledge document（`knowledge-has-udd-architecture`）としてまとめる。

### ヘキサゴナルアーキテクチャとの対応

has-udd のアーキテクチャはヘキサゴナルアーキテクチャに完全に対応する。これは偶然ではなく、**Harness 原則とヘキサゴナルアーキテクチャが同じ問いへの答えだから**。

```
ヘキサゴナル: 外部システムが変わっても Application Core を守る
Harness:      AI ツールが変わっても文書構造・ワークフローを守る
```

```
Primary Adapter:   Claude Code / Kiro / Codex（AI ツール本体）
    ↓ 読む
Primary Port:      CLAUDE.md / AGENTS.md
                   ← has-udd init が生成する薄いエントリポイント
                   ← 「Orchestrator を起動せよ」の指示のみ
                   ← engine routing を持たない（持つと Primary→Secondary 違反）
    ↓ 起動
─────────────────────────────────────────────────────────────
Application Core
  Application Layer:
    Orchestrator ← AgentSchema document.json（CLAUDE.md とは別物）
      ← engine routing の知識・engine（Secondary Adapter）を呼ぶ唯一の責務
      ↓ subagent に委譲
    subagent（agentKind: "subagent"）← AgentSchema document.json
      ← skillRefs = Custom Skills のみ（engine は直接呼ばない）
      ↓ Custom Skills を呼ぶ
    Custom Skills ← 「何をするか」の定義のみ・インフラ（engine）を意識しない
  Domain Layer:
    Skill集約 / Spec集約 / Knowledge集約 / Agent集約 / Coding集約（Job集約は廃止）
    domain core = JSON Schema（DATA）= src/has_udd/domain/model/
─────────────────────────────────────────────────────────────
Secondary Adapter: engine Skills（harness-*-engine）の実装（Python / MCP）
                   harness-query / render / knowledge / scaffold / audit -engine
```

> **2ヘキサゴンの視点**: engine = エージェントシステム視点では Secondary Adapter、ソースコード視点では Application use case。
>
> **ヘキサゴナルなパッケージ構成（src/has_udd/）**:
> `domain/model`（schema）, `domain/ports`（document_repository / schema_repository）, `domain/services`（schema_integrity）, `application`（engine use cases）, `adapters/inbound`（cli=typer / mcp=fastmcp）, `adapters/outbound`（fs / jinja / jsonschema）, `shared`（result / tags）。

---

### CLAUDE.md と Orchestrator の責務分け（確定）

混同しやすいが役割は明確に異なる。

| | CLAUDE.md（Primary Port） | Orchestrator（Application Core） |
|---|---|---|
| 性質 | 静的・薄いエントリポイント | 動的・AI ツールが担う役割（AgentSchema document.json） |
| 責務 | 「Orchestrator を起動せよ」の指示のみ・engine routing を持たない | ユーザーインテントの解釈・engine routing・subagent への委譲 |
| 生成元 | has-udd init が生成 | AI が schema を読んで値を書く |
| 実行者 | なし（読まれるだけ） | AI ツール自身がこの役割を担う |
| タイミング | セッション開始時に一度読まれる | ユーザー入力のたびに動く |

**動作フロー:**

```
ユーザー入力（UserInputEvent）
  ↓
AI ツールが CLAUDE.md（Primary Port）を読む
  ↓「Orchestrator として振る舞え」の指示
  ↓
AI が Orchestrator として Application Core で動く
  ← AgentSchema document.json に従って振る舞う
  ↓ インテントを解釈 → engine routing → subagent へ委譲
  ↓ engine Skills（Secondary Adapter）で document.json を読み書き
```

CLAUDE.md は「起動の引き金」。Orchestrator は「起動後の Application Core としての振る舞い」。CLAUDE.md が engine routing を持つと Primary Port が Secondary を直接呼ぶアーキテクチャ違反になるため、routing の知識は Orchestrator が持つ。

---

### Application Core の2層構造

```
Application Core
│
├── Application Layer（ユースケース・オーケストレーション）
│   ├── Orchestrator ← AgentSchema document.json
│   │     ユーザーのインテントを解釈して subagent へ委譲する = Application Service
│   │     engine routing の知識を持つ → engine（Secondary Adapter）を呼ぶ唯一の主体
│   │
│   ├── subagent（agentKind: "subagent"）← AgentSchema document.json = Role の実体
│   │     roleKind（PO / SM / Dev / QA / Backend / Frontend…）＋ Persona ＋ skillRefs ＋ knowledgeRefs
│   │     skillRefs = Custom Skills のみ（engine を直接呼ばない）
│   │
│   └── Custom Skills（skillKind: "custom"）
│         「PBI から UsecaseSpec を作る手順」など
│         「何をするか」の定義のみ・インフラを意識しない
│         （Application Layer が Domain Layer を呼ぶ自然な依存方向）
│
└── Domain Layer（ドメインモデル・業務ルール）
    ├── domain core = JSON Schema（DATA）― src/has_udd/domain/model/（配布しない）
    ├── Skill 集約 ― SkillSchema
    ├── Spec 集約 ― UsecaseSpecSchema / DomainModelSpecSchema
    ├── Knowledge 集約 ― 独立した知識集約
    ├── Agent 集約 ― AgentSchema
    ├── Coding 集約 ― CodingSchema
    ├── schema_integrity（Domain Service）
    └── Knowledge documents（ユーザーが作成・Knowledge 集約に属する）
          Scrum のルール・DDD の概念・業務ルール = ドメイン知識の具現化
          通常のシステムではコードの不変条件として書かれる部分を AI が読める形で外部化
          ※ クエリ基盤（SKILL.md）は harness-knowledge-engine（Secondary Adapter）が提供
```

**Knowledge documents が Domain Layer に属する理由:**

Knowledge documents は「何が正しいか」の判断基準を持つ（ドメインルール）。Custom Skills は「何をどの順で行うか」の手順を持つ（手順実装）。クエリ基盤（harness-knowledge-engine）は Secondary Adapter だが、Knowledge の中身はドメイン層の知識として Domain Layer に属する。

**schema（JSON Schema）との関係:**

Knowledge documents は AI の生成を導くソフトガイド。schema は出力の構造を検証するハード制約。両者は補完ではなく独立した役割を担う。

---

### has-udd 固有の特徴（AI が Runtime である前提のアーキテクチャ）

```
通常のシステム: ドメイン知識 = コードの不変条件（機械的強制）
has-udd:       ドメイン知識 = Knowledge documents（AI が読む知識ファイル）
```

AI が Runtime であることを前提に、Domain Layer の知識を AI が読める形で外部化している。これにより：

- ドメイン知識の更新がコード変更なしに行える（Knowledge document の編集のみ）
- ユーザーが Knowledge documents を追加して自社ドメインを注入できる
- OSS 提供者がコアの知識を保守し、ユーザーがカスタマイズする境界が明確になる
- Claude Code が Kiro になっても Codex になっても Core は変わらない（Primary Port が吸収）

これが Harness Architecture の本質的な差分であり、「AI に構造を推論させない」原則の構造的な根拠でもある。

---

## UsecaseSpec 設計原則（確定）

### ユースケースの切り方

**1 Actor + 1 Intent + 1 Aggregate = 1 UsecaseSpec**

| 種別 | 定義 | 例 |
|---|---|---|
| Command | 1集約を生成・更新する操作 | `uc-order-place`（Order 集約を新規作成） |
| Query | 1つの問いへの回答 | `uc-order-query-status`（Order 集約を読む） |

**切り方の判断軸:** 「この操作が失敗したとき、どこまでロールバックするか？」= 1トランザクション境界 = 1UsecaseSpec

```
1つの PBI → 複数の UsecaseSpec に分解されることが通常

例: PBI「顧客が商品を注文できる」
  → uc-cart-add-item    （Cart 集約・更新）
  → uc-order-place      （Order 集約・新規作成）
  → uc-order-cancel     （Order 集約・更新）
  → uc-order-query-status（Order 集約・読み取り）
```

### UsecaseSpec のライフサイクル

```
PBI（外部ツール: Jira / Linear）
  ↓ DDD 観点での設計具体化（この行為が DDD 分析）

UsecaseSpec（CREATED）← リファインメント段階
  refs.pbiRef → 外部 PBI を参照
  Actor / Intent / 対象集約 / 受け入れ条件（粗い）
  ← この粒度で集約が特定できれば見積もり・SBI 分解が可能

  ↓ スプリント内で AI + 人間が詳細化

UsecaseSpec（VALIDATED）← 詳細設計完了
  詳細フロー・ビジネスルール・ドメインイベント・テストシナリオ
  ← AI が実装の入力として使う SSOT

  ↓ AI が harness-render-engine で実装（Coding は Phase 3）

RENDERED（実装・ドキュメント出力完了）
```

### UDD 原則（TDD に対応する Usecase-Driven の規律）

```
TDD: テスト → 実装。実装変更前にテストを確認。
UDD: Spec  → 実装。実装変更前に必ず紐づく UsecaseSpec を確認。
```

**陳腐化防止の2分類:**

| 変化の種類 | Spec への影響 | 対応 |
|---|---|---|
| 「何をするか」が変わった（業務ルール・フロー変更） | Spec を先に更新してから実装 | 正当な仕様変更 |
| 「どうするか」が変わった（技術スタック・実装詳細） | Spec は変わらない | harness-render-engine が吸収 |

**CLAUDE.md への反映（Primary Port として構造的に強制）:**

```
実装を変更または追加する前に、必ず紐づく UsecaseSpec を確認すること。
Spec に変更が必要な場合は Spec を先に更新し、VALIDATED 状態にしてから実装に着手すること。
```

<!-- 論点5: Orchestrator 設計（委譲ロジック・インテント解釈）へ続く -->
