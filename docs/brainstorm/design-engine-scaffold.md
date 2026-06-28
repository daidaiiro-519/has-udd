# harness-scaffold-engine 設計ブレスト

## 目的

`harness-scaffold-engine` を設計する。schema から**空の document.json を生成**し、AI が値を埋められる状態を作る engine。
document の **validate 操作**も担う（schema 適合の両端: create=誕生時 / validate=充填後）。

---

## 前提（確定事項から）

- engine セット再導出（ES-1）で **spec-engine → scaffold-engine** に再構成
- 責務 = { **create**（schema → 空 document.json）, **validate**（document.json を schema で検証） }
- ⭐ **`_index` は document.json に保存しない（動的）**。`_index` の中身 = schema の x-prompt-query から導出可能 → 保存は重複+陳腐化。query/knowledge engine が読み取り時に動的計算する。**scaffold は `_index` を生成しない**
- schema はパッケージ内（`src/has_udd/domain/model/`）から `schema_repository`（importlib.resources）で解決。`.has-udd` に schema は無い
- 出力先 = `.has-udd/documents/{type}/{documentId}.json`（SOURCE）
- core 共有: schema 走査（$defs 解決・blockType→$def）は render と共有 / scan は query と共有
- 戻り値 = `{ prompt, value }` / `{ error, prompt, message }`
- **Harness 原則**: AI は schema を直接読まない・JSON 構造を作らない。engine が空の正しい構造 + 埋めるべき指示（x-prompt-write）を用意し、AI は**値だけ**埋める

---

## 論点一覧

| # | 論点 | 状態 |
|---|---|---|
| SC-1 | scaffold の責務範囲と create の出力契約（空骨格 + 充填ガイド） | ✅ CLOSED |
| SC-2 | skeleton 生成・fill テンプレート生成・fill 方式 | ✅ CLOSED |
| SC-3 | discriminator の扱い（skillKind/agentKind 等 if/then/else を解決する入力） | ✅ CLOSED |
| SC-4 | validate 操作の設計（jsonschema・戻り値・status 遷移との関係） | ✅ CLOSED |
| SC-5 | インターフェース（input / output）・invocationSpec | ✅ CLOSED |
| SC-6 | Steps / Guardrails | ✅ CLOSED |

---

## 論点 SC-1: 責務範囲と create の出力契約

### AI の立場（たたき台）

**scaffold = create + validate の2操作。create の出力は「空骨格（value）+ 充填ガイド（prompt）」。**

#### なぜ create が「骨格 + ガイド」を返すか

Harness 原則: AI は schema を直接読まない。でも値を埋めるには「何を埋めるか」を知る必要がある。
→ x-prompt-write（各フィールドの記入指示）を **engine が prompt として surface** する。AI は schema を見ずに、返ってきた prompt を読んで value を埋める。

```
create(schemaRef, documentId, discriminator…)
  → value:  空の document.json（メタ + const(blockType/title) + documentId 由来 + 空の値フィールド + status/打刻）
            ※ _index は含めない（動的計算なので保存しない）
  → prompt: 「以下のフィールドを埋めてください:
             content.purpose.text — <x-prompt-write>
             content.role.items   — <x-prompt-write>
             ...」（x-prompt-write を機械的に集約）
```

- **value** = jsonschema 的に「構造は正しいが値は空」の document.json。`blockType`/`title`/`documentType`/`schemaRef` 等の const は埋まり、documentId 由来（title.title）・初期 status・打刻も埋まる。**`_index` は持たない**
- **prompt** = 埋めるべきフィールドと各々の x-prompt-write を並べたガイド（query/knowledge と同じく prompt で AI を導く）

これで「engine が骨格、AI が値」の Harness 原則が成立する。

#### validate 操作

```
validate(documentPath)
  → schema をパッケージから解決 → Draft202012Validator で検証
  → value: { valid: bool, errors: [...] }
  → prompt: 不適合なら「どこを直すか」を案内 / valid なら VALIDATED への遷移可を通知
```

CREATED → VALIDATED の遷移判定に使う。

#### 2操作を1 engine に置く根拠

create（誕生時に valid な骨格）と validate（充填後に valid か検証）は**同じ「schema 適合」の関心の両端**。凝集する（ES-2b で合意）。

---

### 合意（SC-1 CLOSED）

scaffold = create + validate の2操作。create は `{ prompt, value }`:
- value = 空 document.json（メタ + const + documentId 由来 + 空値 + status/打刻。**`_index` は持たない**）
- prompt = 埋めるべきフィールド × x-prompt-write のガイド（AI は schema を見ずに値だけ埋める）
validate は jsonschema で document を検証し `{ valid, errors }` + prompt を返す（CREATED→VALIDATED 判定）。

---

## 論点 SC-2: skeleton 生成アルゴリズム

### AI の立場（たたき台）

**schema を再帰的に走査し、required を辿って空構造を組み立てる。const は埋め、x-prompt-write を持つ値フィールドは空にしつつ prompt に集約する。**

```python
def create(schema_ref, document_id, discriminator):
    schema = schema_repository.load(schema_ref)          # パッケージ内 domain/model

    # 1. ルートの固定値・与えられた値を埋める
    doc = {
        "documentId":   document_id,
        "documentType": _const_of(schema, "documentType"),   # const から
        "schemaRef":    schema_ref,
        **discriminator,                                      # skillKind / agentKind 等（SC-3）
        "status":       _initial_status(schema),              # status enum の先頭（DRAFT/CREATED）
        "tags":         [],
        "createdAt":    now, "updatedAt": now,
    }

    # 2. if/then/else を discriminator で解決して content の $ref を特定（SC-3）
    content_def = _resolve_content(schema, discriminator)     # 例: EngineContent

    # 3. content の各ブロックを走査して空骨格 + 充填ガイドを組み立てる
    content, guide = {}, []
    for key, ref in content_def["properties"].items():
        block_def = _deref(schema, ref)                       # 例: PurposeBlock
        block, block_guide = _scaffold_block(schema, key, block_def)
        content[key] = block
        guide += block_guide
    doc["content"] = content

    return { "prompt": _format_guide(guide), "value": doc }


def _scaffold_block(schema, key, block_def):
    block, guide = {}, []
    for field, fdef in block_def["properties"].items():
        if "const" in fdef:
            block[field] = fdef["const"]                      # blockType / title 等は埋める
        elif _is_document_id_derived(fdef):
            block[field] = document_id                        # title.title = documentId
        else:
            block[field] = _empty_of(fdef["type"])            # "" / [] / {} で空に
            if "x-prompt-write" in fdef:
                guide.append((f"content.{key}.{field}", fdef["x-prompt-write"]))
    return block, guide
```

**設計のポイント:**

| 項目 | 方針 |
|---|---|
| const フィールド（blockType / title / documentType / schemaRef） | schema の `const` をそのまま埋める |
| documentId 由来（title.title） | document_id を埋める（x-prompt-write が「documentId をそのまま」の場合） |
| 値フィールド（x-prompt-write 持ち） | 型に応じた空値（string→`""` / array→`[]` / object→`{}`）+ x-prompt-write を guide に集約 |
| 配列要素の中身（steps.items[] 等） | **空配列のまま**。要素の構造は埋めない（AI が要素を足すときの形は prompt で案内） |
| `_index` | **生成しない**（動的） |
| required / optional | required は必ず骨格に含める。optional も骨格に含めて空にする（AI が埋めやすい）か・含めないか → SC-2 で確定したい |

**配列要素の扱い（要検討）:**
`steps.items[]` のように配列の中身が複雑なオブジェクト（stepId/title/body/children）の場合、空配列 `[]` を返し、prompt で「各要素は stepId/title/body/children を持つ」と案内する。要素のテンプレート（空の1要素）を入れておくかは選択肢。私の推奨は **空配列 + prompt 案内**（AI が必要数だけ足せる・余計な空要素を残さない）。

### 合意（SC-2 CLOSED）

#### 1. scaffold は3操作（create / fill / validate）

| 操作 | 入力 | 動作 |
|---|---|---|
| **create** | schemaRef, documentId, discriminator | 空 skeleton + fillTemplate + guide を返す（空 skeleton を `.has-udd/documents/{type}/{id}.json` に永続化・CREATED） |
| **fill** | documentPath, values（AI 生成の field→value 写像） | 値を**宣言済み値フィールドだけ**に機械的に書き込む（構造は触らせない） |
| **validate** | documentPath | jsonschema 検証 → `{ valid, errors }` + prompt（CREATED→VALIDATED 判定） |

#### 2. skeleton 生成（schema 再帰走査）

| フィールド種別 | 扱い |
|---|---|
| const（blockType/title/documentType/schemaRef） | schema の const を埋める |
| documentId 由来（title.title） | document_id を埋める |
| 値フィールド（x-prompt-write 持ち） | 型に応じた空値（`""`/`[]`/`{}`） |
| optional（guardrails/references） | **空で骨格に含める（案A）**・guide で【任意】明示 |
| 配列要素 | **空配列（案X）**・要素の形は fillTemplate で案内 |
| `_index` | 生成しない（動的） |

if/then/else は discriminator（skillKind 等）で解決して content 分岐を特定（SC-3）。

#### 3. fillTemplate 生成（要素・ネストを再帰展開）

skeleton と同じ走査だが、**値フィールドの型と要素構造まで再帰展開**する（`_shape_of`）:
```json
{ "path": "content.steps.items", "type": "array", "prompt": "...", "required": true,
  "element": {
    "stepId": { "type": "string", "prompt": "step-1, step-2..." },
    "title":  { "type": "string", "prompt": "..." },
    "body":   { "type": "string", "prompt": "..." },
    "children": { "type": "array", "prompt": "...", "element": { "stepId": {...}, "title": {...}, "body": {...} } }
  }
}
```
AI は element の形を見て正しい構造の値を必要数だけ生成 → fill が書き込む。

#### 4. ⭐ fillTemplate は schema から機械生成可能（決定論的・AI 不要）

全要素（path/type/prompt/required/enum/要素形）が JSON Schema から再帰走査で取得できる。$ref deref・allOf マージ・if/then/else 解決・有界再帰も機械的。

**唯一の制約 = scaffoldable schema 規約（Phase 3 全 Schema に課す）:**
1. 値フィールドに `oneOf`/`anyOf` を使わない（文書分岐は if/then/else + discriminator のみ）
2. 開いた `additionalProperties` を使わない（固定 properties）
3. 再帰は有界にする（無限ネスト禁止）

#### 5. fill 方式を採る根拠（簡易案より優先）

- **信頼性**: AI は「値（内容）」だけ生成・engine が JSON を組む → JSON 機構ミス（カンマ/ネスト/フィールド名）がゼロ。「ちゃんと埋まるか」に直接効く
- **再現性**: 構造は engine 管理で決定論的・値だけ変わる（Harness の書き込み版）
- **書き込み時バリデーション・schema 進化からの decoupling・マルチツール一貫性**
- 簡易案（AI 直接 Write + validate）でも安全性の大半は得られるが、JSON 機構ミス根絶 + 再現性が fill 方式の上乗せ価値

---

## 論点 SC-3: discriminator の扱い ✅ CLOSED

### 合意内容

**discriminator（skillKind/agentKind 等）= 文書の形を分岐させる判別キー。create の入力で受け取り、schema の if/then/else を機械的に解決する。**

- discriminator = 「どの分岐の形で空骨格を作るかを決める入力値」（例: SkillSchema の skillKind=engine/custom、AgentSchema の agentKind=orchestrator/subagent）
- create 入力: `discriminator = { "skillKind": "custom" }`
- 判別キーの特定: engine が schema の `if.properties` のキーから機械抽出（どのキーが必要か schema から分かる）
- 評価: `_matches(schema["if"], discriminator)` で then/else を解決 → content の $ref を特定（例: CustomContent / SubAgentContent）
- 未指定時: `MISSING_DISCRIMINATOR` + prompt で候補（enum 値）を案内
- discriminator 値は document.json のルートフィールドに**create 時に埋める**（AI が後で埋めるのではない）
- ネストした if/then/else は再帰評価（多段分岐対応）

```python
def _resolve_content(schema, discriminator):
    if _matches(schema["if"], discriminator):
        return _deref(schema, schema["then"]["properties"]["content"])
    return _deref(schema, schema["else"]["properties"]["content"])
```

---

## 論点 SC-4: validate 操作の詳細（status 遷移との関係）

### AI の立場（たたき台）

**validate = jsonschema 検証 + status 遷移可否の判定。実際の status 書き換えは別操作（または fill 経由）。**

#### validate の戻り値

```
validate(documentPath)
  → schema をパッケージ解決 → Draft202012Validator(schema).iter_errors(doc)
  → value: { "valid": bool, "errors": [ {path, message}, ... ], "nextStatus": "VALIDATED" | null }
  → prompt: valid なら「VALIDATED へ遷移できます」/ invalid なら「content.purpose.text が空です。…」
```

#### status 遷移は誰がやるか（論点）

`CREATED → VALIDATED` の status 書き換えを:
- 案A: validate が valid 時に自動で status を書き換える（validate に副作用）
- 案B: validate は判定だけ（純粋）。status 書き換えは fill / 別操作が行う ← 私の推奨

私の推奨は **案B**。validate を「検証のみ・副作用なし」に保つ（冪等・安全）。status 遷移は明示的な操作（fill で status フィールドを更新、or Orchestrator が遷移を指示）で行う。validate は「遷移してよいか」を `nextStatus` で示すだけ。

#### validate のタイミング

- fill 後（CREATED→VALIDATED 判定）
- render 前（render-engine も内部で validate を呼ぶ・R-4）。**ロジックは shared core**（validate 操作も render も同じ jsonschema validator を使う）

#### status 値は schema 由来

status の enum（DRAFT/ACTIVE/DEPRECATED や CREATED/VALIDATED/RENDERED）は schema 定義。`nextStatus` も schema の status enum の「次」を機械的に示す。

### 合意（SC-4 CLOSED・案B）

- validate は **検証のみ・副作用なし（冪等）**。status 書き換えはしない
- 戻り値: `{ valid, errors: [{path,message}], nextStatus }` + prompt（valid なら遷移可案内 / invalid なら修正箇所案内）
- status 遷移（CREATED→VALIDATED）は **fill / 別操作が明示的に行う**（validate は `nextStatus` で「遷移可」を示すだけ）
- validate ロジックは **shared core**（render-engine の R-4 内部検証と同じ jsonschema validator を共有）
- 何度呼んでも安全（render 前チェック等で繰り返し呼べる）

---

## 論点 SC-5: インターフェース（input / output）・invocationSpec

### AI の立場（たたき台）

3操作（create / fill / validate）を1 engine の Interface にまとめる。query/render と同じく `operation` で分岐。

#### 入力（input）

| name | type | required | 対象操作 | 説明 |
|---|---|---|---|---|
| operation | string | ✅ | 全 | `create` / `fill` / `validate` |
| schemaRef | string | create | create | 生成元 schema（例: SkillSchema/v1） |
| documentId | string | create | create | 生成する document の id |
| discriminator | object | create（schema に分岐がある場合） | create | 判別キー写像（例: `{skillKind:"custom"}`） |
| documentPath | string | fill/validate | fill, validate | 対象 document.json のパス |
| values | object | fill | fill | AI 生成の field→value 写像 |

#### 出力（output）

| name | type | 説明 |
|---|---|---|
| prompt | string \| null | create: 充填ガイド / fill: 結果や残り未充填の案内 / validate: 遷移可否・修正案内 |
| value | object | create: `{ skeleton, fillTemplate }` / fill: `{ documentPath, written: [...] }` / validate: `{ valid, errors, nextStatus }` |
| error | string | エラー時のみ。INVALID_SCHEMA_REF / MISSING_DISCRIMINATOR / INVALID_VALUES / VALIDATION_ERROR 等 |
| message | string | エラー時のみ。人間可読な詳細 |

#### invocationSpec

```
Skills モード:
  has-udd scaffold --operation create   --schema-ref <ref> --document-id <id> [--discriminator '<json>']
  has-udd scaffold --operation fill      --path <documentPath> --values '<json>'
  has-udd scaffold --operation validate  --path <documentPath>

MCP モード:
  MCP ツール名: scaffold_document
  パラメータ: { operation, schemaRef?, documentId?, discriminator?, documentPath?, values? }
```

---

### 合意（SC-5 CLOSED）

3操作（create/fill/validate）を1 engine に `operation` 分岐でまとめる。create 出力は `{ skeleton, fillTemplate }` を両方返す。input/output/invocationSpec は上表の通り。

---

## 論点 SC-6: Steps / Guardrails ✅ CLOSED

### Steps（operation 別）

```
Step 1: 入力を検証する
  SubStep 1-1: operation が create / fill / validate のいずれかか確認する
  SubStep 1-2: operation 別の必須パラメータを確認する（create: schemaRef/documentId / fill: documentPath/values / validate: documentPath）

Step 2: schema をパッケージから解決する
  SubStep 2-1: schemaRef（create）or document の schemaRef（fill/validate）から schema_repository（importlib.resources）で解決する

Step 3: operation に応じて処理する
  SubStep 3-1（create）: discriminator で if/then/else を解決 → content 分岐を特定 → schema を再帰走査して skeleton（const/documentId由来/空値・_index なし）と fillTemplate（要素再帰展開）を生成 → skeleton を .has-udd/documents/{type}/{id}.json に書き出し（CREATED）
  SubStep 3-2（fill）: values の各 field path が schema の宣言済み値フィールドか検証 → 該当フィールドのみに書き込み（構造は触らない）→ updatedAt 更新
  SubStep 3-3（validate）: Draft202012Validator で検証 → { valid, errors, nextStatus } を組み立てる（status 書き換えはしない・冪等）

Step 4: 結果を { prompt, value } でラップする（shared/result）

Step 5: エラーを { error, prompt, message } でラップする（全例外を try/except）
```

### Guardrails

**入力検証**
- operation が create/fill/validate 以外なら `INVALID_OPERATION` + prompt
- create で必要な discriminator が未指定なら `MISSING_DISCRIMINATOR` + prompt（候補 enum を案内）
- schemaRef に対応する schema がパッケージに無ければ `INVALID_SCHEMA_REF` + prompt

**create**
- _index を生成しない（動的）
- const / documentId 由来のみ engine が埋める。x-prompt-write を持つ値フィールドは空にして fillTemplate/guide に集約
- scaffoldable schema 規約（値フィールドに oneOf/anyOf 不可・開いた additionalProperties 不可・有界再帰）に反する schema は `UNSCAFFOLDABLE_SCHEMA` + prompt

**fill**
- values の field path が schema の**宣言済み値フィールド**でなければその値を拒否し skipped に記録（構造改変・未知フィールドへの書き込み禁止）
- const / readonly / discriminator フィールドへの書き込みは拒否

**validate**
- 副作用なし（status を書き換えない・冪等）
- 全エラーを収集してから返す（1件で止めない）

**Harness 原則**
- Python が skeleton 生成・書き込み・検証を担う。AI は値（内容）だけ生成する
- 書き込みは `.has-udd/documents/` 配下のみ（パストラバーサル禁止）
- 全例外を捕捉し `{ error, prompt, message }` で返す

---

## 合意事項（全 SC CLOSED）

| # | 合意 |
|---|---|
| SC-1 | scaffold = create + fill + validate。create は { skeleton, fillTemplate, guide }。値は AI・書き込みは engine |
| SC-2 | skeleton/fillTemplate は schema 再帰走査で機械生成。fill 方式（AI 値生成・engine 書き込み）。scaffoldable schema 規約 |
| SC-3 | discriminator（skillKind 等）を create 入力で受け取り if/then/else を機械解決。判別キーは schema の if から抽出 |
| SC-4 | validate は副作用なし・冪等。{ valid, errors, nextStatus }。status 遷移は fill/別操作 |
| SC-5 | 3操作を operation 分岐で1 engine に。input/output/invocationSpec 確定 |
| SC-6 | Steps 5フェーズ・Guardrails 4カテゴリ |

---

## 次のアクション

全 SC CLOSED → harness-scaffold-engine ブレスト完了。残る engine ブレストは **audit** のみ。

---

## 旧 合意事項欄

（論点解決後に記録）

---

## 次のアクション

SC-1〜SC-6 解決後 → 実装フェーズで application/scaffold.py
