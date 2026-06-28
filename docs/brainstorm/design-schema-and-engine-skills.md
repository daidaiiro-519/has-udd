# ブレインストーミング: Schema / harness-query-engine / harness-render-engine 設計

**目的:** schema（JSON Schema, ドメインモデル）の構造を設計し、harness-query-engine のアクセスパターン・harness-render-engine の出力パターン（MD / HTML / DocComment）を確定する
**モード:** アイデア発散 → 設計収束
**前提:** `brainstorm-has-udd-design.md` で確定した集約設計・ユビキタス言語・engine Skills（harness-*-engine）優先順位を引き継ぐ

---

## 設計の順序と関係

```
① Schema（JSON Schema, ドメインモデル）を設計する   ← 今ここ
     バリデーション定義
   + AI への指示（x-prompt）
   + レンダリング構造（x-render: MD / HTML / DocComment）
   を1ファイルに統合した SSOT

       ↓ Schema に従ってランタイムで生成される

② document.json（Schema のインスタンス）
   AI が Schema を読んで値を入れたもの
   harness-query-engine / harness-render-engine が実際に扱う対象

       ↓ document.json を読み書きする基盤

③ harness-query-engine / harness-render-engine の設計
   Schema が確定してから詰める
```

**設計の順序: Schema → document.json の構造 → harness-query-engine / harness-render-engine**
実装の優先順位（query/render engine 先）とは別の話。

---

## 論点 1: Schema の基本構造をどう定義するか ✅ CLOSED

**決定:** Schema の基本構造・document.json の最終構造を以下の通り確定する。

---

#### Schema のアノテーション体系（確定）

消費者 × フェーズ の2軸で整理する。

```
# AI 向け（x-prompt-* 系）― 誰が読むか: AI

x-prompt-write    → AI が document.json に「値を書く」ときの指示（命令形）
                    ※ AI が空の document.json に値を埋める際に Schema から参照する

x-prompt-query    → AI が document.json を「読む・クエリする」ときの説明（説明形）
                    query-engine / knowledge-engine が読み取り時に動的にインデックスの
                    prompt を生成するために参照する（document.json には保存しない）
                    ※ _index はどこにも保存されない。読み取り時に schema の x-prompt-query から算出する

x-prompt-template → AI が CodingTemplate（.j2）を「作成する」ときの指示
                    テンプレートでこの変数をどう使うかを説明する
                    ※ CodingSpec Schema にのみ存在

# スクリプト向け（x-render-* 系）― 誰が読むか: has-udd スクリプト

x-render          → Jinja2 レンダリングテンプレート（md / html / docComment）
x-render-order    → ブロックの出力順（JSON の properties は順序保証なし）
x-render-level    → 見出しレベル（旧 x-document-structure.headingLevel）
```

| アノテーション | タイミング | 消費者 |
|---|---|---|
| `x-prompt-write` | document.json 作成時 | AI |
| `x-prompt-query` | クエリ・読み取り時 | AI |
| `x-prompt-template` | CodingTemplate 作成時 | AI |
| `x-render` | レンダリング時 | スクリプト |
| `x-render-order` | レンダリング時 | スクリプト |
| `x-render-level` | レンダリング時 | スクリプト |

#### Schema の具体構造（`UsecaseSpecSchema/v1.json`）

```json
{
  "$schema": "https://has-udd.dev/schemas/meta/v1",
  "title": "UsecaseSpec",
  "x-render-order": ["overview", "acceptanceCriteria", "domainEvents"],
  "properties": {
    "overview": {
      "type": "object",
      "required": ["blockType", "title", "summary"],
      "properties": {
        "blockType": { "type": "string", "const": "Overview" },
        "title":     { "type": "string" },
        "summary":   { "type": "string" }
      },
      "x-prompt":       "このユースケースの目的と概要を1〜2文で記述してください。",
      "x-prompt-query": "ユースケースの目的と概要を持ちます。実装全体の出発点として使います。",
      "x-render": {
        "md":   "## {{block.title}}\n{{block.summary}}",
        "html": "<section><h2>{{block.title}}</h2><p>{{block.summary}}</p></section>"
      }
    },
    "acceptanceCriteria": {
      "type": "object",
      "required": ["blockType", "title", "items"],
      "properties": {
        "blockType": { "type": "string", "const": "AcceptanceCriteria" },
        "title":     { "type": "string" },
        "items":     { "type": "array", "items": { "type": "string" } }
      },
      "x-prompt":       "何ができれば完了かをユーザー視点でテスト可能な粒度で列挙してください。",
      "x-prompt-query": "受け入れ条件ブロック。何ができれば完了かの判断基準を持ちます。テスト設計・レビューの入力として使います。",
      "x-render": {
        "md":   "## {{block.title}}\n{{#each block.items}}- [ ] {{this}}\n{{/each}}",
        "html": "<section><h2>{{block.title}}</h2><ul>{{#each block.items}}<li>{{this}}</li>{{/each}}</ul></section>"
      }
    },
    "domainEvents": {
      "type": "object",
      "required": ["blockType", "title", "items"],
      "properties": {
        "blockType": { "type": "string", "const": "DomainEvents" },
        "title":     { "type": "string" },
        "items":     { "type": "array", "items": { "type": "string" } }
      },
      "x-prompt":       "このユースケースで発生するドメインイベントを列挙してください。",
      "x-prompt-query": "ドメインイベントブロック。このユースケースで発生するイベント一覧を持ちます。イベント駆動実装設計の入力として使います。",
      "x-render": {
        "md":   "## {{block.title}}\n{{#each block.items}}- {{this}}\n{{/each}}",
        "html": "<section><h2>{{block.title}}</h2><ul>{{#each block.items}}<li>{{this}}</li>{{/each}}</ul></section>"
      }
    }
  }
}
```

#### document.json の具体構造（`uc-order-create.json`）

```json
{
  "documentId":   "uc-order-create",
  "documentType": "Spec",
  "schemaRef":    "UsecaseSpecSchema/v1",
  "status":       "VALIDATED",
  "refs":         { "pbiRef": "pbi-order-001" },
  "tags":         ["subdomain:order"],
  "createdAt":    "2026-06-21T00:00:00Z",
  "updatedAt":    "2026-06-21T00:00:00Z",

  "content": {
    "overview": {
      "blockType": "Overview",
      "title":     "注文作成",
      "summary":   "顧客が商品を選択して注文を確定するユースケース。"
    },
    "acceptanceCriteria": {
      "blockType": "AcceptanceCriteria",
      "title":     "受け入れ条件",
      "items": [
        "顧客が商品を選択できる",
        "在庫がない場合は注文不可",
        "注文確定後に OrderCreated イベントが発行される"
      ]
    },
    "domainEvents": {
      "blockType": "DomainEvents",
      "title":     "ドメインイベント",
      "items":     ["OrderCreated", "InventoryReserved"]
    }
  }
}
```

#### Schema と document.json の役割分担（確定）

```
Schema（JSON Schema, ドメインモデル）        document.json（インスタンス）
───────────────────────────────             ─────────────────────────────────
type / required / enum                      documentId / documentType / schemaRef
x-prompt       （書くときの AI 指示）        status / refs / tags
x-prompt-query （読むときの AI 説明）        createdAt / updatedAt
x-render       （レンダリングテンプレート）   content
x-render-order （ブロック出力順）              {blockKey}.blockType
                                              {blockKey}.title
                                              {blockKey}.items / summary / ...（値のみ）

※ _index は document.json に保存しない。query/knowledge engine が読み取り時に
  （document の blockType 群 × schema の x-prompt-query）から動的に算出する。
```

#### Schema バリデーション階層（oneOf + discriminator の使いどころ）

`oneOf` + `discriminator` は Schema ファイル側のバリデーションルールとして使う。document.json 自体の構造は変わらない。

**2段階のルーティング:**

```
Level 1: documentType で大分類（共通メタ Schema）
  discriminator: documentType → Spec / Coding / Agent / Skill / Knowledge

Level 2: schemaRef で具体 Schema にルーティング（Spec 基底 Schema）
  discriminator: schemaRef → UsecaseSpec / DomainModelSpec ...
```

※ schema はパッケージ内（`src/has_udd/domain/model/`）。`.has-udd/schemas/` は存在しない。

```json
// src/has_udd/domain/model/common/meta.json（全ドキュメント共通のメタ Schema）
{
  "discriminator": { "propertyName": "documentType" },
  "oneOf": [
    { "$ref": "spec/base.json" },
    { "$ref": "coding/base.json" },
    { "$ref": "agent/base.json" },
    { "$ref": "skill/base.json" },
    { "$ref": "knowledge/base.json" }
  ]
}

// src/has_udd/domain/model/spec/base.json（Spec 系の基底 Schema）
{
  "allOf": [{ "$ref": "common/DocumentEnvelope" }],
  "properties": {
    "documentType": { "const": "Spec" },
    "status": { "enum": ["CREATED", "VALIDATED", "RENDERED", "SUPERSEDED"] }
  },
  "discriminator": { "propertyName": "schemaRef" },
  "oneOf": [
    { "$ref": "UsecaseSpec/v1.json" },
    { "$ref": "DomainModelSpec/v1.json" }
  ]
}
```

**各 Schema ファイルでの blockType 分岐（content ブロックの構造検証）:**

```json
// UsecaseSpec/v1.json の content バリデーション
"content": {
  "type": "object",
  "additionalProperties": {
    "discriminator": { "propertyName": "blockType" },
    "oneOf": [
      {
        "properties": { "blockType": { "const": "Overview" } },
        "required": ["blockType", "title", "summary"]
      },
      {
        "properties": { "blockType": { "const": "AcceptanceCriteria" } },
        "required": ["blockType", "title", "items"]
      },
      {
        "properties": { "blockType": { "const": "DomainEvents" } },
        "required": ["blockType", "title", "items"]
      }
    ]
  }
}
```

**JSON Schema 機能の使いどころ（確定）:**

| 機能 | has-udd での使いどころ | 層 |
|---|---|---|
| `$defs` + `$ref` | DocumentEnvelope の共通定義・再利用 | Schema |
| `allOf` | DocumentEnvelope + documentType 固有フィールドの合成 | Schema |
| `oneOf` + `discriminator` | documentType → schemaRef → blockType の2段階ルーティング | Schema |
| `if/then/else` | CodingTemplate の templateKind 別必須フィールド分岐 | Schema |
| 上記機能の適用対象 | document.json のバリデーション（document.json 自体には登場しない） | — |

#### document.json の生成フロー（確定）

```
Schema
  ↓ scaffold-engine が properties を走査して空の骨格を生成（_index は生成しない）
空の document.json（エンベロープ + 空の content ブロック）
  ↓ AI が Schema の x-prompt を読んで各 content ブロックに値を記入
  ※ ブロックインデックス（{blockType, prompt}）は保存しない。
    query/knowledge engine が読み取り時に schema の x-prompt-query から動的に算出する。
値が入った document.json（CREATED 状態）
  ↓ has-udd がメタ Schema → documentType → schemaRef の順でルーティングしてバリデーション
VALIDATED 状態の document.json
  ↓ harness-render-engine が schemaRef → Schema の x-render を適用（x-render-order 順）
MD / HTML 出力（RENDERED 状態）
※ Coding documentType のみ docComment も出力
```

---

## 論点 2: harness-query-engine はどんなアクセスパターンを持つべきか

Agent が document.json を読む全パターンを洗い出す。ここで漏れたパターンは実装できない。

### 前提（確定済み）

**フォルダ構成（最新確定・旧記述を supersede）:**

> 原則: `.has-udd/` が canonical（source + rendered）。source（document.json）は `documents/{type}/` に分離・集約フォルダは rendered。schema は配布せずパッケージ内（`src/has_udd/domain/model/`）。ツール固定パスへは deploy（コピー・symlink 非依存）。`_index.json` は廃止（動的集約）。

```
.has-udd/                          # canonical
├── config.json
├── documents/                    # 【SOURCE】document.json（集約ごと・内部）
│   ├── skills/<id>.json
│   ├── specs/<id>.json
│   ├── knowledge/<id>.json
│   ├── agents/<id>.json
│   └── coding/<id>.json
├── skills/<name>/SKILL.md         # 【RENDERED】tool-recognized
├── agents/<name>.md               # 【RENDERED】フラット .md
├── specs/<id>.html                # 【RENDERED】人間向け HTML（集約フォルダ内で完結）
├── knowledge/<id>.html            # 【RENDERED】人間向け HTML
├── hooks/
└── traces/                        # harness-audit-engine .trace.json（.gitignore）

# deploy（コピー・rendered のみ・document.json 除外）→ ツール固定パス
.claude/skills/<name>/SKILL.md     # symlink 非推奨 → 実ファイルコピー
.claude/agents/<name>.md           # Phase 6: .github/ .agents/ にも複製
CLAUDE.md / AGENTS.md              # ルート（Primary Port）
# coding rendered → 実コードファイルへ DocComment 注入（.has-udd 外）
# schema は .has-udd に置かない（パッケージ内 src/has_udd/domain/model/・importlib.resources で解決）
```

**確定事実（claude-code-guide 確認済み）:**
- Skill = `.claude/skills/<name>/SKILL.md`（フォルダ per skill・他ファイル無視）
- Subagent = `.claude/agents/<name>.md`（フラット・非 .md 無視）
- symlink 非ドキュメント化 → rendered は実ファイル deploy
- `_index.json` 廃止（動的集約・フォルダ／ドキュメント両レベル）

※ PBI は外部ツール（Jira / Linear 等）で管理。has-udd は refs.pbiRef で参照のみ。
※ SBI は Sprint Planning ツールで管理。has-udd のスコープ外。

パスに subdomain と documentType が既に含まれるため、フォルダトラバーサルが絞り込みの第一手段になる。

**harness-query-engine の実装方針（確定）:**
- manifest.json は持たない
- 初回クエリ時にファイルスキャン → 結果をインメモリキャッシュに格納。セッション終了でキャッシュは消える
- 100〜200件規模のスキャンは実用上問題なし
- JSON Schema 機能（$defs / discriminator 等）は validation と構造定義に専念。フォルダルーティングには使わない
- has-udd バイナリがロジックを持つ。SKILL.md と Schema のみ init 時にプロジェクトへ配置
- jsonpath-ng は内部実装として使用。AI にパス式を直接渡させない

**フォルダレベルの動的集約（`_index.json` は廃止）:**

`_index.json` ファイルは廃止。フォルダ内ドキュメントの概要（title + summary 一覧）が必要な場合は、harness-query-engine が初回クエリ時のファイルスキャン結果から動的に集約してインメモリで返す（永続インデックスファイルは持たない）。

```
.has-udd/documents/specs/
├── uc-order-create.json
├── uc-order-cancel.json
└── uc-order-confirm.json
  ↑ harness-query-engine がスキャンして title + summary を動的集約（_index.json ファイルは作らない）
```

**動的集約の性質:**
- 永続インデックスファイルを持たない（stale 検知・再生成ロジック不要）
- スキャン結果はセッション限りのインメモリキャッシュ
- AI はクエリ API を呼ぶだけで全ドキュメントの概要を把握できる

**インデックスの位置づけ:**

| レベル | 仕組み | 役割 |
|---|---|---|
| ドキュメント内 | 動的算出（query-engine が document の blockType 群 × schema の x-prompt-query から生成） | 「このドキュメントに何のブロックがあるか」 |
| フォルダ内 | 動的集約（永続ファイルなし） | 「このフォルダに何のドキュメントがあるか」 |

※ ドキュメント内のブロックインデックスは document.json に保存されない。query-engine が読み取り時に schema の x-prompt-query から動的に算出する（`_index.json` ファイル廃止と同じ原則）。

**フェーズクエリ戦略:**
```
Phase 0: フォルダ動的集約（最軽量・複数ドキュメント横断時）
  → フォルダ内の全ドキュメントの title + summary を把握（スキャンから動的生成）
  → 関連する documentId を絞り込む

Phase 1: エンベロープ参照（軽量）
  → documentId / status / schemaRef / refs / tags だけ読む
  → status・refs による追加フィルタ

Phase 2: ブロックインデックス参照（中量）
  → query-engine が document の blockType 群 × schema の x-prompt-query からインデックスを動的算出
  → 「どのブロックキーを取るべきか」を prompt から導出（インデックスは保存されていない）

Phase 3: content 参照（重量）
  → content[blockKey] だけ読む
  → Phase 2 で確定したキーのみ。全 content 読み込みは原則禁止
```

### AI 初期見解

**見解:** 3フェーズ戦略に基づき、ブロックインデックス参照（読み取り時に動的算出）パターンを含めた全アクセスパターンを整理する。

**パターン一覧:**

| # | フェーズ | ユースケース | 操作 | 必要なフィールド |
|---|---|---|---|---|
| 1 | 1 | UsecaseSpec を1件取得 | ID 指定の単件取得 | documentId |
| 2 | 1 | VALIDATED な UsecaseSpec を全件取得 | status × schemaRef でフィルタ | status / schemaRef |
| 3 | 1 | 外部 PBI に紐づく UsecaseSpec 一覧 | refs.pbiRef で逆引き | refs.pbiRef |
| 4 | 1 | 特定 subdomain の UsecaseSpec 一覧 | tags / パスで絞り込み | tags:subdomain |
| 5 | 1 | Subdomain:order の全 Spec | フォルダトラバーサルで `documents/specs/` をスキャン + tag フィルタ | パス + tags |
| 6 | 1 | VALIDATED な UsecaseSpec の件数を数える | 件数集計 | status / schemaRef |
| 7 | 1 | 最新バージョンの DomainModelSpec を取得 | SUPERSEDED 除外 + 最新1件 | status / updatedAt |
| 8 | 1 | CREATED のまま停滞している UsecaseSpec を全件取得 | status でフィルタ | status / schemaRef |
| 9 | 2 | ドキュメントの構造把握 | 単件のブロックインデックスを動的算出 | blockType 群 × x-prompt-query |
| 10 | 2 | 複数ドキュメントのインデックスを一括スキャン | エンベロープ + 動的算出インデックスを一括返す | blockType 群 × x-prompt-query（複数件） |
| 11 | 3 | UsecaseSpec の acceptanceCriteria[] を取得 | content 内ブロックの取得 | content.acceptanceCriteria |
| 12 | 3 | 特定 Role の skillRefs 一覧 | Agent 単件 → content フィールド | content.skillRefs |

**正しいクエリフロー例（subdomain:order の全 UsecaseSpec の acceptanceCriteria を確認）:**
```
❌ 間違い: findBySchema → getContent（全 content を読む）

✅ 正しい:
  Phase 1: findBySchema("UsecaseSpec", {tags: "subdomain:order"})
    → [uc-001, uc-002, uc-003] がヒット（エンベロープのみ）

  Phase 2: scanIndex(["uc-001", "uc-002", "uc-003"])
    → query-engine が各ドキュメントのブロックインデックスを動的算出し "acceptanceCriteria" キーを導出

  Phase 3: getContent("uc-001", "acceptanceCriteria")
           getContent("uc-002", "acceptanceCriteria")
           getContent("uc-003", "acceptanceCriteria")
    → 対象ブロックだけ取得
```

**インターフェース案:**

```
--- Phase 1: エンベロープ参照 ---

harness-query-engine.getById(documentId)
  → 単件のエンベロープを返す。存在しない場合は null

harness-query-engine.findBySchema(schemaRef, filters?)
  → schemaRef で絞り込み。filters で status / tags / refs 等を追加指定
  → エンベロープのみ返す（content / ブロックインデックスは含まない）

harness-query-engine.findByRef(refKey, refValue, schemaRef?)
  → refs フィールドで逆引き（例: pbiRef="pbi-001" な UsecaseSpec[]）
  → エンベロープのみ返す

harness-query-engine.count(schemaRef, filters?)
  → 件数のみ返す

--- Phase 2: ブロックインデックス参照（読み取り時に動的算出） ---

harness-query-engine.getIndex(documentId)
  → 単件のブロックインデックスを動的算出して返す（blockType + prompt の一覧）
  → document の blockType 群 × schema の x-prompt-query から算出（document.json には保存されていない）
  → content 取得前のクエリキー導出に使う

harness-query-engine.scanIndex(documentIds[], filters?)
  → 複数ドキュメントのエンベロープ + 動的算出インデックスを一括返す
  → AI が「どの documentId の何のブロックを読むべきか」を1回で判断できる

--- Phase 3: content 参照 ---

harness-query-engine.getContent(documentId, blockKey)
  → content[blockKey] を返す（blockKey 必須。全 content 返却は禁止）
```

**根拠:**
- Phase 1 はエンベロープだけで解決できる → content / インデックスを読まず高速
- Phase 2 のブロックインデックス参照（動的算出）でクエリキーを確定してから Phase 3 に進む → 全 content 読み込みを防ぐ
- フォルダパスが subdomain × documentType を表現するため、パターン5はフォルダトラバーサルで解決できる
- `scanIndex` で N 件のインデックスを1回で算出できるため、N 回のファイルオープンを防ぐ
- `getContent` は blockKey 必須とし、全 content 返却 API を意図的に設けない

### 合意決定 ✅ CLOSED

**決定:** harness-query-engine は4フェーズ構成で設計する。フォルダ集約とドキュメント内インデックスを持ち、全 content 読み込みを禁止する。

**確定した設計:**

**フォルダ構成:**
- `documents/specs/` — ドメイン仕様（SOURCE・Git 管理・永続成果物）
- `documents/knowledge/` — Knowledge 集約（SOURCE・Git 管理・harness-knowledge-engine が動的集約でクエリ）
- `specs/<id>.html` / `knowledge/<id>.html` — harness-render-engine 出力（RENDERED 人間向け）
- PBI は外部ツール（Jira / Linear 等）で管理（has-udd は refs.pbiRef で参照のみ）
- SBI は Sprint Planning ツールで管理（has-udd のスコープ外）

**インデックスの2層構造:**
- フォルダ集約 — フォルダ内ドキュメントの title + summary（複数ドキュメント横断時の入口・`_index.json` ファイルは廃止し動的集約）
- ドキュメントブロックインデックス — ドキュメント内ブロック一覧と prompt（blockKey 導出の入口・読み取り時に document の blockType 群 × schema の x-prompt-query から動的算出。document.json には保存しない）
- 両方とも has-udd が読み取り時に動的算出する。人間が編集しない・永続インデックスファイルは持たない

**harness-query-engine 実装方針:**
- manifest.json は持たない
- スキャン + インメモリキャッシュ（セッション限り）
- jsonpath-ng は内部実装。AI にパス式を渡させない
- has-udd バイナリがロジックを保持。SKILL.md のみプロジェクトへ配置

**4フェーズクエリ戦略（確定）:**

| フェーズ | 操作 | コスト |
|---|---|---|
| Phase 0 | フォルダ動的集約 → 関連ドキュメントを絞り込む | 最軽量 |
| Phase 1 | エンベロープ → status / refs / tags でフィルタ | 軽量 |
| Phase 2 | ブロックインデックス（動的算出）→ blockKey を導出 | 中量 |
| Phase 3 | `content[blockKey]` → 必要ブロックのみ取得 | 重量（必要最小限） |

**インターフェース（確定）:**
```
Phase 0: findByFolder(path, filters?)       フォルダをスキャンして title+summary を動的集約
Phase 1: getById(documentId)                ファイル名スキャン → エンベロープ返却
         findByPath(subdomain, type, filters?) フォルダトラバーサル → エンベロープ返却
         findByRef(refKey, refValue)         refs 逆引き → エンベロープ返却
         count(path, filters?)              件数のみ返す
Phase 2: getIndex(documentId)               ブロックインデックスを動的算出して返す
         scanIndex(documentIds[])           複数件のインデックスを動的算出して一括返す
Phase 3: getContent(documentId, blockKey)   blockKey 必須。全 content 返却禁止
```

**理由:**
- AI に構造を推論させない原則を harness-query-engine まで貫く（動的算出されるブロックインデックスがクエリの入口）
- フォルダ構造が subdomain × documentType を表現するため、パス = 第一フィルタ
- manifest を持たないことで保守コストをゼロにし、スキャンで十分な規模（100〜200件）

**次のアクション:** 論点3（harness-render-engine 出力パターン）に進む

---

## 論点 3: harness-render-engine はどんな出力パターンを持つべきか

出力形式は MD / HTML / DocComment の3種類。それぞれ用途・トリガー・テンプレート管理が異なる。

### 前提（確定済み）

**ドキュメントの4層構造（全 documentType 共通）:**

```
[1] HEADER    → document identity（envelope から固定ロジックで生成）
                documentId / documentType / status / tags

[2] OVERVIEW  → purpose summary（content.overview ブロックから固定ロジックで生成）
                全 documentType で必ず最初に配置

[3] BODY      → 主要定義（x-document-structure + x-render で Schema 駆動）
                documentType によって異なる部分

[4] RELATIONS → connections（refs フィールドから固定ロジックで生成）
                全 documentType で必ず最後に配置
```

**レンダリングの責務分担:**

| 責務 | 担当 |
|---|---|
| HEADER / OVERVIEW / RELATIONS の生成 | スクリプト固定ロジック |
| BODY ブロックの見出しレベル | `x-document-structure.headingLevel`（各ブロックに付与） |
| BODY ブロックの内容フォーマット | `x-render`（Jinja2 テンプレート・JSON 内インライン） |
| ブロックの出力順 | `x-render-order` |

**Schema 構成（テンプレート DRY 化）:**

schema はパッケージ内（`src/has_udd/domain/model/`・配布しない・importlib.resources / schema_repository で解決）。`.has-udd/schemas/` は存在しない。

```
src/has_udd/domain/model/
├── common/
│   └── block-types.json     ← blockType 共通定義（validation + x-render）
│       $defs: AcceptanceCriteria / DomainEvents / Overview / ...
├── UsecaseSpec/v1.json      ← $ref で参照 + x-prompt / x-prompt-query を追加
└── DomainModelSpec/v1.json
```

同じ blockType（例: AcceptanceCriteria）は複数 Schema から `$ref` で参照。x-render は1箇所に定義。x-prompt / x-prompt-query だけ各 Schema で文脈に合わせて記述。

**レンダリングエンジン:** Python + Jinja2（テンプレート文字列は Schema JSON に inline）

**骨格生成（create コマンド）:**

```bash
uvx has-udd create UsecaseSpec uc-order-create --refs pbiRef=pbi-001 --tags subdomain:order
```

Schema を読んで骨格 document.json を機械的に生成する：
- `blockType` → Schema の `properties.{key}.properties.blockType.const` から確定
- 必須フィールド → Schema の `required` から空値（`""` / `[]`）で初期化
- ブロックインデックスは生成しない → 読み取り時に query/knowledge engine が schema の `x-prompt-query` から動的算出する
- `status: "CREATED"` で作成

AI はこの骨格 + Schema の `x-prompt` を読んで値を埋めるだけ。

### AI 初期見解（DocComment 未議論）

**見解:** MD / HTML は document.json → x-render テンプレートで直接生成（MD → HTML 変換を経由しない）。DocComment は別カテゴリ（ソースコードへの注入）として未確定。

**has-udd validate の検証フロー（確定）:**
```
① JSON Schema バリデーション（フィールド型・required）
② blockType が common/block-types.json に登録されているか
③ x-render の Jinja2 構文が正しいか（env.parse() で検証）
④ x-render が参照する変数が block の properties に存在するか（Jinja2 AST 解析）
```

**MD / HTML の出力フロー（確定）:**

```
document.json（.has-udd/documents/{type}/{id}.json）
  ↓ schemaRef → Schema を読む
  ↓ x-render-order 順に BODY ブロックを処理
  ↓ Jinja2 で x-render テンプレートを展開

HTML: .has-udd/{type}/{id}.html         ← 人間向けビューア（集約フォルダ内で完結）
MD:   .has-udd/skills/{name}/SKILL.md   ← Skill のみ（tool-recognized 出力）
```

HTML は MD から変換しない。Schema の `x-render` アノテーション（`md` キーと `html` キー）それぞれ独立して document.json から生成する。

**CLI:**
```bash
uvx has-udd render documents/specs/uc-order-create.json --format md
uvx has-udd render documents/specs/uc-order-create.json --format html
uvx has-udd render documents/specs/uc-order-create.json --format all
```

**DocComment（未議論）:** CodingTemplate 専用。ソースコードへの注入であり MD/HTML と性質が異なる。別途議論が必要。

### 合意決定 ✅ CLOSED

**決定:** harness-render-engine の出力は MD / HTML / DocComment + CodingTemplate の4形式。DocComment は言語別シンタックスラッパー + Jinja2 コンテンツで再現性・冪等性を保証。CodingTemplate は has-udd が契約基盤を提供し、エンドユーザが Tech Stack に応じたテンプレートを用意する。

---

#### DocComment 設計（確定）

**アーキテクチャ（2層分離）:**

```
x-render アノテーション（docComment キー・Jinja2）  ← コンテンツ生成（言語非依存）
       ↓
言語別シンタックスラッパー         ← has-udd スクリプトが適用
       ↓
ソースコードへ注入（冪等）
```

**コンテンツ層（x-render の docComment キー）:**

```json
"docComment": "{{ block.summary }}\n{% for method in block.methods.items %}\n@param {{ method.name }} {{ method.description }}\n{% endfor %}\n@returns {{ block.returns }}"
```

- Jinja2 テンプレートで記述（MD / HTML と同じエンジン）
- クラスレベル / メソッドレベルの2種類

**シンタックスラッパー（対応言語）:**

| 言語 | スタイル | マーカー例 |
|---|---|---|
| TypeScript / JavaScript | JSDoc `/** */` | `@param` `@returns` `@throws` |
| Python | Google style | `Args:` `Returns:` `Raises:` |
| Java | Javadoc `/** */` | `@param` `@return` `@throws` |
| Go | godoc `//` | prose のみ（タグなし） |
| Rust | rustdoc `///` | `# Examples` `# Panics` |
| C# | XML doc `///` | `<param>` `<returns>` |
| PHP | PHPDoc `/** */` | `@param` `@return` |

**冪等性の保証:**

```
注入済みマーカーで二重注入を防止:
  # @has-udd:uc-order-create
  
  → マーカーがあれば上書き、なければ挿入
  → 同じ document.json から2回実行しても結果が同じ
```

CodingSpec Schema に `targetFile` / `targetFunction` / `language` を持ち、注入先を確定的に定義する。

---

#### CodingTemplate 設計（確定）

**has-udd が提供する基盤（契約）:**

```
① CodingSpec Schema      ← テンプレートで使える変数の定義（SSOT）
② Jinja2 テンプレートエンジン ← 実行基盤
③ DocComment レンダラー   ← 言語別シンタックスラッパー
④ テンプレート変数仕様    ← 自動生成されるリファレンス
⑤ バリデーション          ← テンプレートが参照する変数が Schema に存在するか
```

**エンドユーザが用意するもの:**

```
.has-udd/templates/
├── repository.ts.j2       ← TypeScript Repository
├── repository.py.j2       ← Python Repository
├── usecase.java.j2        ← Java UseCase
└── ...                    ← Tech Stack に応じて自由に追加
```

**テンプレート例（TypeScript）:**

```jinja2
{# .has-udd/templates/repository.ts.j2 #}
{{ doc_comment }}
export class {{ content.className.value }} {

  {% for method in content.methods.items %}
  {{ method | method_doc_comment }}
  async {{ method.name }}(
    {% for param in method.params %}
    {{ param.name }}: {{ param.type }}{% if not loop.last %},{% endif %}
    {% endfor %}
  ): Promise<{{ method.returns }}> {
    throw new Error("Not implemented");
  }
  {% endfor %}
}
```

**スコープ境界（重要）:**

```
CodingTemplate が保証するもの（構造契約）:
  ✅ クラス名・メソッドシグネチャ・パラメータ型・戻り値型
  ✅ DocComment の再現的生成
  ✅ import 文（DependencyList から導出）
  ✅ テストスタブ骨格（AcceptanceCriteria から導出）
  ✅ 構造の一貫性（全 Repository が同じパターン）

CodingTemplate が保証しないもの:
  ❌ メソッド内部のロジック（開発者 or AI の責任）
  ❌ エラーハンドリングの詳細
  ❌ アルゴリズム・データ変換

→ 実装保証は UsecaseSpec の AcceptanceCriteria + テストスタブで担う
```

---

#### Skills 設計（確定）

---

**アノテーション拡張（x-prompt-template）:**

CodingSpec Schema に `x-prompt-template` を追加。`properties` だけでは AI がテンプレートでの変数の使い方を推論できないため、使い方の指示を Schema に持たせる。

```json
"methods": {
  "x-prompt-write":    "このクラスが持つメソッドを列挙してください。",
  "x-prompt-query":    "メソッド一覧ブロック。実装すべき操作の一覧を持ちます。",
  "x-prompt-template": "{% for method in content.methods.items %} でループ。method.name でメソッド名、method.params で引数リスト、method.returns で戻り値型を参照します。スタブは throw new Error('Not implemented') で統一してください。"
}
```

---

**Skill の種別（カテゴリ）と層の対応:**

| 種別 | 作成者 | アーキテクチャ上の層 |
|---|---|---|
| `engine` | OSS 提供（不変）| Secondary Adapter（インフラ基盤） |
| `custom` | ユーザー作成 | Application Layer の手順・参照定義 |

両種別とも `SkillSchema/v1` の同一 Schema。`invocationSpec` は engine 固有の optional フィールド。
どちらも document.json を持ち、Schema の `x-render`（`md` キー）から SKILL.md を生成する（Harness 原則を OSS 提供者にも適用）。

**命名規則:**
- フォルダ名: kebab-case
- engine 種別: `harness-*-engine` パターン（Secondary Adapter であることを明示）
- custom 種別: 自由命名（プロジェクト固有の目的を表す名前）

---

**Skill フォルダ構成:**

```
.has-udd/skills/
│
├── skills-creator/                  ← スキル生成エンジン（種別ごとに生成）
│   ├── SKILL.md                     ← 種別判定 → references/ に委譲
│   └── references/
│       ├── gen-engine-skill.md      ← engine 型の生成手順（OSS提供者向け）
│       └── gen-custom-skill.md      ← custom 型の生成手順（ユーザー向け）
│
│ # engine 種別（OSS 提供・不変）
├── harness-query-engine/            ← document.json 読み取り基盤
├── harness-render-engine/           ← MD/HTML レンダリング基盤
├── harness-knowledge-engine/        ← Knowledge クエリ基盤（動的集約・Facade パターン）
├── harness-scaffold-engine/         ← schema→空の document.json 骨格生成 + validate op（旧 spec-engine を置換）
└── harness-audit-engine/            ← I/O トレース + Interface 契約整合性（旧 contract-engine を改名）
│ # template/coding engine は Phase 3 に延期
│
│ # custom 種別（ユーザー作成）
└── {user-defined}/                  ← プロジェクト固有の手順・参照定義（例: create-coding-template）
```

---

**engine と custom の役割分担（CodingTemplate の例）:**

```
template/coding engine（engine 種別・Phase 3 に延期）:
  → 生成基盤の提供
  → Jinja2 エンジン・構造契約・DocComment レンダラー・x-prompt-template の仕様
  → 「テンプレートがどう動くか」のインフラ

create-coding-template（custom 種別）:
  → template/coding engine の基盤を使ってユーザーが Tech Stack 向けテンプレートを作る手順
  → TypeScript + NestJS のリポジトリテンプレートを作る、など
  → 「自分のテンプレートを作る」操作
```

**Spec 作成は custom スキル不要:**

Spec の構造は harness-scaffold-engine が骨格を生成し、x-prompt-write が AI を完全にガイドする。ユーザーが「構造を決める」必要がないため custom スキルは不要。

```
CodingTemplate 作成: ユーザーが構造を決める → custom スキルで手順定義
Spec 作成:          has-udd が構造を決めている  → custom スキル不要・scaffold engine が骨格を生成し x-prompt がガイド
```

---

**CLAUDE.md の役割（Primary Port）:**

CLAUDE.md は has-udd init が生成するファイルで、ヘキサゴナルアーキテクチャの Primary Port に相当する。CLAUDE.md は engine routing を持たない（Primary→Secondary 直接参照は依存違反になるため）。CLAUDE.md は Orchestrator（HarnessAgent）の起動指示のみを持ち、engine への接続は Orchestrator が一手に担う。

```markdown
# has-udd（CLAUDE.md 内・薄いエントリポイント）
has-udd の操作を行う際は Orchestrator（HarnessAgent）を起動せよ。
engine への接続・ルーティングは Orchestrator が担う（CLAUDE.md は engine を直接参照しない）。
```

```
Primary Adapter:  Claude Code / Kiro / Codex
Primary Port:     CLAUDE.md / AGENTS.md（has-udd init が生成・engine routing を持たない）
Application Core: Orchestrator（engine routing）+ Custom Skills（Application層）
                  + Skill/Spec/Knowledge/Agent/Coding 集約（Domain層・documentType ごと）
Secondary Port:   ファイルI/O・バリデーション・レンダリングのインターフェース
Secondary Adapter: engine Skills（harness-*-engine）
```

---

**理由:**
- DocComment を Jinja2 + 言語別ラッパーに分離することで再現性・冪等性を機械的に保証
- CodingTemplate のスコープを「構造契約」に限定することで複雑なロジックを適切に除外
- engine / custom の2種別でユーザーと OSS提供者の責務境界が明確になる
- CLAUDE.md は Primary Port として Orchestrator 起動のみを担い、engine routing は Orchestrator に集約（Primary→Secondary 違反を回避）

**次のアクション:** 論点4（schema 要件）に進む

---

## 論点 4: schema（JSON Schema, ドメインモデル）と harness-audit-engine の要件は何か ✅ CLOSED

論点1〜3とアーキテクチャ原則から逆算して、schema の契約整合性と harness-audit-engine の要件を確定する。

### 合意決定 ✅ CLOSED

**決定:** I/O トレースと契約整合性は harness-audit-engine が担い、品質ゲート（document-vs-schema バリデーション）は harness-scaffold-engine の validate op + 共有コア（jsonschema）が担う。I/O トレース JSON は Spec 単位で管理し、修正時はリセット（履歴蓄積なし）。

---

#### 責務1: I/O トレース（harness-audit-engine）

Role間・Skill間のやり取りを追跡可能にする実行記録。**成果物（document.json）とは独立した JSON ファイル**として管理する。

> **位置づけ（最新合意）**: この `.trace.json` の `jobs[]` は **Job 集約廃止後の実行トレース＝リードモデル**。Job 集約を持たない代わりに、実行の可観測性（失敗・I/O・委譲チェーン）が必要な場合のみ harness-audit-engine がこのトレースで担う。状態の SSOT はあくまで各 document の status であり、このトレースは一時ファイル（.gitignore）。`jobs[]` は「作業実行」の語彙としての Job であって集約ではない。

```
document.json（成果物）           .trace.json（実行記録）
───────────────────────           ─────────────────────────────
AI が値を書く                      スクリプトが update / append する
人間が読む成果物                   Orchestrator が読む実行状態
永続（Git 管理）                   一時（.gitignore）
```

**I/O トレース JSON の構造:**

```json
{
  "specRef": "uc-order-create",
  "workflowStatus": "IN_PROGRESS",
  "startedAt": "2026-06-23T10:00:00Z",
  "completedAt": null,

  "jobs": [
    {
      "jobId": "job-001",
      "agentIdRef": "orchestrator",
      "parentJobRef": null,
      "status": "DONE",
      "inputDocRef": "pbi-order-001",
      "outputDocRef": "uc-order-create",
      "skillsInvoked": ["harness-query-engine", "harness-scaffold-engine"],
      "delegations": ["job-002"]
    },
    {
      "jobId": "job-002",
      "agentIdRef": "dev-agent",
      "parentJobRef": "job-001",
      "status": "RUNNING",
      "inputDocRef": "uc-order-create",
      "outputDocRef": null,
      "skillsInvoked": ["harness-render-engine"],
      "delegations": []
    }
  ],

  "events": [
    { "type": "WORKFLOW_STARTED",    "timestamp": "...", "jobId": "job-001" },
    { "type": "JOB_DELEGATED",       "timestamp": "...", "from": "job-001", "to": "job-002" },
    { "type": "SKILL_INVOKED",       "timestamp": "...", "jobId": "job-002", "skillId": "harness-render-engine" }
  ]
}
```

**スクリプトの操作:**
```
append: jobs[]        新 Job 追加時
update: jobs[].status Job 状態変化時
update: jobs[].outputDocRef DONE 確定時
update: workflowStatus 全 Job DONE 時に COMPLETED へ
append: events[]      監査ログ追記（常に append のみ）
```

**配置と管理:**
```
配置: .has-udd/traces/{id}.trace.json（harness-audit-engine が管理）
Git:  .gitignore（ローカル一時ファイル）
```

**ライフサイクル:**
```
has-udd create UsecaseSpec {id}  → .trace.json 生成（spec init と同時）
  ↓ Orchestrator が Jobs を委譲しながらワークフローが進む
UsecaseSpec RENDERED
  ↓ まだ削除しない
CodingTemplate / TestTemplate 生成完了  → .trace.json 削除可能
```

**Spec 修正時の方針:**
```
修正が入った場合 → trace をリセット（上書き）して新ワークフローを開始
履歴は蓄積しない（シンプルさを優先）
FeedbackReport が必要とする集計は events[] から取得できるため履歴蓄積は不要
```

---

#### 責務2: 構造的な契約整合性

has-udd が生成した全成果物は Schema に準拠していることを保証する。

```
harness-render-engine への影響:
  → x-render テンプレートが参照する変数は必ず存在する（Schema が保証）
  → 出力が常に決定論的

harness-query-engine への影響:
  → Phase 3 で取得する blockKey は必ず存在する（Schema が保証）
  → null チェック不要・クエリが信頼できる
```

schema はドメインモデルそのもの。全 Role が同じ schema（ドメインモデル）に基づいて動く。

---

#### 責務3: 品質ゲート（バリデーション）

document-vs-schema バリデーションが常に通過点として機能することで再現性・冪等性を担保する。共有コア（jsonschema）+ harness-scaffold-engine の明示的 validate op が担う。

```
CREATED
  ↓ has-udd validate（4段階）
  ① JSON Schema バリデーション（フィールド型・required）
  ② blockType が common/block-types.json に登録されているか
  ③ x-render の Jinja2 構文チェック（env.parse()）
  ④ x-render が参照する変数が block の properties に存在するか
VALIDATED  ← ここを通過したものだけが下流に流れる
  ↓
harness-query-engine / harness-render-engine（入力が常に valid であることを前提にできる）
  ↓
RENDERED
```

品質ゲートが機能していれば:
```
同じ VALIDATED な document.json → 同じ Render 出力（再現性）
2回 Render しても同じ結果（冪等性）
```

**3つの責務の関係:**

```
責務1（I/O トレース・audit-engine）  → 何がどこを通ったかを定義・記録する
責務2（構造整合性・schema）          → 成果物が常に正しい形をしていることを保証する
責務3（品質ゲート・scaffold validate）→ 責務2を強制する仕組み・再現性と冪等性を制度的に担保
```

---

**理由:**
- I/O トレースを Spec 単位・1ファイルにすることで Orchestrator が1ファイルを読むだけで全 Job の状態を把握できる
- 履歴蓄積なしのリセット方式でファイル肥大化を防ぎ Orchestrator の混乱を避ける
- 品質ゲート（document-vs-schema バリデーション）が常に機能することで harness-query-engine / harness-render-engine の入力信頼性が確立される

---

### harness-audit-engine / scaffold validate スクリプト設計（確定）

#### 論点A: 呼び出しモデル ✅

**v1 から Skills モード / MCP モード両対応。コアロジックは共通。**

```
コアロジック（Python 関数）
  validate(), create_trace(), append_job() ...
         ↑
┌────────┴────────┐
CLI adapter       MCP adapter
（typer）         （fastmcp）
```

```python
# コアロジック（1回書くだけ）
def validate_document(doc_id: str) -> ValidationResult: ...

# CLI adapter
@app.command()
def validate(doc_id: str):
    result = validate_document(doc_id)
    console.print(result)

# MCP adapter
@mcp.tool()
def validate_document_tool(doc_id: str) -> dict:
    return validate_document(doc_id).to_dict()
```

```
Skills モード: has-udd init で即使える（デフォルト）
              harness-audit-engine / harness-scaffold-engine が AI に CLI 呼び出しタイミングを指示
MCP モード:   has-udd serve で起動（オプション）
              Claude Code が validate_document MCP ツールを直接呼ぶ
```

fastmcp のデコレータを付けるだけで MCP ツール化できるため追加コストはほぼゼロ。

---

#### 論点B: トレース操作の原子性 ✅

**write-then-rename パターンで対応。ファイルロック不要。**

```python
def atomic_write(path: Path, data: dict):
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(data, ensure_ascii=False, indent=2))
    tmp.rename(path)  # OS レベルでアトミック（同一FS内）
```

単一開発者 × 単一セッションでの真の並行実行はほぼ起きない。MCP モードで複数ツール呼び出しが重なるケースも、OS のアトミックリネームで読み取り側は常に完全なファイルを見る。

---

#### 論点C: バリデーション失敗時の挙動 ✅

**全4段階ハードフェイル。エラーメッセージは AI が自己修正できる粒度で返す。**

```
① JSON Schema バリデーション  → ハードフェイル（構造が壊れた document は下流で必ず失敗）
② blockType レジストリチェック → ハードフェイル（未登録 blockType は x-render が存在しない）
③ Jinja2 構文チェック         → ハードフェイル（構文エラーは実行時に必ず例外）
④ 変数クロスチェック           → ハードフェイル（存在しない変数参照はレンダリング時に失敗）
```

```
# エラーメッセージ例（AI が自己修正できる粒度）
ValidationError: Stage 4 failed
  Block 'acceptanceCriteria': x-render references '{{ block.dueDate }}'
  but 'dueDate' is not defined in properties.
  Available: ['blockType', 'title', 'items']
```

警告にしてしまうと品質ゲートとしての意味が消える。全段階ハードフェイルで VALIDATED を通過したものだけが下流に流れる保証を守る。

---

#### 論点D: Schema 解決 ✅

**schemaRef からパッケージ内リソースへの単純な解決。レジストリ不要。schema は `.has-udd/` に配布しない。**

```
schemaRef フォーマット: "{SchemaName}/v{N}"
例: "UsecaseSpecSchema/v1"

解決ルール（パッケージ内・importlib.resources / schema_repository 経由）:
  src/has_udd/domain/model/{SchemaName}/v{N}.json
  → src/has_udd/domain/model/UsecaseSpecSchema/v1.json
```

```python
def resolve_schema(schema_ref: str) -> Path:
    name, version = schema_ref.split("/")
    # importlib.resources でパッケージ内リソースとして解決（.has-udd には置かない）
    return schema_repository.path(name, version)
```

決定論的でシンプル。schemaRef が不正な場合は ① のバリデーションで即検知。

---

**次のアクション:** 各 documentType の Schema 具体定義へ進む（UsecaseSpec / DomainModelSpec / CodingSpec 等）

---

---

## 論点 5: Knowledge Schema の設計 ✅ CLOSED

Knowledge document.json の blockType 構成・x-render 方針を確定する。

### 前提（確定済み）

- Knowledge 集約は `.has-udd/knowledge/` に独立配置（他 documentType と混在しない）
- Knowledge クエリは **Facade パターン**を採用。`harness-knowledge-engine` が `knowledge/_index.json` 経由で全 Knowledge を動的クエリする
  - ユーザーは Knowledge document を追加するだけ。KnowledgeSkill document の作成は不要
  - `knowledge/_index.json` は Spec 系と同じ仕組みで自動生成（fields: title / summary / domains / tags）
- Knowledge は **AI が JSON で読む** → `x-render` の `md` キーは不要
- Knowledge は **人間が HTML で見る** → `x-render` の `html` キーのみ提供する

### 合意決定 ✅ CLOSED

#### blockType 構成（7種・全 Knowledge に適用）

AI が Knowledge に持っていてほしい内容を「AI の利用シーン」から逆算して導出した汎用構造。

| blockType | 役割 | 主なフィールド |
|---|---|---|
| `Overview` | 対象範囲と目的（1〜2文）。スコープマッチングと導入文に使う | title / summary |
| `Definitions` | 用語定義一覧 | items[].term / definition / characteristics[] |
| `Classifications` | 分類・種別（比較表） | categories[].name / definition / characteristics[] / guidance |
| `DecisionCriteria` | 判断基準フロー（どれを選ぶか・どこに配置するか） | steps[].question / yes / no |
| `Examples` | 具体例 | items[].label / description |
| `Warnings` | よくある設計ミス・アンチパターン | items[].name / description |
| `RelatedConcepts` | 関連する他の Knowledge への参照 | items[].knowledgeId / relationship |

**導出方針:** DDD・Scrum・ビジネスドメインを問わず、AI が知識を活用する際に必要とする情報（定義 / 分類 / 判断 / 例 / 注意 / 関連）を網羅した。

#### x-render 方針（確定）

```
x-render（md キー）   → 不要（AI は JSON を直接読む。Facade パターンで SKILL.md への埋め込みもしない）
x-render（html キー） → 必要（人間向けビューア）
```

#### x-render（html キー）テンプレート仕様（blockType 別）

| blockType | レンダリング形式 |
|---|---|
| `Overview` | `<section><h2>title</h2><p>summary</p></section>` |
| `Definitions` | 3カラム HTML テーブル（用語 / 定義 / 特性） |
| `Classifications` | 4カラム HTML テーブル（種別 / 定義 / 特性 / 運用指針） |
| `DecisionCriteria` | インライン SVG フローチャート（Python 側で steps[] から生成） |
| `Examples` | `<article>` カード列（label / description） |
| `Warnings` | `<section class="warning">` + `<article>` カード列（⚠️ name / description） |
| `RelatedConcepts` | リンクリスト（knowledgeId → .html / relationship テキスト） |

**DecisionCriteria の SVG 生成方針:**
- Mermaid JS（CDN）は使わない。JS 非対応環境・オフライン環境で表示できないため
- Python スクリプト（harness-render-engine）が `steps[]` から SVG を直接生成し、HTML にインラインで埋め込む
- CDN・外部リソース不要。スタンドアロンで動作する

#### document.json 構造例（バリデーション済みサンプル）

```json
{
  "documentId": "knowledge-hexagonal-architecture",
  "documentType": "Knowledge",
  "schemaRef": "KnowledgeSchema/v1",
  "status": "VALIDATED",
  "tags": ["domain:architecture", "topic:hexagonal"],
  "content": {
    "overview":          { "blockType": "Overview",          "title": "...", "summary": "..." },
    "definitions":       { "blockType": "Definitions",       "title": "...", "items": [...] },
    "classifications":   { "blockType": "Classifications",   "title": "...", "categories": [...] },
    "decisionCriteria":  { "blockType": "DecisionCriteria",  "title": "...", "steps": [...] },
    "examples":          { "blockType": "Examples",          "title": "...", "items": [...] },
    "warnings":          { "blockType": "Warnings",          "title": "...", "items": [...] },
    "relatedConcepts":   { "blockType": "RelatedConcepts",   "title": "...", "items": [...] }
  }
}
```

※ `_index` は保存しない（動的計算）。Knowledge の発見（list_index）も、各 document の Overview の x-prompt-query を schema から動的算出する。詳細は `design-engine-knowledge.md`（K-2/K-3）。

#### harness-knowledge-engine の役割（確定・詳細は design-engine-knowledge.md）

ユーザーが作成する KnowledgeSkill document は廃止。`harness-knowledge-engine` が OSS 側で1本提供し、全 Knowledge クエリを一元担当する（Facade）。

- インターフェース（2操作）: `list_index(tags?, match?)`（発見・tags namespace:value で絞り込み・prompt 付き）/ `get_by_ids(ids)`（取得）
- **`knowledge/_index.json` ファイルは持たない**（動的集約）。`_index` フィールドも document に保存しない（動的計算）
- 戻り値は structured JSON（レンダリングしない）。テキスト化は Orchestrator が delegation prompt 組成時に行う
- 呼び出すのは Orchestrator のみ（engine 認識は Orchestrator に閉じる）

ユーザーが行うこと: `.has-udd/documents/knowledge/{id}.json` を追加するだけ。has-udd が次回クエリ時に動的に拾う（事前生成インデックス不要）。

**理由:** Facade パターンを貫くと KnowledgeSkill の定義は不変になる。不変ならユーザーが作る必要はなく OSS 側で固定するのが原則に忠実。

---

**次のアクション:** Skill Schema（skillKind: "engine" / "custom"）の具体定義へ進む

## 論点 6: Skill Schema の集約設計と Schema 構造をどうするか ✅ CLOSED（集約設計・Schema方針まで）

Skill document.json の Schema 設計（SkillSchema/v1）と、集約設計の前提となる全体方針を確定する。

---

### 合意決定 ✅ CLOSED

---

#### 集約設計の大前提（確定）

**documentType ごとに別集約。「一つの Document 集約」ではない。**

| 判断理由 | 内容 |
|---|---|
| 業務ロジックが異なる | Skill は SKILL.md をレンダリング・Spec は設計詳細化・Knowledge は知識クエリ |
| status lifecycle が異なる | Spec（CREATED→VALIDATED）・Skill（RENDERED）など種別で変化する |
| スキーマが別々 | すでに schemaRef で種別ごとに別スキーマを参照している |
| 一貫性境界が別 | Skill と Spec は互いのトランザクション境界に関係しない |

「集約はできるだけ小さく設計する」原則に従い、documentType を集約の境界として採用。

**フォルダ構成は集約境界に従う（確定）:**

```
.has-udd/
├── documents/   ← 【SOURCE】document.json（集約ごとに type サブフォルダ）
│   ├── skills/
│   ├── specs/        ← Spec 集約（UsecaseSpec / DomainModelSpec）
│   ├── knowledge/
│   ├── agents/
│   └── coding/       ← Coding 集約（CodingTemplate）
├── skills/      ← 【RENDERED】<name>/SKILL.md
├── agents/      ← 【RENDERED】<name>.md
├── specs/       ← 【RENDERED】<id>.html
├── knowledge/   ← 【RENDERED】<id>.html
└── traces/      ← harness-audit-engine の .trace.json（.gitignore）
# schema は .has-udd に置かない（パッケージ内 src/has_udd/domain/model/）
```

source（document.json）は `documents/{type}/` に集約・rendered は集約フォルダ直下。Schema 集約はパッケージ内（`src/has_udd/domain/model/`）に置き、`.has-udd/` には配布しない。

---

#### Schema 集約と Document 集約の関係（確定）

```
Schema 集約（src/has_udd/domain/model/{Name}/v{N}.json・パッケージ内・配布しない）
  ← 構造の型定義（= ドメインモデル）。生成元。Class に相当。
  ← 変化するのはバージョン更新のみ（v1 → v2）
  ← importlib.resources / schema_repository で解決
  ↓ harness-scaffold-engine が空の document.json を骨格生成
Document 集約（.has-udd/documents/{type}/{id}.json）
  ← Schema のインスタンス。生成物。Instance に相当。
  ← AI が x-prompt を読んで値を記入 → CREATED → VALIDATED → RENDERED
  ← Document は Schema を schemaRef: "SkillSchema/v1" で ID 参照する
```

---

#### Skill 集約の DDD 設計（確定）

| 部品 | DDD 上の位置づけ | 理由 |
|---|---|---|
| Skill document.json | **集約（集約のルート）** | documentId で識別・status lifecycle あり（CREATED→VALIDATED→RENDERED）|
| content 内の blockType（目的/役割/Steps/インターフェース等） | **値オブジェクト** | 個別の ID なし・Skill と同じ lifecycle・Skill が変われば一緒に変わる |
| スキル固有ガードレール | **Skill 集約内の値オブジェクト** | Skill の一貫性境界内。document.json の content に blockType として持つ |
| 共通ガードレール（OSS 提供） | **各集約の content 内 blockType** | GuardrailSchema/v1（独立集約）は廃止。HarnessAgent/Skill それぞれの content に Guardrails blockType として定義 |

**スキル固有ガードレールのレンダリング方針:**
```
document.json（Skill 集約内の値オブジェクトとして保持）
  ↓ x-render（md キー）でレンダリング
SKILL.md → "Step N: 以下のガードレールに従うこと" として出力（内容を直接展開しない場合もある）
```
集約内に持ちつつ、SKILL.md への反映方法は x-render テンプレートで制御する。

---

#### Guardrail 集約の設計 ⛔ 廃止（後続の設計で覆された）

~~**Guardrail は独立した集約。OSS 提供 / custom の2種別を1つの Schema（GuardrailSchema/v1）で表現。**~~

**廃止理由:** Orchestrator も SubAgent も AgentSchema/v1 document.json として AI が値を書くため、ガードレールをわざわざ独立集約にする必要がない。各集約（AgentSchema/v1・SkillSchema/v1）の content 内 blockType（`Guardrails`）として定義する方針に変更。詳細は「ガードレール設計（確定・更新）」セクション参照。

- `GuardrailSchema/v1` → 廃止
- `skillKind: "guardrail"` → 廃止
- `guardrailKind` フィールド → 廃止

---

#### SkillSchema/v1 の設計方針（確定）

**一本の Schema（SkillSchema/v1）で `skillKind` による条件分岐。**

```json
{
  "allOf": [
    { "$ref": "#/$defs/DocumentEnvelope" },
    {
      "if":   { "properties": { "skillKind": { "const": "engine" } } },
      "then": { "required": ["invocationSpec", "io"] }
    },
    {
      "if":   { "properties": { "skillKind": { "const": "custom" } } },
      "then": { "required": ["io"] }
    }
  ]
}
```

skillKind 一覧（確定）:
- `engine`: OSS 提供・Secondary Adapter。`invocationSpec`（呼び出し仕様）が必須
- `custom`: ユーザー作成・Application Layer。IO のみ必須
- ~~`guardrail`~~: 廃止（ガードレールは各集約 content 内 blockType として表現）

---

#### SKILL.md 骨格（確定）

**engine 種別:**
```markdown
---
name: （engine 名）
description: （Triggers）
version: 1.0.0
---
## 目的
## 役割
## インターフェース
  ### 入力
  ### 出力
## 呼び出し仕様
  ### Skills モード
  ### MCP モード
## 実行手順
  ### Step 1: ...
## 参照
```

**custom 種別:**
```markdown
---
name: （スキル名）
description: （Triggers）
version: 1.0.0
---
## 目的
## 役割
## 処理対象と成果物
  ### 処理対象
  ### 成果物
## 実行手順
  ### Step 1: 共通ガードレールを読む
  ### Step 2: ...
## 参照
```

**IO セクション見出し名の使い分け:**
- engine: `## インターフェース`（機械的契約・厳密な入出力仕様）
- custom: `## 処理対象と成果物`（AI 可読・自然な表現）

**SKILL.md の軽量化方針:**
- 本体: 目的 + 役割 + Steps のみ（トークンコスト最小化）
- ガードレール・参照ファイルは Step の中で必要なタイミングに明示的に Read（オンデマンド）
- frontmatter description は常にコンテキストに存在（Triggers として機能）

---

#### ヘキサゴナルアーキテクチャの正しい位置づけ（確定・重要）

```
Primary Adapter:   Claude Code / Kiro / Codex
    ↓ 読む
Primary Port:      CLAUDE.md / AGENTS.md
                   ← has-udd init が生成する薄いエントリポイント
                   ← 「HarnessAgent を起動せよ」の指示のみ
                   ← engine routing を持たない（Primary→Secondary 違反になるため）
    ↓ 起動
────────────────────────────────────────────────
Application Core
  Application Layer:
    HarnessAgent（Orchestrator）
      ← AgentSchema/v1 document.json（CLAUDE.md とは別物・別ファイル）
      ← engine routing の知識を持つ・Secondary Port を呼ぶ唯一の責務
      ↓ SubAgent に委譲
    SubAgent（agentKind: "subagent"）= Role の実体（1 Role = 1 document）
      ← AgentSchema/v1 document.json
      ← roleKind（職種）＋ Persona ＋ skillRefs（Custom Skills のみ）＋ knowledgeRefs（不変 knowledge）
      ← engine Skills は含まない（engine 認識は Orchestrator のみ）
      ↓ Custom Skills を呼ぶ
    Custom Skills
      ← 「何をするか」の定義のみ
      ← インフラ（engine Skills）を意識しない
  Domain Layer:
    Skill集約 / Spec集約 / Knowledge集約 / Agent集約 / Coding集約
────────────────────────────────────────────────
Secondary Port:    各 harness-*-engine の SKILL.md（インターフェース定義）
    ↑ HarnessAgent（Orchestrator）のみが呼ぶ
Secondary Adapter: 各 engine Skills の実装（Python / MCP）
```

**engine Skills = 各々が独立した Secondary Port + Secondary Adapter（5本）**
- `harness-query-engine`    → document.json 読み取り
- `harness-render-engine`   → MD/HTML レンダリング
- `harness-knowledge-engine`→ Knowledge クエリ（knowledge ファイル群の Facade パターン）
- `harness-scaffold-engine` → schema→空の document.json 骨格生成 + validate op（旧 spec-engine を置換）
- `harness-audit-engine`    → I/O トレース + Interface 契約整合性（旧 contract-engine を改名）
- ※ template/coding engine（CodingTemplate 生成）は Phase 3 に延期

**⭐ CLAUDE.md と HarnessAgent は別物（重要）**

| | CLAUDE.md | HarnessAgent document.json |
|---|---|---|
| 種別 | Primary Port | Application Core（AgentSchema/v1） |
| 生成元 | has-udd init が生成 | AI が Schema を読んで値を書く |
| 内容 | 薄いエントリポイント・起動指示のみ | agentKind / roleKind / persona / guardrails / engine routing / knowledgeRefs / 委譲先 Role |
| engine routing | 持たない | 持つ（Orchestrator の知識として） |

**Custom Skills がインフラを意識してはならない理由:**
- Custom Skills = Application Layer（エンドユーザーが書く手順定義）
- engine Skills = Secondary Adapter（インフラ基盤）
- Custom Skills が engine を直接呼ぶ = エンドユーザーがインフラを意識する = 依存崩壊
- engine を呼ぶ責務は Orchestrator（Application Core）が一手に担う

---

#### ガードレール設計（確定・更新）

**GuardrailSchema/v1 は不要。ガードレールは各集約の content 内 blockType として持つ。**

| ガードレール種別 | 場所 | Schema |
|---|---|---|
| Orchestrator ガードレール | HarnessAgent document.json の content 内 | AgentSchema/v1 |
| Custom Skill ガードレール | 各 Custom Skill document.json の content 内 | SkillSchema/v1 |
| Engine Skill ガードレール | 各 engine Skill document.json の content 内 | SkillSchema/v1 |

- `GuardrailSchema/v1` → 廃止
- `common-guardrails-loader` Skill → 廃止
- Orchestrator も AgentSchema/v1 document.json として AI が値を書く → ガードレールも AI が記述

---

#### 論点6-D: 全 engine Skills をまとめる Facade SKILL.md は必要か ✅ CLOSED

**合意:** 不要。

- 各 engine は独立した Secondary Port として機能するため、まとめる Facade SKILL.md は作らない
- `skillKind` に `"facade"` 種別を追加しない（`"engine" | "custom"` の2種のみ）
- `harness-knowledge-engine` の Facade パターンは別の話（knowledge ファイル群をまとめる Facade）

---

#### 論点6-E: SkillSchema/v1 の blockType 完全定義（進行中）

**Schema アノテーション補足確定事項（論点6-E 議論中に合意）:**

**① x-render-order / x-render-level は MD・HTML 両方に効く**
- `x-render-order`: ブロック出力順を MD・HTML で統一
- `x-render-level`: 見出しレベルを harness-render-engine が動的解決
  - MD → level 2 = `##` / level 3 = `###`
  - HTML → level 2 = `<h2>` / level 3 = `<h3>`
- テンプレート内に `##` や `<h2>` をハードコードしない（x-render-level と重複するため）
- テンプレートはボディ部分のみ担う。見出しは engine が x-render-level から自動付与

**② x-render は md・html 両キーを Schema に持つ（契約の一貫性）**
- Schema = 契約。MD・HTML 両方のレンダリング仕様を Schema に定義することで契約が完結する
- HTML は MD から変換せず、各テンプレートから独立生成

**③ テンプレートエンジンは Jinja2 で統一（確定）**
- harness-render-engine（x-render テンプレート）も harness-template-engine（CodingTemplate）も Jinja2 統一
- 追加依存ゼロ（Python + Jinja2 は確定済み技術スタック）
- 論点1 例示の `{{#each block.items}}` は Handlebars 記法で誤り → Jinja2 記法 `{% for item in block.items %}` が正しい

```json
"x-render": {
  "md":   "{% for item in block.items %}- {{ item }}\n{% endfor %}",
  "html": "<ul>{% for item in block.items %}<li>{{ item }}</li>{% endfor %}</ul>"
}
```

---

**論点6-E-1: blockType 一覧 ✅ CLOSED**

| order | key | blockType | engine | custom |
|---|---|---|---|---|
| 1 | `purpose` | `Purpose` | 必須 | 必須 |
| 2 | `role` | `Role` | 必須 | 必須 |
| 3 | `interface` | `Interface` | 必須 | ❌ |
| 3 | `processingTarget` | `ProcessingTarget` | ❌ | 必須 |
| 4 | `invocationSpec` | `InvocationSpec` | 必須 | ❌ |
| 5 | `steps` | `Steps` | 必須 | 必須 |
| 6 | `guardrails` | `Guardrails` | 任意 | 任意 |
| 7 | `references` | `References` | 任意 | 任意 |

**`Guardrails` の位置づけ（確定）:**
- GuardrailSchema/v1（独立 document.json）廃止の結果、Skill の `content` 内 blockType として定義する
- Agent（AgentSchema/v1）にも同名の blockType が存在するが、**各 Schema が独立して `$defs` に定義する**
- 値オブジェクトはそれを所有する集約の Schema 内 `$defs` に定義する。共有ファイルは作らない

```json
// SkillSchema/v1.json（Skill 集約が所有）
{
  "$defs": {
    "Guardrails": { ... }
  },
  "properties": {
    "guardrails": { "$ref": "#/$defs/Guardrails" }
  }
}

// AgentSchema/v1.json（Agent 集約が独立して所有）
{
  "$defs": {
    "Guardrails": { ... }
  },
  "properties": {
    "guardrails": { "$ref": "#/$defs/Guardrails" }
  }
}
```

**値オブジェクトの Schema 定義方針（確定）:**
- 値オブジェクトは所有する集約の Schema ファイル内 `$defs` に定義する
- 同名の値オブジェクト（例: `Guardrails`）が複数 Schema に存在しても共有ファイルは作らない
- 理由: `x-prompt-write` / `x-prompt-query` の内容は集約のコンテキストによって異なるため、各 Schema が独自定義を持つことが正しい
- `$ref: "#/$defs/Guardrails"` で同ファイル内参照（外部ファイル参照なし）

**論点6-E-2: 各 blockType のフィールド構造詳細 ✅ CLOSED**

**engine / custom の I/O 表現は非対称（確定）:**

| | engine `Interface` | custom `ProcessingTarget` |
|---|---|---|
| 呼び出し元 | Orchestrator（プログラム） | SubAgent（AI） |
| 越える境界 | Python / MCP（技術的境界） | AI の理解（意味的境界） |
| 必要な情報 | 型・必須フラグ・名前 | 処理の意味・成果物の意味 |
| 表現 | 構造化配列 | 自然言語テキスト |

- engine Skills = Secondary Port + Adapter → 型情報まで含む構造化契約が必要
- custom Skills = Application Layer（AI が読む） → 意味的な自然言語記述が適切
- 非対称性は設計の欠陥ではなく、両者の責務の違いを正直に反映した結果

**各 blockType フィールド構造（確定）:**

```
Purpose:          text: string
Role:             items: string[]
Interface:        input: { name, type, required, description }[]
                  output: { name, type, description }[]
ProcessingTarget: target: string / artifact: string
InvocationSpec:   skillsMode: string / mcpMode: string
Steps:            items: { stepId, title, instruction }[]  ← 論点6-E-3 で詳細確定
Guardrails:       items: string[]
References:       items: { path, description }[]
```

**x-prompt 粒度の原則（確定）:**
- セマンティックに一塊のオブジェクト → オブジェクトルートに x-prompt-write 1つで成立
- 配列の各フィールドで書いてほしい観点が異なる → フィールドごとに x-prompt-write が必要
- 特定の観点で読んで欲しい → その粒度で x-prompt-query を仕込む

適用例:
- `Steps.items[].title` と `Steps.items[].body` → 観点が異なるため個別に x-prompt-write
- `Interface.input[].name / type / required / description` → 各フィールドに x-prompt-write

**呼び出しチェーンの確認（再確定）:**
- Orchestrator → has-udd として必須のガードレールを担う
- SubAgent（Role を体現） → Custom Skills を呼ぶ（Orchestrator が knowledge をコンテキスト読み込み済みで delegation prompt で渡す）
- Custom Skill Steps → ドメイン固有の手順のみ。インフラ語彙（document.json 等）は書かない
- 不変的な振る舞い（ガードレール確認等）→ Orchestrator の Guardrails blockType が担う

**論点6-E-3: Steps のネスト構造 ✅ CLOSED**

**B案（children ネスト）採用。理由:**
- 人間の可読性だけでなく AI の実行精度にも有効
- SubAgent が SubStep 単位で原子的に実行・確認・リトライできる
- Harness 原則：AI に構造を推論させない → SubStep として明示することで AI は構造を与えられた状態で動く
- ネストは1階層まで（Step → SubStep）

**Steps Schema 定義（確定）:**

```json
"steps": {
  "type": "object",
  "x-render-order": 5,
  "x-render-level": 2,
  "x-prompt-write": "このSkillの実行手順を定義してください。各Stepは独立したドメイン固有の操作単位です。複数の操作が含まれる場合は children に SubStep として分割してください。インフラ操作（document.json 読み取り等）は書かないでください。",
  "x-prompt-query": "このSkillの実行手順を持ちます。Skillの処理内容の把握・デバッグ・進捗追跡に使います。",
  "x-render": {
    "md":   "{% for step in block.items %}\n{{ step_h }} Step {{ loop.index }}: {{ step.title }}\n\n{% if step.body %}{{ step.body }}\n{% endif %}{% if step.children %}{% for sub in step.children %}\n{{ substep_h }} {{ sub.title }}\n\n{{ sub.body }}\n{% endfor %}{% endif %}{% endfor %}",
    "html": "{% for step in block.items %}<article><{{ step_tag }}>Step {{ loop.index }}: {{ step.title }}</{{ step_tag }}>{% if step.body %}<p>{{ step.body }}</p>{% endif %}{% if step.children %}<div class=\"substeps\">{% for sub in step.children %}<section><{{ substep_tag }}>{{ sub.title }}</{{ substep_tag }}><p>{{ sub.body }}</p></section>{% endfor %}</div>{% endif %}</article>{% endfor %}"
  },
  "properties": {
    "blockType": { "const": "EngineSteps または CustomSteps（skillKind で決定）" },
    "items": {
      "type": "array",
      "x-prompt-write": "実行順に Step を列挙してください。",
      "items": {
        "type": "object",
        "properties": {
          "stepId": {
            "type": "string",
            "x-prompt-write": "step-1, step-2 のように連番で付けてください。"
          },
          "title": {
            "type": "string",
            "x-prompt-write": "このStepで行うことを動詞で始めて簡潔に記述してください。"
          },
          "body": {
            "type": "string",
            "x-prompt-write": "SubAgentが実行すべきドメイン固有の手順を記述してください。children がある場合はこのStepの概要を書いてください。"
          },
          "children": {
            "type": "array",
            "x-prompt-write": "このStepをさらに細かい操作単位に分割する場合に SubStep を列挙してください。",
            "items": {
              "type": "object",
              "properties": {
                "stepId": {
                  "type": "string",
                  "x-prompt-write": "step-1-1, step-1-2 のように親 stepId を引き継いで連番で付けてください。"
                },
                "title": {
                  "type": "string",
                  "x-prompt-write": "このSubStepで行うことを動詞で始めて簡潔に記述してください。"
                },
                "body": {
                  "type": "string",
                  "x-prompt-write": "SubAgentが実行すべき具体的な手順を記述してください。"
                }
              },
              "required": ["stepId", "title", "body"]
            }
          }
        },
        "required": ["stepId", "title"]
      }
    }
  },
  "required": ["blockType", "items"]
}
```

※ x-render テンプレート内の見出し（Step = level+1、SubStep = level+2）の適用は harness-render-engine が x-render-level を基に動的解決する。テンプレートはボディ部分のみ担う（見出しタグはエンジンが付与）。

---

#### 論点6-F: Orchestrator の engine routing 表現（未着手）

HarnessAgent document.json（AgentSchema/v1）の content 内でどのように engine routing を表現するか。
routing 専用の blockType が必要か、それとも Guardrails や Steps の中に含めるか。

---

**次のアクション:** 論点6-E（SkillSchema/v1 の blockType 完全定義）→ 論点6-F の順で進む

---

## SKILL.md フロントマター設計（確定）

SKILL.md のフロントマターに設定できるフィールドは Claude Code 公式ドキュメントで定義されている。
harness-render-engine が document.json から SKILL.md を生成する際の出力仕様として確定する。

### サポート済みフィールド一覧（公式確認済み）

| フィールド | 型 | 用途 | has-udd での使用方針 |
|---|---|---|---|
| `name` | string | スキル一覧の表示名 | `doc["documentId"]` をそのまま使う |
| `description` | string | AI の自動呼び出し判定に使用。最大1,536文字 | `doc["content"]["purpose"]["text"]` を使う |
| `when_to_use` | string | description に追記される追加の呼び出しタイミング情報 | 将来的に purpose とは別の呼び出し条件を定義する場合に使用 |
| `argument-hint` | string | `/スキル名` 入力時のオートコンプリートヒント | engine Skill では不要（Orchestrator が直接呼ぶため） |
| `allowed-tools` | string \| list | 実行中に使用を許可するツールのホワイトリスト | engine Skill でセキュリティ制限が必要な場合 |
| `disallowed-tools` | string \| list | 実行中に使用を禁止するツール | 同上 |
| `model` | string | 実行時の LLM モデル指定 | 原則省略（セッションモデルを継承） |
| `effort` | string | 推論エフォートレベル（low / medium / high / xhigh / max） | 原則省略（セッション設定を継承） |
| `paths` | string \| list | このスキルが自動ロードされるグロブパターン | ディレクトリ限定スキルに使用 |
| `disable-model-invocation` | boolean | true にすると Claude の自動実行を禁止（手動呼び出しのみ） | デプロイ・破壊的操作スキルに使用 |
| `user-invocable` | boolean | false にすると `/` メニューから非表示（Claude には見える） | 内部スキル・バックグラウンド知識に使用 |
| `context` | string | `fork` でメイン会話から独立したサブエージェントを起動 | 重い処理・コンテキスト分離が必要な場合 |
| `agent` | string | `context: fork` と組み合わせてサブエージェントの種別を指定（Explore / Plan など） | 同上 |

**`version` フィールドは公式非サポート。** Claude Code は無視するが、既存 SKILL.md（ddd-advisor 等）には `version: 1.0.0` が含まれている。has-udd が生成する SKILL.md には含めない。

### has-udd が生成する SKILL.md の基本フロントマター

```yaml
---
name: <doc["documentId"]>
description: <doc["content"]["purpose"]["text"]>
---
```

engine Skill・custom Skill ともに最低限 `name` + `description` のみを出力する。
その他フィールドは document.json に専用フィールドを追加して対応する（将来拡張）。

### `disable-model-invocation` と `user-invocable` の使い分け

```
user-invocable: false のみ
  → メニューには表示されないが Claude は自動実行できる
  → 内部知識・バックグラウンドコンテキスト向け

disable-model-invocation: true
  → Claude は絶対に自動実行できない（ユーザー手動のみ）
  → デプロイ・破壊的操作・本番環境変更スキル向け
```
