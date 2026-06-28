# harness-render-engine 設計ブレスト

## 目的

`harness-render-engine` の document.json を Python 実装レベルで設計する。
核心は以下の3つ：
1. **ドキュメントスケルトンテンプレート**（出力 MD / HTML の骨格）
2. **レンダリングアルゴリズム**（Schema の x-render-* 情報を使った Jinja2 展開処理）
3. **バリデーションアルゴリズム**（Schema による document.json の検証処理）

---

## 前提（既存ブレスト・Schema から確認済み）

### Jinja2 統一（確定）

`design-schema-and-engine-skills.md` 論点6-E にて確定済み。

- harness-render-engine（x-render テンプレート）は **Jinja2** で展開する
- テンプレートは Schema の `$defs` 内に JSON インラインで定義済み（`x-render.md` / `x-render.html`）
- Handlebars 記法（`{{#each}}`）は誤り。`{% for item in block.items %}` が正しい

### 見出しレベルの動的解決（確定）

`design-schema-and-engine-skills.md` 論点6-E より確定済み。

- **テンプレート（x-render.md / x-render.html）はボディ部分のみ担う**
- **見出しは harness-render-engine が x-render-level から動的に付与する**
- テンプレート内に `##` や `<h2>` をハードコードしない

```
x-render-level: 2 の場合:
  ブロック見出し   → MD: ## / HTML: <h2>
  Step 見出し     → MD: ### / HTML: <h3>  （level + 1）
  SubStep 見出し  → MD: #### / HTML: <h4> （level + 2）
```

→ **これは Jinja2 テンプレートに変数として渡す**

```python
# engine が計算してテンプレートに渡す変数
render_context = {
    "block":         doc["content"][blockKey],     # ブロックデータ
    "step_h":        "#" * (level + 1),            # Step 見出しマーカー
    "substep_h":     "#" * (level + 2),            # SubStep 見出しマーカー
    "step_tag":      f"h{level + 1}",              # Step HTMLタグ名
    "substep_tag":   f"h{level + 2}",              # SubStep HTMLタグ名
}
jinja2.Template(x_render_md).render(**render_context)
```

→ **現在の Schema テンプレートは `step_h` / `substep_h` 変数を使っていない**
→ テンプレートを更新する必要がある（R-3 で確定する）

### 定義済み x-render テンプレート一覧（SkillSchema/v1.json $defs）

| $def 名 | blockType | order | level | x-render.md（概要） |
|---|---|---|---|---|
| TitleBlock            | Title            | 0 | 1 | `{{ doc.content.purpose.text }}` （h1 = documentId） |
| PurposeBlock          | Purpose          | 1 | 2 | `{{ block.text }}` |
| RoleBlock             | Role             | 2 | 2 | `{% for item in block.items %}\n- {{ item }}\n{% endfor %}` |
| InterfaceBlock        | Interface        | 3 | 2 | 入力・出力の Markdown テーブル |
| ProcessingTargetBlock | ProcessingTarget | 3 | 2 | `### 処理対象\n{{ block.target }}\n### 成果物\n{{ block.artifact }}` |
| InvocationSpecBlock   | InvocationSpec   | 4 | 2 | `### Skills モード\n...\n### MCP モード\n...` |
| EngineStepsBlock      | Steps（engine）  | 5 | 2 | step_h + step.title + step.body + children ループ |
| CustomStepsBlock      | Steps（custom）  | 5 | 2 | 同上 |
| GuardrailsBlock       | Guardrails       | 6 | 2 | `{% for item in block.items %}\n- {{ item }}\n{% endfor %}` |
| ReferencesBlock       | References       | 7 | 2 | `` - `{{ item.path }}` — {{ item.description }} `` |

**blockType → $def マッピング規則:**
- 通常: `blockType + "Block"` → 例: `"Purpose"` → `"PurposeBlock"`
- Steps のみ例外: `skillKind == "engine"` → `"EngineStepsBlock"` / `"custom"` → `"CustomStepsBlock"`

---

## 論点一覧

| # | 論点 | 状態 |
|---|---|---|
| R-1 | SKILL.md フロントマターの設計 | ✅ CLOSED |
| R-2 | ドキュメントスケルトン（MD / HTML 全体の骨格） | ✅ CLOSED |
| R-3 | Step / SubStep テンプレートの見出し変数設計 | ✅ CLOSED |
| R-4 | バリデーションアルゴリズム | ✅ CLOSED |
| R-5 | レンダリングアルゴリズム全体設計 | ✅ CLOSED |
| R-6 | Steps 詳細設計（Python 実装粒度） | ✅ CLOSED |
| R-7 | ガードレール | ✅ CLOSED |
| R-8 | 出力ルーティング（documentType → 出力先・骨格・deploy） | ✅ CLOSED |

---

## 論点 R-1: SKILL.md フロントマターの設計 ✅ CLOSED

→ 詳細は `design-schema-and-engine-skills.md` の「SKILL.md フロントマター設計」セクション参照

**確定内容:**
```yaml
---
name: <doc["documentId"]>
description: <doc["content"]["purpose"]["text"]>
---
```
`version` は Claude Code 公式非サポートのため含めない。

---

## 論点 R-2: ドキュメントスケルトン（MD / HTML 全体の骨格） ✅ CLOSED

### 合意内容

**TitleBlock を `$defs` に定義し `content.title` として document.json に持つ。MD / HTML 両方 h1 あり。見出しレベル体系を完全統一する。**

**TitleBlock の位置づけ:**
- `x-render-order: 0`（全ブロック中最初）
- `x-render-level: 1`（h1 / `#`）
- 既存の「content をループ → $def を引く → Jinja2 展開」と**同じアルゴリズム**で処理される
- エンジンに特別処理なし。x-render-order: 0 でソートされ先頭に来るだけ

**見出しレベル体系（MD / HTML 完全統一）:**

| レベル | MD | HTML | 用途 |
|---|---|---|---|
| 1 | `#` | `<h1>` | ドキュメントタイトル（TitleBlock） |
| 2 | `##` | `<h2>` | ブロック見出し（x-render-level=2） |
| 3 | `###` | `<h3>` | Step 見出し（level+1） |
| 4 | `####` | `<h4>` | SubStep 見出し（level+2） |

**出力イメージ（MD）:**
```markdown
---
name: harness-query-engine
description: ...（Claude Code が読むメタデータ）
---

# harness-query-engine

document.json に対してセマンティッククエリを実行し構造化データを返す

## 目的

...

## 役割

...
```

**出力イメージ（HTML）:**
```html
<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="UTF-8">
  <title>harness-query-engine</title>
</head>
<body>
  <section>
    <h1>harness-query-engine</h1>
    <p>document.json に対して...</p>
  </section>
  <section>
    <h2>目的</h2>
    ...
  </section>
</body>
</html>
```

**合意根拠:**
- frontmatter は視覚的に見えないため h1 でタイトルを出す
- 計算ロジックを MD / HTML で統一するため両方 h1 を持つ
- $defs に TitleBlock を定義することで仕組みを同一に保ち、アルゴリズムをシンプルに維持する

**Schema への影響:**
- `$defs` に `TitleBlock` を追加
- document.json の `content` に `"title": { "blockType": "Title", "title": "<documentId>" }` を必須フィールドとして追加

---

## 論点 R-3: Step / SubStep テンプレートの見出し変数設計 ✅ CLOSED

### 合意内容

**エンジンが計算した見出しマーカーを Jinja2 変数として全ブロックに一律で渡す。テンプレートはそれを使うだけ。**

**render_context（全ブロック共通）:**
```python
render_context = {
    "doc":         doc,
    "block":       block,
    "step_h":      "#" * (level + 1),   # MD: level=2 → "###"
    "substep_h":   "#" * (level + 2),   # MD: level=2 → "####"
    "step_tag":    f"h{level + 1}",     # HTML: level=2 → "h3"
    "substep_tag": f"h{level + 2}",     # HTML: level=2 → "h4"
}
```

PurposeBlock 等は `step_h` を使わないが Jinja2 は未使用変数を無視するため問題なし。エンジンに blockType 判定ロジックが生まれない。

**EngineStepsBlock / CustomStepsBlock テンプレート更新（MD）:**
```jinja2
{% for step in block.items %}
{{ step_h }} Step {{ loop.index }}: {{ step.title }}

{% if step.body %}{{ step.body }}
{% endif %}{% if step.children %}{% for sub in step.children %}
{{ substep_h }} {{ sub.title }}

{{ sub.body }}
{% endfor %}{% endif %}{% endfor %}
```

**EngineStepsBlock / CustomStepsBlock テンプレート更新（HTML）:**
```jinja2
{% for step in block.items %}
<article>
  <{{ step_tag }}>Step {{ loop.index }}: {{ step.title }}</{{ step_tag }}>
  {% if step.body %}<p>{{ step.body }}</p>{% endif %}
  {% if step.children %}
  <div class="substeps">
    {% for sub in step.children %}
    <section>
      <{{ substep_tag }}>{{ sub.title }}</{{ substep_tag }}>
      <p>{{ sub.body }}</p>
    </section>
    {% endfor %}
  </div>
  {% endif %}
</article>
{% endfor %}
```

**合意根拠:**
- テンプレートが x-render-level に依存しない（level が変わっても変更不要）
- エンジンに blockType 固有の条件分岐が生まれない（汎用性を保つ）
- 「テンプレートはボディのみ担う・見出しマーカーはエンジンが計算する」原則と整合

**Schema への影響:**
- SkillSchema/v1.json の EngineStepsBlock / CustomStepsBlock の x-render.md / x-render.html を上記テンプレートに更新（ブレスト完了後に一括実施）

---

## 論点 R-4: バリデーションアルゴリズム ✅ CLOSED

### 合意内容

**レンダリング前に `Draft202012Validator` で検証する（fail fast）。エラー・警告を問わず `prompt` フィールドを返し AI が次のアクションを判断できるようにする。**

#### 全 harness-*-engine 共通原則（R-4 で確定）

> エラーが起きたとき、AI がそれを読んで次のアクションを考えられる `prompt` を戻り値に含める。
> Python が「何が起きたか」を確定し、AI が「どうするか」を判断する。

**戻り値フォーマット（全エンジン統一）:**

| ケース | フォーマット |
|---|---|
| 正常完了 | `{ "prompt": null, "value": { ... } }` |
| 警告あり（部分成功） | `{ "prompt": "〜のためスキップしました。〜を確認してください。", "value": { ..., "skipped": [...] } }` |
| エラー（中断） | `{ "error": "ERROR_CODE", "prompt": "〜が原因で失敗しました。〜を確認してください。", "message": "..." }` |

#### バリデーション3段階

```
Stage 1: JSON パース確認
  → json.load() 失敗 → INVALID_JSON エラー・prompt 付きで即終了

Stage 2: jsonschema バリデーション（構造・型チェック）
  → Draft202012Validator(schema).iter_errors(doc) で全エラーを収集
  → $defs / $ref / if/then/else は自動解決
  → 1件でもあれば VALIDATION_ERROR + prompt 付きで終了（全件収集してからまとめて返す）

Stage 3: x-render 整合性確認（警告どまり）
  → content の各 blockType に対応する $def が存在するか
  → その $def に x-render 定義があるか
  → ない場合 → skipped に追記してレンダリング続行・最後に prompt 付きで返す
```

**Stage 3 が警告どまりの理由:**
将来の blockType 追加時、x-render テンプレートが未定義でも既存ドキュメントのレンダリングが止まらないようにする（拡張に対してオープン）。

#### 戻り値の具体例

警告ありの場合:
```json
{
  "prompt": "以下のブロックは x-render 定義が Schema に存在しないためスキップしました。\n- TitleBlock（blockType: Title）\n意図した blockType か確認してください。Schema に x-render を追加するか document.json の blockType を修正することを検討してください。",
  "value": {
    "renderedPaths": ["docs/harness-query-engine.md"],
    "skipped": [{ "blockKey": "title", "blockType": "Title", "reason": "x-render 定義なし" }]
  }
}
```

正常完了:
```json
{
  "prompt": null,
  "value": {
    "renderedPaths": ["docs/harness-query-engine.md", "docs/harness-query-engine.html"]
  }
}
```

---

## 論点 R-5: レンダリングアルゴリズム全体設計 ✅ CLOSED

> **R-8 による更新（このコード例は一部 supersede）**: schema 解決は `.has-udd/schemas/` ではなく **パッケージ（importlib.resources / schema_repository）**。出力先は `output_dir="docs/"` フラットではなく **schema の `x-render-target.path`**。frontmatter ハードコードは **`x-frontmatter` + engine エンベロープ**。下記コードは骨子（ブロック収集・Jinja2 展開・skipped・prompt 戻り値）の参照用で、パス/出力部は R-8 が正。

### 合意内容

```python
import json, os
import jinja2
from jsonschema import Draft202012Validator

def render(doc_path: str, format: str = "both", output_dir: str = "docs/"):
    try:
        # Stage 1: 読み込み
        doc    = json.load(open(doc_path))
        schema = schema_repository.load(doc["schemaRef"])   # パッケージ内 src/has_udd/domain/model/{schemaRef}.json（importlib.resources）

        # Stage 2: jsonschema バリデーション（R-4）
        errors = list(Draft202012Validator(schema).iter_errors(doc))
        if errors:
            detail = "\n".join(f"- {list(e.absolute_path)}: {e.message}" for e in errors)
            return {
                "error":   "VALIDATION_ERROR",
                "prompt":  f"document.json のバリデーションに失敗しました。以下を修正してください。\n{detail}",
                "message": detail,
            }

        # Stage 3: ブロック収集・x-render 整合性確認・ソート
        blocks, skipped = [], []
        for key, block in doc["content"].items():
            def_name  = block["blockType"] + "Block"   # get_def_name() 不要（R-3）
            block_def = schema["$defs"].get(def_name)
            if not block_def or "x-render" not in block_def:
                skipped.append({"blockKey": key, "blockType": block["blockType"], "reason": "x-render 定義なし"})
                continue
            level = block_def["x-render-level"]
            blocks.append({
                "key": key, "block": block,
                "order":       block_def["x-render-order"],
                "heading_md":  "#" * level,
                "heading_tag": f"h{level}",
                "step_h":      "#" * (level + 1),
                "substep_h":   "#" * (level + 2),
                "step_tag":    f"h{level + 1}",
                "substep_tag": f"h{level + 2}",
                "tpl_md":      block_def["x-render"]["md"],
                "tpl_html":    block_def["x-render"]["html"],
            })
        blocks.sort(key=lambda b: b["order"])

        # Stage 4: Jinja2 展開
        sections_md, sections_html = [], []
        for b in blocks:
            ctx = {
                "doc":         doc,
                "block":       b["block"],
                "step_h":      b["step_h"],
                "substep_h":   b["substep_h"],
                "step_tag":    b["step_tag"],
                "substep_tag": b["substep_tag"],
            }
            if format in ("md", "both"):
                heading = f"{b['heading_md']} {b['block']['title']}"
                body    = jinja2.Template(b["tpl_md"]).render(**ctx).strip()
                sections_md.append(f"{heading}\n\n{body}")
            if format in ("html", "both"):
                tag  = b["heading_tag"]
                body = jinja2.Template(b["tpl_html"]).render(**ctx).strip()
                sections_html.append(
                    f"  <section>\n    <{tag}>{b['block']['title']}</{tag}>\n    {body}\n  </section>"
                )

        # Stage 5: スケルトン組み立て・書き出し
        # TitleBlock（x-render-order: 0）がソートで先頭に来るため追加処理不要
        doc_id       = doc["documentId"]
        purpose_text = doc["content"].get("purpose", {}).get("text", "")

        os.makedirs(output_dir, exist_ok=True)
        rendered = []
        if format in ("md", "both"):
            md_out = f"---\nname: {doc_id}\ndescription: {purpose_text}\n---\n\n"
            md_out += "\n\n".join(sections_md)
            p = os.path.join(output_dir, f"{doc_id}.md")
            open(p, "w").write(md_out)
            rendered.append(p)
        if format in ("html", "both"):
            html_out = (
                f'<!DOCTYPE html>\n<html lang="ja">\n<head>\n'
                f'  <meta charset="UTF-8">\n  <title>{doc_id}</title>\n</head>\n<body>\n'
                + "\n".join(sections_html)
                + "\n</body>\n</html>"
            )
            p = os.path.join(output_dir, f"{doc_id}.html")
            open(p, "w").write(html_out)
            rendered.append(p)

        # Stage 6: 戻り値（R-4 原則：prompt で AI に次のアクションを伝える）
        if skipped:
            names = "\n".join(f"- {s['blockKey']}（{s['blockType']}）" for s in skipped)
            return {
                "prompt": f"以下のブロックは x-render 定義が Schema に存在しないためスキップしました。\n{names}\nSchema に x-render を追加するか blockType を確認してください。",
                "value":  {"renderedPaths": rendered, "skipped": skipped},
            }
        return {"prompt": None, "value": {"renderedPaths": rendered}}

    except Exception as e:
        return {
            "error":   "RENDER_ERROR",
            "prompt":  f"レンダリング中に予期しないエラーが発生しました。詳細を確認してください: {e}",
            "message": str(e),
        }
```

**設計上のポイント:**
- `blockType + "Block"` のみで $def 解決（`get_def_name()` 不要・R-3 案C 反映）
- `doc` も render_context に含める（TitleBlock テンプレートが `{{ doc.documentId }}` を参照）
- TitleBlock は order=0 でソート先頭に来るだけ。特別処理なし（R-2 反映）
- x-render-order 同値（InterfaceBlock と ProcessingTargetBlock が共に order=3）は engine / custom で排他的に存在するため衝突しない

---

## 論点 R-6: Steps 詳細設計（Python 実装粒度） ✅ CLOSED

### 合意内容

```
Step 1: document.json と Schema を読み込む
  SubStep 1-1: doc_path の document.json を json.load() する
  SubStep 1-2: doc["schemaRef"] を schema_repository（importlib.resources）でパッケージ内 src/has_udd/domain/model/{schemaRef}.json として解決する（.has-udd に schema は無い）

Step 2: jsonschema でバリデーションする
  SubStep 2-1: Draft202012Validator(schema).iter_errors(doc) で全エラーを収集する
  SubStep 2-2: エラーがある場合は { error, prompt, message } を返して中断する

Step 3: レンダリング対象ブロックを収集・ソートする
  SubStep 3-1: doc["content"] の各 blockKey に対して blockType + "Block" で $def を特定する
  SubStep 3-2: $def に x-render 定義がない場合は skipped に追記してスキップする
  SubStep 3-3: x-render-level から見出しマーカー（heading_md / step_h / substep_h 等）を計算する
  SubStep 3-4: x-render-order の昇順でソートする（同値の場合は blockKey のアルファベット順）

Step 4: 各ブロックを Jinja2 テンプレートで展開する
  SubStep 4-1: format に応じて x-render.md / x-render.html を選択する
  SubStep 4-2: { doc, block, step_h, substep_h, step_tag, substep_tag } を render_context として渡す
  SubStep 4-3: jinja2.Template(tpl).render(**render_context) で本文を生成する
  SubStep 4-4: 見出し（heading_md / heading_tag）と本文を結合してセクションを組み立てる

Step 5: スケルトンに組み込んで出力ファイルを書き出す
  SubStep 5-1: MD はフロントマター（name / description）+ セクション結合
  SubStep 5-2: HTML は DOCTYPE + head + body + セクション結合
  SubStep 5-3: outputDir が存在しない場合は os.makedirs() で自動作成する
  SubStep 5-4: {documentId}.md / {documentId}.html を書き込み renderedPaths を収集する

Step 6: 戻り値を組み立てて返す
  SubStep 6-1: skipped がある場合は prompt に警告メッセージを設定して返す
  SubStep 6-2: skipped がない場合は { prompt: null, value: { renderedPaths } } を返す
  SubStep 6-3: 例外が発生した場合は { error, prompt, message } を返す（try/except で全体をラップ）
```

---

## 論点 R-7: ガードレール ✅ CLOSED

### 合意内容

全エラー・警告に `prompt` を付与し AI が次のアクションを判断できるようにする（R-4 共通原則を適用）。

**カテゴリ1: 入力バリデーション**
- `doc_path` が存在しない場合は `INVALID_PATH` + prompt で返す
- document.json が有効な JSON でない場合は `INVALID_JSON` + prompt で返す
- `doc["schemaRef"]` が存在しない場合は `MISSING_SCHEMA_REF` + prompt で返す
- `format` が `"md"` / `"html"` / `"both"` 以外の場合は `INVALID_FORMAT` + prompt で返す

**カテゴリ2: Schema 解決**
- `schemaRef` に対応する Schema ファイルが存在しない場合は `SCHEMA_NOT_FOUND` + prompt で返す
- jsonschema バリデーションでエラーが検出された場合は `VALIDATION_ERROR` + prompt で返す（全件収集してから返す）
- `$def` に x-render 定義がない blockType は skipped に追記して継続・最後に prompt 付き部分成功として返す

**カテゴリ3: レンダリング**
- Jinja2 テンプレート展開に失敗した場合はそのブロックを skipped に追記して継続する（全体を止めない）
- `output_dir` への書き込みに失敗した場合は `WRITE_ERROR` + prompt で返す
- `output_dir` が存在しない場合は `os.makedirs()` で自動作成する（エラーにしない）

**カテゴリ4: Harness 原則**
- Python スクリプトが実行する。AI が直接 content を読んでレンダリングしてはならない
- すべての例外を `try/except` で捕捉し `{ error, prompt, message }` 形式で返す（例外を AI に素通りさせない）
- 書き込み操作は `output_dir` 配下のみに限定する（パストラバーサル禁止）

---

## 論点 R-8: 出力ルーティング（documentType → 出力先・骨格・deploy） ✅ CLOSED

### 背景

汎用 render engine が documentType ごとに違う出力先・形式・骨格をどう解決するか。フォルダ構成確定（`.has-udd/` canonical・schema 非配布）で R-5 の `output_dir="docs/"` フラットが乖離したため新設。

### 合意内容

#### 1. 出力ルーティング = schema ルートの `x-render-target`

「ドメイン知識は schema に・engine は汎用」の原則どおり、出力先は schema が宣言。

```json
// SkillSchema/v1.json ルート（確定）
"x-render-target": {
  "formats": ["md"],
  "path":    ".has-udd/skills/{documentId}/SKILL.md"   // canonical のみ
}
```

| schema | formats | path（Skill 確定・他は仮決め） |
|---|---|---|
| SkillSchema | `["md"]` | `.has-udd/skills/{documentId}/SKILL.md` ✅確定 |
| AgentSchema | `["md"]` | `.has-udd/agents/{documentId}.md` ⚠仮 |
| UsecaseSpecSchema | `["html"]` | `.has-udd/specs/{documentId}.html` ⚠仮 |
| DomainModelSpecSchema | `["html"]` | `.has-udd/specs/{documentId}.html` ⚠仮 |
| KnowledgeSchema | `["html"]` | `.has-udd/knowledge/{documentId}.html` ⚠仮 |
| CodingSchema | 特殊 | 実コードへ DocComment 注入（template-engine ブレストで別途） |

engine は type→path の表を Python に持たない。schema が宣言、engine は実行するだけ。

#### 2. ドキュメント骨格（R-8a）= エンベロープは engine・frontmatter フィールドのみ schema

**x-render-skeleton 案は撤回（保守性 NG）**。理由: HTML ラッパー等の format エンベロープを全 schema に重複させ DRY 違反・インフラを schema に混入。

分離:
| 部分 | 性質 | 置き場 |
|---|---|---|
| HTML ラッパー（doctype/head/body・`<title>`={{documentId}}） | format インフラ・全 schema 共通 | **engine（HtmlRenderer・1箇所）** |
| MD frontmatter の `---` 区切り | format インフラ・共通 | **engine（MdRenderer・1箇所）** |
| frontmatter の中身フィールド | schema 固有・ドメイン | **schema の `x-frontmatter`** |

```json
// schema が宣言するのはフィールド写像だけ（MD frontmatter を持つ型のみ）
"x-frontmatter": {
  "name":        "{{ doc.documentId }}",
  "description": "{{ doc.content.purpose.text }}"
}
```
engine: `x-frontmatter` を読み → `---\nkey: value\n---` で包む（MdRenderer）。HTML 型は frontmatter 不要。TitleBlock（h1）は R-2 通り通常ブロック。

#### 3. deploy（R-8b）= render は canonical を書くだけ・symlink 基本

**前案「全部コピー」は撤回。**

| ケース | 方式 | render 後 |
|---|---|---|
| 同形式・同ファイル名のツール | **symlink**（init 時に1回） | render→canonical 書き込みで自動反映・コピー不要 |
| 形式・ファイル名が違うツール | **変換 deploy** | ツール形式に合わせ別途生成（Phase 6） |

- render の責務 = **canonical 書き込みのみ**
- 同形式ツール反映 = init の symlink（config.json の有効ツール × 規約で張る・自動反映）
- 異形式ツール = 変換 deploy（形式差があるときだけ・Phase 6）
- SSOT = `.has-udd/documents/` の document.json。canonical rendered は派生
- symlink caveat: ドキュメント化されていない（=未明記、壊れる確証ではない）。基本 symlink・辿れないツールのみコピーにフォールバック

### 合意

| 項目 | 決定 |
|---|---|
| 出力ルーティング | schema ルート `x-render-target`（formats / path）。Skill 確定・他仮決め |
| 骨格（R-8a） | エンベロープ=engine（1箇所）／ frontmatter フィールド=schema の `x-frontmatter`。x-render-skeleton 撤回 |
| deploy（R-8b） | render は canonical 書き込みのみ。同形式=init symlink・異形式=変換 deploy（Phase 6） |
| Coding | 特殊・template-engine ブレストで別途 |

### R-5 への影響（修正必要）

- `output_dir="docs/"` フラット → `x-render-target.path` 駆動に変更
- schema 解決 `.has-udd/schemas/...` → パッケージ（importlib.resources / schema_repository）に変更
- frontmatter ハードコード → `x-frontmatter` + engine エンベロープに変更

---

## 合意事項

| # | 合意内容 |
|---|---|
| R-1 | SKILL.md フロントマター = `name` + `description` のみ。`version` は Claude Code 非サポートのため除外 |
| R-2 | TitleBlock を `$defs` に追加（order=0 / level=1）。MD/HTML 両方 h1 あり。見出しレベル体系を完全統一 |
| R-3 | `blockType + "Block"` のみで $def 解決（`get_def_name()` 廃止）。`"Steps"` → `"EngineSteps"` / `"CustomSteps"` に分割。見出しマーカーを Jinja2 変数（step_h 等）で渡す |
| R-4 | 全エラー・警告に `prompt` フィールドを付与。Python が「何が起きたか」を確定し AI が「どうするか」を判断する（全エンジン共通原則） |
| R-5 | レンダリングアルゴリズム確定（6 Stage）。`doc` も render_context に含める |
| R-6 | Steps 6 フェーズ設計確定（SubStep 粒度まで） |
| R-7 | ガードレール 4 カテゴリ確定。全エラーに prompt 付与（R-4 原則適用） |
| R-8 | 出力ルーティング = schema の `x-render-target`（Skill 確定・他仮）／ 骨格 = engine エンベロープ + schema の `x-frontmatter`（x-render-skeleton 撤回）／ deploy = render は canonical のみ・同形式 symlink・異形式変換（Phase 6） |

---

## 次のアクション（影響ファイル一括更新）

**注: 旧パス前提（`.has-udd/schemas/` 等）は最新フォルダ構成で supersede 済み。schema はパッケージ内 `src/has_udd/domain/model/`。**

| ファイル | 変更内容 | 状態 |
|---|---|---|
| SkillSchema/v1.json | TitleBlock / EngineSteps/CustomSteps / Steps テンプレート | ✅ 適用済み |
| harness-query-engine.json / harness-render-engine.json / analyze-domain-model.json | title ブロック / blockType 更新 | ✅ 適用済み |
| **R-8 由来の未適用**: SkillSchema に `x-render-target` + `x-frontmatter` 追加 / R-5 アルゴリズムの schema 解決・出力先・frontmatter を修正 | 実装フェーズ or 乖離修正で対応 | ⬜ 未 |
| **乖離修正（query/render 監査）**: schema パス `.has-udd/schemas/` → パッケージ / document パス → `.has-udd/documents/` | ブレスト・json 反映済み | ✅ 適用済み |
