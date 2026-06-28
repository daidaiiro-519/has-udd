# harness-query-engine 設計ブレスト

## 目的

`harness-query-engine` の Skill document.json（SkillSchema/v1）を設計する。
engine の責務・I/O インターフェース・Steps・ガードレールを確定し、完全な document.json を作成する。

---

## 前提（確定事項）

- skillKind: `"engine"` → SkillSchema/v1 の EngineContent を使用
- 呼び出し元: Orchestrator（HarnessAgent）のみ
- 技術スタック: Python + `jsonpath-ng.ext`（フィルタ式対応のため標準 `jsonpath-ng` ではなく ext 版を使用）
- 2つの動作モード: Skills 配置モード / MCP サーバーモード

---

## 設計思想・参考にした仕組み

### DynamoDB の Query / Scan パターンを参考

クエリの設計は DynamoDB の Query パターンと Scan パターンの思想を参考にしている。

| DynamoDB | harness-query-engine |
|---|---|
| Scan（全件読む・重い） | Scan モード → ファイル全体を返す |
| Index Scan（index だけ読む・軽い） | Index Scan モード → ブロックの index（blockType + prompt）を返す。index は document.json に保存されず、読み取り時に schema の x-prompt-query から動的に算出する |
| Query（キーで直接取る） | Query モード → `content[blockKey]` を直接取得 |
| FilterExpression | JsonPath 拡張フィルタ（`?(@.required==true)` 等） |

**PK / SK の制約なし:** DynamoDB の Query は必ず PK を指定する必要があるが、has-udd では blockKey を指定すればどのブロックでも取れる。より柔軟なアクセスパターン。

**JsonPath は DynamoDB の FilterExpression より表現力が高い拡張クエリ:** ネスト・正規表現・存在チェック・ワイルドカードに対応。

### index の本来の役割（動的算出・非保存）

- 全コンテンツを読み込ませないためのルックアップインデックス
- index は document.json に保存しない。読み取り時に schema の x-prompt-query から動的に算出する（単一ソース）
- `index[key].prompt` が AI の意味的判断の根拠（schema の x-prompt-query が出どころ）
- prompt の質が AI の blockKey 選択精度を左右する → x-prompt-query の設計が重要
- document.json に index を保存すると schema との二重管理・同期ずれが生じる → 保存せず毎回算出する。`_index.json` ファイルを廃止したのと同じ原則を、ドキュメント単位の index にも適用する

### ディレクトリ Index Scan が `_index.json` ファイルを不要にする（重要）

**ディレクトリ全体の index（各 document.json から動的算出したもの）を動的集約できるため、事前生成の `_index.json` ファイルは不要になった。**

```
従来の設計（廃止）:
  has-udd が knowledge/_index.json を自動生成・更新管理
  → document 追加・変更のたびに同期が必要
  → 同期ずれリスクあり

新しい設計（採用）:
  has-udd query --index-scan .has-udd/knowledge/
  → 毎回動的に全 document.json の index（schema の x-prompt-query から算出）を集約して返す
  → 常に最新・同期ずれなし・管理コストゼロ
```

消えるもの:
- `knowledge/_index.json`（自動生成・維持コスト）
- document 追加・変更時に index ファイルを更新するロジック
- index ファイルと実ファイルの同期ずれリスク

### Skills モードと MCP モードの統一

**どちらのモードも Python スクリプトが実行する。AI が直接ファイルを読み解釈することはしない。**

| | Skills モード | MCP モード |
|---|---|---|
| 呼び出し方 | AI が InvocationSpec を読んで has-udd CLI を呼ぶ | AI が MCP ツールを呼ぶ |
| 実行者 | Python スクリプト（同じ実装） | Python スクリプト（同じ実装） |

AI が自分でファイルをパース・解釈した瞬間に Harness 原則（AI に構造を推論させない）が崩壊する。

---

## クエリモード（4種・確定）

| モード | スコープ | 用途 | jsonpath-ng |
|---|---|---|---|
| **Scan** | 単一ファイル | 全内容が必要なとき（document.json / SKILL.md / 通常ファイル） | 不要 |
| **Index Scan（ファイル）** | 単一 document.json | そのドキュメントに何があるか把握（index は schema の x-prompt-query から動的算出） | 不要 |
| **Index Scan（ディレクトリ）** | ディレクトリ内の全 document.json | どんなドキュメントが存在するか横断把握（各 index は動的算出） | 不要 |
| **Query** | 単一 document.json のブロック | 特定ブロックだけ取得・JsonPath で絞り込み | 必要（ext 版） |

### CLI イメージ

```bash
has-udd query --scan         <filePath>                        # Scan
has-udd query --index        <filePath>                        # Index Scan（ファイル）
has-udd query --index-scan   <dirPath>                         # Index Scan（ディレクトリ）
has-udd query --block <key>  <filePath>                        # Query（ブロック全体）
has-udd query --block <key>  --path "$.items[*].title" <path>  # Query + JsonPath
```

### クエリアルゴリズム（Index Scan → Query の2段階）

```
Stage 1: Index Scan（ファイルまたはディレクトリ）
──────────────────────────────────────────────────
  Python: index を動的に算出して返す（jsonpath-ng 不要）。
          index は document.json に保存されていないため、
          各ブロックの blockType から schema の x-prompt-query を引いて組み立てる:

    schema = schema_repository.load(doc["schemaRef"])   # パッケージ src/has_udd/domain/model/ から importlib.resources で解決
    index = {}
    for key, block in doc["content"].items():
        blockType = block["blockType"]
        prompt    = schema["$defs"][blockType + "Block"]["x-prompt-query"]
        index[key] = { "blockType": blockType, "prompt": prompt }
    return index

  単一ファイルの戻り値:
  { "<blockKey>": { "blockType": "...", "prompt": "..." }, ... }

  ディレクトリの戻り値:
  {
    "harness-render-engine": { "purpose": {...}, "steps": {...}, ... },
    "analyze-domain-model":  { "purpose": {...}, "steps": {...}, ... }
  }

    ↓ ここだけ AI の意味的判断
    prompt を読んで必要な blockKey / documentId を選択

Stage 2: Query
──────────────
  Python:
    block     = doc["content"][blockKey]
    schema    = schema_repository.load(doc["schemaRef"])
    prompt    = schema["$defs"][block["blockType"] + "Block"]["x-prompt-query"]  # x-prompt-query から動的算出
    value     = block
    if jsonpath:
        value = jsonpath_ng.ext.parse(jsonpath).find(value)

  戻り値:
  { "prompt": "...", "value": { ...ブロックの中身... } }
```

### query-engine は schema にアクセスする（prompt 算出のため）

index・prompt は document.json に保存されず、schema の x-prompt-query から動的に算出する。
そのため query-engine は読み取り時に schemaRef から schema を解決する必要がある。
schema はパッケージ `src/has_udd/domain/model/` に同梱され、`schema_repository`（`importlib.resources`）経由でロードする（`.has-udd` には配布されない）。

### blockKey が既知の場合は Stage 1 をスキップできる

---

## 実証済みクエリパターン（jsonpath-ng.ext）

| パターン | JsonPath 式 | 戻り値例 |
|---|---|---|
| ブロック全体 | `$.content.steps` | Steps ブロック全体のオブジェクト |
| 配列全要素 | `$.content.steps.items[*].title` | `["step1タイトル", ...]` |
| 先頭要素 | `$.content.steps.items[0].title` | `["step1タイトル"]` |
| スライス | `$.content.steps.items[0:2].title` | `["step1", "step2"]` |
| 値フィルタ | `$.input[?(@.required==true)].name` | `["documentPath"]` |
| 存在チェック | `$.items[?(@.children)].stepId` | `["step-1"]` |
| 正規表現 | `$.items[?(@.stepId=~"step-1.*")].title` | `["step-1 のタイトル"]` |
| 再帰 | `$..stepId` | 全階層の stepId 一覧 |
| SubStep ネスト | `$.items[*].children[*].title` | SubStep タイトル一覧 |

※ フィルタ式（`?(@...)`）は `jsonpath_ng.ext.parse` が必要。`jsonpath_ng.parse` では動かない。

---

## 実証済みシナリオ別戻り値

### S1: query_index(engine)
```json
{
  "title":          { "blockType": "Title",          "prompt": "このSkillのタイトルを持ちます。documentId がそのまま使われます。レンダリング時に h1 見出しになります。" },
  "purpose":        { "blockType": "Purpose",        "prompt": "このSkillの目的を1〜2文で持ちます。Skill選択・ルーティング判断・ドキュメント理解に使います。" },
  "role":           { "blockType": "Role",           "prompt": "このSkillの役割リストを持ちます。Skillの責務範囲の把握に使います。" },
  "interface":      { "blockType": "Interface",      "prompt": "このengine SkillのI/Oインターフェース定義を持ちます。Orchestratorがengineを呼び出す際の型・必須フラグの確認に使います。" },
  "invocationSpec": { "blockType": "InvocationSpec", "prompt": "このengine Skillの呼び出し仕様（Skills / MCP モード）を持ちます。Orchestratorが実行環境に応じた呼び出し方を選択するために使います。" },
  "steps":          { "blockType": "EngineSteps",   "prompt": "このSkillの実行手順を持ちます。Skillの処理内容の把握・デバッグ・進捗追跡に使います。" },
  "guardrails":     { "blockType": "Guardrails",     "prompt": "このSkillの実行制約を持ちます。SubAgentがSkillを実行する前に確認すべきガードレールの一覧です。" },
  "references":     { "blockType": "References",     "prompt": "このSkillが参照するリソースの一覧を持ちます。Skill実行時に読み込むべきファイルの特定に使います。" }
}
```

### S2: query_block(engine, "purpose")
```json
{
  "prompt": "このSkillの目的を1〜2文で持ちます。Skill選択・ルーティング判断・ドキュメント理解に使います。",
  "value": {
    "blockType": "Purpose",
    "title": "目的",
    "text": "document.json の x-render テンプレートをもとに Markdown・HTML 形式でレンダリングし、人間が読める形式の成果物を生成する。"
  }
}
```

### S3: query_block(engine, "interface")
```json
{
  "prompt": "このengine SkillのI/Oインターフェース定義を持ちます。Orchestratorがengineを呼び出す際の型・必須フラグの確認に使います。",
  "value": {
    "blockType": "Interface",
    "title": "インターフェース",
    "input": [
      { "name": "documentPath", "type": "string",  "required": true,  "description": "レンダリング対象の document.json のファイルパス" },
      { "name": "format",       "type": "string",  "required": false, "description": "出力形式。\"md\" | \"html\" | \"both\"（省略時は schema の x-render-target.formats に従う）" }
    ],
    "output": [
      { "name": "renderedPaths", "type": "string[]", "description": "生成されたファイルのパス一覧（出力先は schema の x-render-target.path 駆動）" },
      { "name": "status",        "type": "string",   "description": "\"success\" | \"error\"" }
    ]
  }
}
```

### S4: query_block(engine, "interface", "$.input[?(@.required==true)].name")
```json
{
  "prompt": "このengine SkillのI/Oインターフェース定義を持ちます。...",
  "value": ["documentPath"]
}
```

### S5: query_block(engine, "invocationSpec", "$.mcpMode")
```json
{
  "prompt": "このengine Skillの呼び出し仕様（Skills モード / MCP モード）を持ちます。...",
  "value": ["MCP ツール名: render_document。パラメータ: { documentPath: string, format?: string }。出力先は schema の x-render-target 駆動。has-udd serve 起動後に利用可能。"]
}
```

### S6: query_block(engine, "steps") — Steps 全体
```json
{
  "prompt": "このSkillの実行手順を持ちます。Skillの処理内容の把握・デバッグ・進捗追跡に使います。",
  "value": {
    "blockType": "EngineSteps",
    "title": "実行手順",
    "items": [
      {
        "stepId": "step-1",
        "title": "document.json と対応 Schema を取得する",
        "body": "documentPath で指定された document.json を読み込む。schemaRef フィールドから対応する Schema ファイルのパスを解決する。",
        "children": [
          { "stepId": "step-1-1", "title": "document.json を読み込む",   "body": "指定パスの document.json を読み込み、schemaRef を確認する。" },
          { "stepId": "step-1-2", "title": "Schema ファイルを解決する", "body": "schemaRef を schema_repository（importlib.resources）でパッケージ内 src/has_udd/domain/model/{schemaRef}.json として解決する。schema は .has-udd に配布されない。" }
        ]
      },
      { "stepId": "step-2", "title": "レンダリング順序とテンプレートを決定する",     "body": "Schema の各ブロック定義から x-render-order・x-render-level・x-render テンプレートを収集し、出力順のリストを組み立てる。" },
      { "stepId": "step-3", "title": "Jinja2 テンプレートを展開して出力を生成する", "body": "x-render-order 順に各ブロックを処理する。x-render-level に従って見出しを付与し、Jinja2 テンプレートにブロックデータを渡してレンダリングする。" },
      { "stepId": "step-4", "title": "生成コンテンツを出力先に書き込む",            "body": "format に応じて MD・HTML ファイルを schema の x-render-target.path（canonical）に書き込む。生成されたファイルパスを renderedPaths として返す。" }
    ]
  }
}
```

### S7: query_block(engine, "steps", "$.items[*].title")
```json
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": ["document.json と対応 Schema を取得する", "レンダリング順序とテンプレートを決定する", "Jinja2 テンプレートを展開して出力を生成する", "生成コンテンツを出力先に書き込む"]
}
```

### S8: query_block(engine, "steps", "$.items[?(@.children)].stepId")
```json
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": ["step-1"]
}
```

### S9: query_block(engine, "steps", "$.items[*].children[*].title")
```json
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": ["document.json を読み込む", "Schema ファイルを解決する"]
}
```

### S10: query_block(engine, "guardrails")
```json
{
  "prompt": "このSkillの実行制約を持ちます。SubAgentがSkillを実行する前に確認すべきガードレールの一覧です。",
  "value": {
    "blockType": "Guardrails",
    "title": "ガードレール",
    "items": [
      "document.json に schemaRef が存在しない場合はエラーを返し、処理を中断する",
      "Schema ファイルが見つからない場合はエラーを返し、処理を中断する",
      "x-render テンプレートが定義されていない blockType はスキップし、警告を出力する",
      "出力先ディレクトリが存在しない場合は自動作成する"
    ]
  }
}
```

### S11: query_block(custom, "processingTarget")
```json
{
  "prompt": "このcustom Skillの処理対象と成果物を持ちます。SubAgentがSkillの意味的な入出力を把握するために使います。",
  "value": {
    "blockType": "ProcessingTarget",
    "title": "処理対象と成果物",
    "target": "業務要件・会話・既存コードベース・DDD に関する知識。SubAgent がコンテキストとして渡した情報すべてを処理対象とする。",
    "artifact": "業務語彙リスト・集約候補リスト・値オブジェクト候補リスト・業務ルール一覧。DomainModelSpec（CREATED 状態）の content に記載できる形式で出力する。"
  }
}
```

### S12: query_block(custom, "guardrails", "$.items[*]")
```json
{
  "prompt": "このSkillの実行制約を持ちます。...",
  "value": [
    "集約境界の判定には必ず「失敗時のロールバック範囲」を問いとして使うこと。感覚的な境界引きは禁止",
    "「中核サブドメイン」の判定は慎重に行うこと。ほとんどの業務は一般・補完に分類される",
    "エンティティと値オブジェクトの区別を省略しないこと。後の実装設計に直接影響する",
    "ユビキタス言語に技術用語（テーブル・カラム・API 等）を混入しないこと"
  ]
}
```

---

## セマンティック操作 シナリオ別取得値（実証済み）

### Group 1: ドキュメントレベル

```json
// S1. scan(harness-render-engine.json)
{ "type": "raw", "content": "{\"documentId\": \"harness-render-engine\", ...（省略）" }
```

```json
// S2. get_meta(harness-render-engine.json)
{ "documentId": "harness-render-engine", "status": "DRAFT", "skillKind": "engine", "tags": ["engine:render"], "schemaRef": "SkillSchema/v1" }
```

```json
// S3. index_scan(harness-render-engine.json)
{
  "title":          { "blockType": "Title",          "prompt": "このSkillのタイトルを持ちます。documentId がそのまま使われます。レンダリング時に h1 見出しになります。" },
  "purpose":        { "blockType": "Purpose",        "prompt": "このSkillの目的を1〜2文で持ちます。Skill選択・ルーティング判断・ドキュメント理解に使います。" },
  "role":           { "blockType": "Role",           "prompt": "このSkillの役割リストを持ちます。Skillの責務範囲の把握に使います。" },
  "interface":      { "blockType": "Interface",      "prompt": "このengine SkillのI/Oインターフェース定義を持ちます。Orchestratorがengineを呼び出す際の型・必須フラグの確認に使います。" },
  "invocationSpec": { "blockType": "InvocationSpec", "prompt": "このengine Skillの呼び出し仕様（Skills モード / MCP モード）を持ちます。Orchestratorが実行環境に応じた呼び出し方を選択するために使います。" },
  "steps":          { "blockType": "EngineSteps",   "prompt": "このSkillの実行手順を持ちます。Skillの処理内容の把握・デバッグ・進捗追跡に使います。" },
  "guardrails":     { "blockType": "Guardrails",     "prompt": "このSkillの実行制約を持ちます。SubAgentがSkillを実行する前に確認すべきガードレールの一覧です。" },
  "references":     { "blockType": "References",     "prompt": "このSkillが参照するリソースの一覧を持ちます。Skill実行時に読み込むべきファイルの特定に使います。" }
}
```

```json
// S4. index_scan_dir('.has-udd/documents/skills/')  ← ディレクトリ横断集約
{
  "analyze-domain-model": {
    "title":            { "blockType": "Title",            "prompt": "このSkillのタイトルを持ちます。..." },
    "purpose":          { "blockType": "Purpose",          "prompt": "このSkillの目的を1〜2文で持ちます。..." },
    "role":             { "blockType": "Role",             "prompt": "このSkillの役割リストを持ちます。..." },
    "processingTarget": { "blockType": "ProcessingTarget", "prompt": "このcustom Skillの処理対象と成果物を持ちます。SubAgentがSkillの意味的な入出力を把握するために使います。" },
    "steps":            { "blockType": "CustomSteps",      "prompt": "..." },
    "guardrails":       { "blockType": "Guardrails",       "prompt": "..." },
    "references":       { "blockType": "References",       "prompt": "..." }
  },
  "harness-render-engine": {
    "title":          { "blockType": "Title",          "prompt": "このSkillのタイトルを持ちます。..." },
    "purpose":        { "blockType": "Purpose",        "prompt": "..." },
    "role":           { "blockType": "Role",           "prompt": "..." },
    "interface":      { "blockType": "Interface",      "prompt": "このengine SkillのI/Oインターフェース定義を持ちます。..." },
    "invocationSpec": { "blockType": "InvocationSpec", "prompt": "このengine Skillの呼び出し仕様（Skills モード / MCP モード）を持ちます。..." },
    "steps":          { "blockType": "EngineSteps",   "prompt": "..." },
    "guardrails":     { "blockType": "Guardrails",     "prompt": "..." },
    "references":     { "blockType": "References",     "prompt": "..." }
  }
}
```

> **ポイント:** engine と custom で blockKey 構成が異なることが S4 から確認できる（engine は `interface` / `invocationSpec` あり、custom は `processingTarget` あり）

---

### Group 2: ブロックレベル

```json
// S5. get_block(harness-render-engine.json, 'purpose')
{
  "prompt": "このSkillの目的を1〜2文で持ちます。Skill選択・ルーティング判断・ドキュメント理解に使います。",
  "value": { "blockType": "Purpose", "title": "目的", "text": "document.json の x-render テンプレートをもとに Markdown・HTML 形式でレンダリングし、人間が読める形式の成果物を生成する。" }
}
```

```json
// S6. get_block(harness-render-engine.json, 'interface')
{
  "prompt": "このengine SkillのI/Oインターフェース定義を持ちます。...",
  "value": {
    "blockType": "Interface", "title": "インターフェース",
    "input": [
      { "name": "documentPath", "type": "string", "required": true,  "description": "レンダリング対象の document.json のファイルパス" },
      { "name": "format",       "type": "string", "required": false, "description": "出力形式。\"md\" | \"html\" | \"both\"（省略時は schema の x-render-target.formats に従う）" }
    ],
    "output": [
      { "name": "renderedPaths", "type": "string[]", "description": "生成されたファイルのパス一覧" },
      { "name": "status",        "type": "string",   "description": "\"success\" | \"error\"" }
    ]
  }
}
```

```json
// S7. get_field(harness-render-engine.json, 'invocationSpec', 'mcpMode')
{
  "prompt": "このengine Skillの呼び出し仕様（Skills モード / MCP モード）を持ちます。...",
  "value": "MCP ツール名: render_document。パラメータ: { documentPath: string, format?: string }。出力先は schema の x-render-target 駆動。has-udd serve 起動後に利用可能。"
}
```

```json
// S8. get_field(analyze-domain-model.json, 'processingTarget', 'artifact')
{
  "prompt": "このcustom Skillの処理対象と成果物を持ちます。SubAgentがSkillの意味的な入出力を把握するために使います。",
  "value": "業務語彙リスト・集約候補リスト・値オブジェクト候補リスト・業務ルール一覧。DomainModelSpec（CREATED 状態）の content に記載できる形式で出力する。"
}
```

---

### Group 3: 配列操作

```json
// S9. get_items(harness-render-engine.json, 'steps', 'items')
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": [
    {
      "stepId": "step-1", "title": "document.json と対応 Schema を取得する",
      "body": "documentPath で指定された document.json を読み込む。schemaRef フィールドから対応する Schema ファイルのパスを解決する。",
      "children": [
        { "stepId": "step-1-1", "title": "document.json を読み込む",  "body": "指定パスの document.json を読み込み、schemaRef を確認する。" },
        { "stepId": "step-1-2", "title": "Schema ファイルを解決する", "body": "schemaRef を schema_repository（importlib.resources）でパッケージ内 src/has_udd/domain/model/{schemaRef}.json として解決する。schema は .has-udd に配布されない。" }
      ]
    },
    { "stepId": "step-2", "title": "レンダリング順序とテンプレートを決定する",     "body": "..." },
    { "stepId": "step-3", "title": "Jinja2 テンプレートを展開して出力を生成する", "body": "..." },
    { "stepId": "step-4", "title": "生成コンテンツを出力先に書き込む",            "body": "..." }
  ]
}
```

```json
// S10. get_item_field(harness-render-engine.json, 'steps', 'items', 'title')
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": ["document.json と対応 Schema を取得する", "レンダリング順序とテンプレートを決定する", "Jinja2 テンプレートを展開して出力を生成する", "生成コンテンツを出力先に書き込む"]
}
```

```json
// S11. get_items_slice(harness-render-engine.json, 'steps', 'items', 0, 2)  ← 先頭2件
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": [
    { "stepId": "step-1", "title": "document.json と対応 Schema を取得する", "body": "...", "children": [...] },
    { "stepId": "step-2", "title": "レンダリング順序とテンプレートを決定する", "body": "..." }
  ]
}
```

```json
// S12. filter_items(harness-render-engine.json, 'interface', 'input', 'required', true)
{
  "prompt": "このengine SkillのI/Oインターフェース定義を持ちます。...",
  "value": [{ "name": "documentPath", "type": "string", "required": true, "description": "レンダリング対象の document.json のファイルパス" }]
}
```

```json
// S13. filter_exists(harness-render-engine.json, 'steps', 'items', 'children')
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": ["step-1"]
}
```

```json
// S14. filter_pattern(harness-render-engine.json, 'steps', 'items', 'stepId', 'step-1.*')
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": [
    {
      "stepId": "step-1", "title": "document.json と対応 Schema を取得する", "body": "...",
      "children": [
        { "stepId": "step-1-1", "title": "document.json を読み込む",  "body": "..." },
        { "stepId": "step-1-2", "title": "Schema ファイルを解決する", "body": "..." }
      ]
    }
  ]
}
```

```json
// S15. get_by_id(harness-render-engine.json, 'steps', 'items', 'stepId', 'step-3')
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": [{ "stepId": "step-3", "title": "Jinja2 テンプレートを展開して出力を生成する", "body": "x-render-order 順に各ブロックを処理する。..." }]
}
```

```json
// S16. get_nested_items(harness-render-engine.json, 'steps', 'items', 'children')
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": [
    { "stepId": "step-1-1", "title": "document.json を読み込む",  "body": "指定パスの document.json を読み込み、schemaRef を確認する。" },
    { "stepId": "step-1-2", "title": "Schema ファイルを解決する", "body": "schemaRef を schema_repository（importlib.resources）でパッケージ内 src/has_udd/domain/model/{schemaRef}.json として解決する。schema は .has-udd に配布されない。" }
  ]
}
```

```json
// S17. get_children(harness-render-engine.json, 'steps', 'items', 'stepId', 'step-1')
{
  "prompt": "このSkillの実行手順を持ちます。...",
  "value": [
    { "stepId": "step-1-1", "title": "document.json を読み込む",  "body": "指定パスの document.json を読み込み、schemaRef を確認する。" },
    { "stepId": "step-1-2", "title": "Schema ファイルを解決する", "body": "schemaRef を schema_repository（importlib.resources）でパッケージ内 src/has_udd/domain/model/{schemaRef}.json として解決する。schema は .has-udd に配布されない。" }
  ]
}
```

---

### Group 4: 再帰

```json
// S18. find_all(harness-render-engine.json, 'stepId')  ← Step + SubStep 全階層
{ "prompt": null, "value": ["step-1", "step-1-1", "step-1-2", "step-2", "step-3", "step-4"] }
```

```json
// S19. find_all(harness-render-engine.json, 'title')  ← ブロックタイトル + ステップタイトル全部
{
  "prompt": null,
  "value": [
    "目的", "役割", "インターフェース", "呼び出し仕様", "実行手順",
    "document.json と対応 Schema を取得する", "document.json を読み込む", "Schema ファイルを解決する",
    "レンダリング順序とテンプレートを決定する", "Jinja2 テンプレートを展開して出力を生成する",
    "生成コンテンツを出力先に書き込む", "ガードレール", "参照"
  ]
}
```

```json
// S20. find_all(analyze-domain-model.json, 'body')  ← 全 Step + SubStep の body を収集
{
  "prompt": null,
  "value": [
    "渡された要件・会話・コードから業務固有の名詞・動詞を抽出し、定義を整理する。",
    "ドメイン固有の名詞を列挙し、各名詞が何を指すかを1文で定義する。",
    "ドメイン固有の動詞を列挙し、誰が・何を・どうするかを明示する。",
    "収集した名詞を「同一性を持つか（エンティティ）」「値で区別されるか（値オブジェクト）」の観点で分類する。",
    "エンティティ群の中からルートエンティティを特定し、「この操作が失敗した場合どこまでロールバックするか」を問いとして集約境界を引く。",
    "識別された集約が所属するサブドメインを特定し、中核・一般・補完のいずれかに分類する。分類の根拠を明示する。"
  ]
}
```

---

### エラー・フォールバック

```json
// E1. get_block('nonexistent')  ← 存在しない blockKey
{ "error": "NOT_FOUND", "message": "blockKey 'nonexistent' は content に存在しません" }
```

```json
// E2. filter_items でマッチなし
{ "error": "NO_MATCH", "message": "フィルタ条件に一致する要素がありませんでした", "value": [] }
```

```json
// F1. scan(通常ファイル) ← schemaRef なし フォールバック
{ "prompt": null, "type": "raw", "content": "# SKILL.md の生テキスト\n## 目的\n..." }
```

---

## 論点一覧

| # | 論点 | 状態 |
|---|---|---|
| Q-1 | クエリ対象の範囲 | ✅ CLOSED |
| Q-2 | クエリ形式・アルゴリズム | ✅ CLOSED |
| Q-3 | I/O インターフェース詳細 | ✅ CLOSED |
| Q-4 | Steps 設計 | ✅ CLOSED |
| Q-5 | ガードレール | ✅ CLOSED |

---

## 論点 Q-1: クエリ対象の範囲 ✅ CLOSED

**3レイヤー構造で統一（確定）:**

| レイヤー | 対象 | アクセス方法 |
|---|---|---|
| インデックス | content から動的算出した index（blockType + prompt） | `query_index()` → Stage 1 |
| コンテンツ | `content.{blockKey}` | `query_block(blockKey)` → Stage 2 |
| 通常ファイル | schemaRef なしのファイル | フォールバック → raw テキスト |

---

## 論点 Q-2: クエリ形式・アルゴリズム ✅ CLOSED

**2段階クエリ + JsonPath（確定）:**
- Stage 1: `query_index` で index（schema の x-prompt-query から動的算出）を返す（jsonpath-ng 不要）
- Stage 2: `query_block(blockKey, jsonpath?)` でセマンティックオブジェクトを返す
- JsonPath は Stage 2 のサブフィールド絞り込みに使用（`jsonpath_ng.ext` 必須）
- 戻り値は常に `{ prompt, value }` の形（意味と値がセット）

---

## 論点 Q-3: I/O インターフェース詳細 ✅ CLOSED

**採用方針: セマンティック操作で JsonPath を内部に隠蔽（案A）**

Orchestrator は JsonPath を直接指定しない。セマンティックな操作名とパラメータだけを渡す。
Python 実装の内部で JsonPath に変換して実行する。

---

### セマンティック操作カタログ

#### Group 1: ドキュメントレベル（blockKey 不要）

| 操作 | シグネチャ | 内部 JsonPath | 主な用途 |
|---|---|---|---|
| `scan` | `scan(path)` | ファイル全文読み込み | SKILL.md・通常ファイル取得 |
| `get_meta` | `get_meta(path)` | `$.[field]`（直接アクセス） | documentId / status / skillKind / tags 確認 |
| `index_scan` | `index_scan(path)` | content 各ブロックの blockType → schema の x-prompt-query を引いて index を動的算出 | 単一ドキュメントのブロック一覧 |
| `index_scan_dir` | `index_scan_dir(dirPath)` | 全 document.json を走査し、各 index を動的算出して集約 | ディレクトリ横断の横断把握 |

戻り値:
```json
// scan
{ "type": "raw", "content": "..." }

// get_meta
{ "documentId": "...", "status": "...", "skillKind": "...", "tags": [...] }

// index_scan
{ "purpose": { "blockType": "Purpose", "prompt": "..." }, ... }

// index_scan_dir
{
  "harness-render-engine": { "purpose": {...}, "steps": {...} },
  "analyze-domain-model":  { "purpose": {...}, "steps": {...} }
}
```

---

#### Group 2: ブロックレベル（blockKey 必須）

| 操作 | シグネチャ | 内部 JsonPath | 主な用途 |
|---|---|---|---|
| `get_block` | `get_block(path, blockKey)` | `$.content.{blockKey}` | ブロック全体取得 |
| `get_field` | `get_field(path, blockKey, field)` | `$.content.{blockKey}.{field}` | ブロック内の特定フィールド |

戻り値:
```json
// get_block(path, "purpose")
{ "prompt": "...", "value": { "blockType": "Purpose", "title": "...", "text": "..." } }

// get_field(path, "invocationSpec", "mcpMode")
{ "prompt": "...", "value": "MCP ツール名: render_document。..." }
```

---

#### Group 3: 配列操作（blockKey + arrayField 必須）

| 操作 | シグネチャ | 内部 JsonPath | 主な用途 |
|---|---|---|---|
| `get_items` | `get_items(path, blockKey, arrayField)` | `$.content.{blockKey}.{arrayField}[*]` | 配列全要素 |
| `get_item_field` | `get_item_field(path, blockKey, arrayField, field)` | `$.content.{blockKey}.{arrayField}[*].{field}` | 各要素の特定フィールドだけ |
| `get_items_slice` | `get_items_slice(path, blockKey, arrayField, start, end)` | `$.content.{blockKey}.{arrayField}[start:end]` | 先頭N件等のスライス |
| `filter_items` | `filter_items(path, blockKey, arrayField, key, value)` | `$.content.{blockKey}.{arrayField}[?(@.{key}=={value})]` | 値フィルタ |
| `filter_exists` | `filter_exists(path, blockKey, arrayField, field)` | `$.content.{blockKey}.{arrayField}[?(@.{field})]` | フィールド存在チェック |
| `filter_pattern` | `filter_pattern(path, blockKey, arrayField, field, pattern)` | `$.content.{blockKey}.{arrayField}[?(@.{field}=~"{pattern}")]` | 正規表現フィルタ |
| `get_by_id` | `get_by_id(path, blockKey, arrayField, idField, idValue)` | `$.content.{blockKey}.{arrayField}[?(@.{idField}=="{idValue}")]` | ID指定で1件取得 |
| `get_nested_items` | `get_nested_items(path, blockKey, arrayField, nestedField)` | `$.content.{blockKey}.{arrayField}[*].{nestedField}[*]` | 全要素のネスト配列を展開 |
| `get_children` | `get_children(path, blockKey, arrayField, idField, idValue)` | `$.content.{blockKey}.{arrayField}[?(@.{idField}=="{idValue}")].children[*]` | 特定要素の子要素 |

実証済み出力例:
```json
// get_items(path, "steps", "items") → Steps タイトル一覧
{ "prompt": "...", "value": [{ "stepId":"step-1", "title":"...", ... }, ...] }

// get_item_field(path, "steps", "items", "title")
{ "prompt": "...", "value": ["step1タイトル", "step2タイトル", ...] }

// filter_items(path, "interface", "input", "required", true)
{ "prompt": "...", "value": [{ "name": "documentPath", "type": "string", ... }] }

// filter_exists(path, "steps", "items", "children")
{ "prompt": "...", "value": ["step-1"] }   ← stepId 一覧

// filter_pattern(path, "steps", "items", "stepId", "step-1.*")
{ "prompt": "...", "value": ["step-1 タイトル"] }

// get_by_id(path, "steps", "items", "stepId", "step-2")
{ "prompt": "...", "value": [{ "stepId":"step-2", "title":"...", "body":"..." }] }

// get_nested_items(path, "steps", "items", "children")
{ "prompt": "...", "value": [{ "stepId":"step-1-1", ... }, { "stepId":"step-1-2", ... }] }

// get_children(path, "steps", "items", "stepId", "step-1")
{ "prompt": "...", "value": [{ "stepId":"step-1-1", ... }, { "stepId":"step-1-2", ... }] }
```

---

#### Group 4: 再帰（全階層横断）

| 操作 | シグネチャ | 内部 JsonPath | 主な用途 |
|---|---|---|---|
| `find_all` | `find_all(path, fieldName)` | `$..{fieldName}` | 全階層から特定フィールドを収集 |

```json
// find_all(path, "stepId")
{ "prompt": null, "value": ["step-1", "step-1-1", "step-1-2", "step-2", "step-3", "step-4"] }

// find_all(path, "title") → ブロックタイトル + ステップタイトル + SubStep タイトル 全件
{ "prompt": null, "value": ["目的", "役割", "インターフェース", ..., "document.json を読み込む", ...] }
```

---

### 戻り値の統一構造

```json
// 正常系
{ "prompt": "string | null", "value": "any" }

// エラー系
{ "error": "NOT_FOUND | INVALID_JSON | INVALID_PATH | NO_MATCH", "message": "..." }

// 通常ファイル（schemaRef なし）フォールバック
{ "prompt": null, "type": "raw", "content": "ファイルの生テキスト" }
```

`prompt` は対象ブロックの blockType から schema の x-prompt-query を引いて動的算出した値（Group 1 の一部と Group 4 は null）

### ユーザー見解

案A（セマンティック操作で JsonPath を内部に隠蔽）に合意 ✅

---

## 論点 Q-4: Steps 設計 ✅ CLOSED

### AI の立場（たたき台）

**設計方針: 4フェーズ構成**

セマンティック操作は16本あるが、Python スクリプトの実行フローは共通。Steps はフェーズ単位で構成し、各フェーズ内で operation ごとに分岐する。

```
Step 1: 入力パラメータを受け取り、バリデーションする
Step 2: ファイル / ディレクトリを読み込む
Step 3: operation に応じてセマンティック操作を実行する
  SubStep 3-1: Group 1（ドキュメントレベル）の操作を実行する
  SubStep 3-2: Group 2（ブロックレベル）の操作を実行する
  SubStep 3-3: Group 3（配列操作）の操作を実行する
  SubStep 3-4: Group 4（再帰）の操作を実行する
Step 4: 結果を統一形式でラップして返す
Step 5: エラーハンドリング
```

**Step 1: 入力パラメータを受け取り、バリデーションする**

Skills モードは `has-udd query` CLI、MCP モードは `query_document` ツール経由で以下のパラメータを受け取る。

| パラメータ | 型 | 必須 | 説明 |
|---|---|---|---|
| `operation` | string | ✅ | セマンティック操作名（例: `get_block`, `filter_items`） |
| `path` | string | ✅ | 対象ファイルパスまたはディレクトリパス |
| `blockKey` | string | 条件付き | Group 2・3 で必須 |
| `arrayField` | string | 条件付き | Group 3 で必須 |
| `field` | string | 条件付き | `get_field` / `get_item_field` / `filter_exists` / `filter_pattern` で必須 |
| `idField` | string | 条件付き | `get_by_id` / `get_children` で必須 |
| `idValue` | string | 条件付き | `get_by_id` / `get_children` で必須 |
| `key` | string | 条件付き | `filter_items` で必須 |
| `value` | any | 条件付き | `filter_items` で必須 |
| `pattern` | string | 条件付き | `filter_pattern` で必須（正規表現文字列） |
| `start` | integer | 条件付き | `get_items_slice` で必須 |
| `end` | integer | 条件付き | `get_items_slice` で必須 |
| `fieldName` | string | 条件付き | `find_all` で必須 |
| `nestedField` | string | 条件付き | `get_nested_items` で必須 |

**Step 2: ファイル / ディレクトリを読み込む**

- `index_scan_dir` の場合: `glob(".has-udd/documents/**/*.json")` で全 document.json を収集し、各ファイルを `json.load()` する
- それ以外: `path` で指定された単一ファイルを `json.load()` する
- `schemaRef` フィールドが存在しないファイルは「通常ファイル」として扱い、`scan` 以外でアクセスされた場合は raw フォールバックを適用する
- `schemaRef` を持つファイルでは、prompt / index を算出するため schema を解決する。`schema_repository`（`importlib.resources`）でパッケージ内 `src/has_udd/domain/model/{schemaRef}.json` をロードする（index は document.json に保存されていないため、毎回 schema の x-prompt-query から動的に算出する）

**Step 3: operation に応じてセマンティック操作を実行する**

*SubStep 3-1: Group 1（ドキュメントレベル）*

| operation | Python 実装 |
|---|---|
| `scan` | `return {"type": "raw", "content": raw_text}` |
| `get_meta` | `return {k: doc[k] for k in ["documentId","status","skillKind","tags","schemaRef"] if k in doc}` |
| `index_scan` | `return build_index(doc, schema_repository.load(doc["schemaRef"]))`（各ブロックの blockType → schema の x-prompt-query を引いて `{key: {blockType, prompt}}` を組み立てる） |
| `index_scan_dir` | `return {doc["documentId"]: build_index(doc, schema_repository.load(doc["schemaRef"])) for doc in all_docs}` |

*SubStep 3-2: Group 2（ブロックレベル）*

| operation | Python 実装 |
|---|---|
| `get_block` | `block = doc["content"][blockKey]; schema = schema_repository.load(doc["schemaRef"]); prompt = schema["$defs"][block["blockType"]+"Block"]["x-prompt-query"]; return {"prompt": prompt, "value": block}` |
| `get_field` | `value = doc["content"][blockKey][field]; return {"prompt": ..., "value": value}` |

*SubStep 3-3: Group 3（配列操作、jsonpath_ng.ext 使用）*

| operation | 内部 JsonPath |
|---|---|
| `get_items` | `$.{arrayField}[*]` |
| `get_item_field` | `$.{arrayField}[*].{field}` |
| `get_items_slice` | `$.{arrayField}[{start}:{end}]` |
| `filter_items` | `$.{arrayField}[?(@.{key}=={value})]` |
| `filter_exists` | `$.{arrayField}[?(@.{field})]` |
| `filter_pattern` | `$.{arrayField}[?(@.{field}=~"{pattern}")]` |
| `get_by_id` | `$.{arrayField}[?(@.{idField}=="{idValue}")]` |
| `get_nested_items` | `$.{arrayField}[*].{nestedField}[*]` |
| `get_children` | `$.{arrayField}[?(@.{idField}=="{idValue}")].children[*]` |

全操作共通: `block = doc["content"][blockKey]` を取得してから `jsonpath_ng.ext.parse(expr).find(block)` を実行する。

*SubStep 3-4: Group 4（再帰）*

| operation | Python 実装 |
|---|---|
| `find_all` | `jsonpath_ng.ext.parse(f"$..{fieldName}").find(doc)` でドキュメント全体を再帰走査 |

**Step 4: 結果を統一形式でラップして返す**

```python
# 正常系（Group 2・3・4）
return {"prompt": prompt, "value": value}

# Group 1（scan）
return {"prompt": None, "type": "raw", "content": raw_text}

# Group 1（get_meta / index_scan / index_scan_dir）
return result  # prompt なしの dict をそのまま返す

# エラー系
return {"error": "NOT_FOUND | INVALID_JSON | INVALID_PATH | NO_MATCH", "message": "..."}
```

**Step 5: エラーハンドリング**

各ステップで発生しうる例外を Python の try/except で捕捉し、`{ "error": "<CODE>", "message": "<詳細>" }` 形式で返す。例外をそのまま raise しない（呼び出し元の AI がハンドルできないため）。

---

### 論点 Q-4-A: Step 粒度（discussion）

**案X: フェーズ単位（上記案）**
- Step 3 の中に16操作のルーティングが収まる
- Step 数は少なく見通しがよい
- Step 3 の body が大きくなる → SubStep で補う

**案Y: 操作グループ単位（4 Step に分割）**
- Step 1: scan / get_meta
- Step 2: index_scan / index_scan_dir
- Step 3: get_block / get_field
- Step 4: 配列操作 + 再帰
- 操作が増えると Step も増える → 拡張コストが高い

→ **案X（フェーズ単位）を推奨。** Python の実装構造（読み込み→ルーティング→返却）と Steps の構造を一致させる方が直感的。

### ユーザー見解

案X（フェーズ単位）に合意 ✅

---

## 論点 Q-5: ガードレール ✅ CLOSED

### AI の立場（たたき台）

4つのカテゴリに整理する。

---

**カテゴリ1: 入力バリデーション**

| # | ガードレール | エラーコード |
|---|---|---|
| G1 | `operation` が定義済みの16操作以外の場合はエラーを返す | `INVALID_OPERATION` |
| G2 | `path` が存在しないファイル / ディレクトリの場合はエラーを返す | `INVALID_PATH` |
| G3 | Group 2・3 操作で `blockKey` が未指定の場合はエラーを返す | `MISSING_PARAM` |
| G4 | Group 3 操作で `arrayField` が未指定の場合はエラーを返す | `MISSING_PARAM` |
| G5 | `filter_pattern` で `pattern` が不正な正規表現の場合はエラーを返す（実行しない） | `INVALID_PATTERN` |
| G6 | `path` に `../` 等のパストラバーサルが含まれる場合はエラーを返す | `INVALID_PATH` |
| G7 | `index_scan_dir` はプロジェクトルート配下のディレクトリのみを対象とする | `INVALID_PATH` |

---

**カテゴリ2: データアクセス**

| # | ガードレール | エラーコード |
|---|---|---|
| G8 | 対象ファイルが有効な JSON でない場合はパースエラーを返す（例外を素通りさせない） | `INVALID_JSON` |
| G9 | `blockKey` が `content` に存在しない場合はエラーを返す | `NOT_FOUND` |
| G10 | 対象ブロックの `blockType` に対応する schema の `x-prompt-query` が見つからない場合はエラーを返す（schema 不整合検知） | `NOT_FOUND` |
| G11 | `arrayField` がブロック内に存在しない / 配列型でない場合はエラーを返す | `NOT_FOUND` |

---

**カテゴリ3: クエリ結果**

| # | ガードレール | 扱い |
|---|---|---|
| G12 | クエリ結果が空（マッチなし）の場合は `{ "error": "NO_MATCH", "value": [] }` を返す | エラーではなく正常系の一種 |
| G13 | `schemaRef` を持たないファイルはエラーにせず `{ "type": "raw", "content": "..." }` で返す | raw フォールバック |

---

**カテゴリ4: Harness 原則**

| # | ガードレール |
|---|---|
| G14 | Python スクリプトが実行する。AI がファイルを直接読んで値を解釈・返却してはならない |
| G15 | すべての例外を try/except で捕捉し、`{ "error": "<CODE>", "message": "<詳細>" }` 形式で返す（例外を AI に素通りさせない） |
| G16 | クエリ操作は読み取り専用とする。ファイルへの書き込み・削除は一切行わない |

---

**エラーコード一覧（統一）**

| code | 意味 |
|---|---|
| `INVALID_OPERATION` | 定義外の operation 名 |
| `INVALID_PATH` | 存在しないパス / パストラバーサル |
| `INVALID_JSON` | JSON パース失敗 |
| `INVALID_PATTERN` | 不正な正規表現 |
| `MISSING_PARAM` | 必須パラメータ未指定 |
| `NOT_FOUND` | blockKey / arrayField が存在しない |
| `NO_MATCH` | フィルタ結果が空 |

### ユーザー見解

4カテゴリ・16項目に合意 ✅

---

## 合意事項

- クエリアルゴリズム: 2段階（index_scan → AI 判断 → Query）✅
- 技術スタック: `jsonpath_ng.ext`（フィルタ式対応のため ext 版必須）✅
- Skills/MCP 両モードとも Python スクリプト実行（AI が直接 JSON を読まない）✅
- セマンティック操作で JsonPath を内部に隠蔽（案A）✅
- セマンティック操作16本: Group 1（ドキュメントレベル4本）/ Group 2（ブロックレベル2本）/ Group 3（配列操作9本）/ Group 4（再帰1本）✅
- 戻り値統一形式: `{ prompt, value }` / raw フォールバック: `{ type: "raw", content }` / エラー: `{ error, message }` ✅
- Steps 設計: フェーズ単位5ステップ（案X）✅
- ガードレール: 4カテゴリ16項目（G1〜G16）✅

---

## 次のアクション

- `.has-udd/skills/harness-query-engine.json` 作成済み ✅（SkillSchema/v1 バリデーション通過）
- 次: Phase 2 残りの engine Skill document.json 作成（harness-knowledge-engine / harness-scaffold-engine / harness-audit-engine）。template / coding は Phase 3 に延期
