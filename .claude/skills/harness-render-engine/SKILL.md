---
name: "harness-render-engine"
description: "document.json の x-render テンプレートをもとに Markdown・HTML 形式でレンダリングし、人間が読める形式の成果物を生成する。"
---

# harness-render-engine

---

## 目的

document.json の x-render テンプレートをもとに Markdown・HTML 形式でレンダリングし、人間が読める形式の成果物を生成する。

---

## 役割

- document.json の schemaRef からパッケージ内の Schema を schema_repository（importlib.resources）で特定する
- Schema の x-render-order に従ってブロックの出力順を決定する
- 各ブロックの x-render-level に従って見出しレベルを動的付与する（step_h / substep_h 等）
- Jinja2 テンプレート（x-render.md / x-render.html）を展開してコンテンツを生成する
- Schema の x-render-target.path（canonical）に書き込む。frontmatter は x-frontmatter + engine エンベロープで生成する

---

## インターフェース

**入力は自然言語のテキスト（要望）**。受け取ったテキストから下表の **パラメータに成型** し、『呼び出し』の形に当てはめて呼ぶ（実行手順参照）。**出力は `{ prompt, value }`**（`value` はそのまま・`prompt` が読み方）。

### パラメータ（テキストから成型して埋める・呼び出しに渡す値）

| name | type | 必須 | 説明 | 例 |
|---|---|---|---|---|
| documentPath | string | ✓ | レンダリング対象 document.json のパス。要望テキストで指定される | .has-udd/documents/skills/harness-query-engine.json |
| format | string | - | 出力形式の上書き（既定は x-render-target.formats） | md |
| noDeploy | boolean | - | true で canonical のみ生成・deploy 抑制 | true |

### 出力（value の中身）

| name | type | 必須 | 説明 | 例 |
|---|---|---|---|---|
| prompt | string/null | ✓ | 次アクションの指針（正常時 null・警告時は通知） | (正常時 null・警告時のみ通知文) |
| value | object | ✓ | { renderedPaths: string[] } 生成された canonical パス一覧（そのまま返る） | { "renderedPaths": [".has-udd/skills/harness-query-engine/SKILL.md"] } |

---

## 呼び出し

選んだ operation と成型したパラメータを、下記の CLI / MCP の形に当てはめて呼ぶ（各パラメータの意味は『インターフェース』参照）。

### Skills（CLI）

```
uv run --project waffle waffle render --path <document.json> [--no-deploy]
```

例:

```
uv run --project waffle waffle render --path .has-udd/documents/skills/harness-query-engine.json
```

### MCP

```
render_document({ "path": "<document.json>", "deploy": true })
```

例:

```
render_document({"path": ".has-udd/documents/skills/harness-query-engine.json"})
```

MCP は uv run --project waffle waffle serve 起動後に利用可。

---

## 実行手順

### Step 1: 要望テキストから対象 document を読み取る

documentPath（.has-udd/documents/{type}/{id}.json）は要望テキストで指定される前提。無ければ『対象の指定が必要』と返す。

### Step 2: オプションを成型する

形式上書き(format)や canonical のみ(noDeploy)の指定が要望にあれば params に反映（『インターフェース』参照）。通常は documentPath だけでよい。

### Step 3: CLI / MCP で呼ぶ

成型した params で呼ぶ（『呼び出し』参照）。canonical 書き込みと deploy は engine が行う。

### Step 4: 返り値 {prompt, value} を使う

`value.renderedPaths` が生成パス一覧。`prompt` は通知。AI は中身を再生成せずこのパスを参照する。

### Step 5: 警告・エラーに対処する

`schemaRef` 欠落 / Schema 不在 / `x-render-target` 不在はエラー。`x-render` 未定義の blockType は `skipped` に記録され通知される（中断しない）。

---

## ガードレール

- 対象 document は呼び出し側が要望テキストで指定する。無ければ実行しない
- 書き込みは canonical（x-render-target.path）配下のみ。パストラバーサル禁止
- AI は成果物の中身を再生成せず、engine が返す renderedPaths を参照する
- x-render 未定義の blockType はスキップし通知する（中断しない）
- 例外は握りつぶさず { error, prompt, message } で返す

---

## 参照

- `src/waffle/domain/model/SkillSchema/v1.json`: Skill document の Schema 定義（x-render / x-render-target / x-frontmatter の参照元・パッケージ内）
- `docs/brainstorm/design-engine-render.md`: render engine 設計ブレスト（R-1〜R-8）
