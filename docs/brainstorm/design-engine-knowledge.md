# harness-knowledge-engine 設計ブレスト

## ★後日談（別セッション）: 独立engineとしては撤回

このブレストの結論（別Secondary Port・K-1〜K-6）は撤回した。ユーザー指摘:
「knowledge自体もドキュメント集約だと思いますよ」。

Knowledgeを独立集約でなくDocument集約の一documentTypeとして扱えば、この設計の
大部分は既にWaffleの`query`engineが満たしている:
- `list_index`相当 → `index_scan_dir`が既に「ディレクトリ走査＋各blockTypeの
  x-prompt-queryを動的算出」を実装済み（`_index_scan_dir`・`_index.json`を
  保存しない設計まで一致、K-3と同一結論）
- `get_by_ids`相当 → 複数documentへの`get_field`/`get_block`呼び出しで代替可能
- 別engineにする根拠だったFacadeパターン・意味的語彙の違いは、Knowledgeを
  独立集約として見る前提が崩れたことで成立しなくなった

唯一の実ギャップ（tagベースの絞り込み用にindex出力へtagsが無かった）は
`index_scan_dir`の小さな拡張（各documentの`tags`を出力に含める）で解消済み
（uc-query-document.json・query_engine.py実装済み）。

**残タスク**: `KnowledgeSchema`をagg-schemaの対象（`domain/model/`）に追加すれば、
既存のcreate/fill/validate/render/queryが全てそのまま動く（新規engineコード不要）。
下記のK-1〜K-6の設計内容（7 blockType・タグ規約・x-render=HTML等）は
KnowledgeSchema自体の設計として参考にできる部分もあるが、「別engine」の
決定は無効。

## 目的

`harness-knowledge-engine` の document.json を Python 実装レベルで設計する。
Knowledge 集約（`.has-udd/documents/knowledge/{id}.json`）への意味的アクセスを担う Secondary Port。

---

## 前提（確定済みアーキテクチャから）

### Knowledge の位置づけ

- Knowledge document は `.has-udd/documents/knowledge/{id}.json`（KnowledgeSchema/v1）
- KnowledgeSchema の blockType（7種・CLOSED）: `Overview` / `Definitions` / `Classifications` / `DecisionCriteria` / `Examples` / `Warnings` / `RelatedConcepts`
- x-render は **HTML のみ**（人間が読む用。MD 不要）。`DecisionCriteria` のみ Python でインライン SVG 生成
- KnowledgeSkill は廃止。ユーザーは Knowledge document を作るだけ

### knowledge の2軸（確定）

| 軸 | 値 |
|---|---|
| **提供者軸** | OSS デフォルト（`has-udd:framework` — DDD・has-udd 思想）/ エンドユーザー（`domain:*` — 自社業務） |
| **束ね方軸** | Role 不変（Role 定義の `knowledgeRefs` が宣言）/ 要望スコープ（Orchestrator が要望から判断） |
| 区別 | `tags`（`has-udd:framework` / `domain:order` 等）で表現 |

### アクセス主体（確定・engine 認識は Orchestrator のみ）

- **knowledge-engine を呼ぶのは Orchestrator のみ**。Role / Custom Skill は呼ばない
- Orchestrator は2経路で knowledge を取得する:
  1. **Role 不変 knowledge**: Role 定義の `knowledgeRefs` を読んで機械解決（推論ゼロ）
  2. **要望スコープ knowledge**: 要望のドメインスコープ（tags）で取得
- 取得結果を delegation prompt に載せて Role に渡す（コンテキストは共有されないため）

### Harness 原則

- Python（knowledge-engine）が JSON を読む。AI は engine が返した content を読む（生 JSON を直接読まない）

---

## 論点一覧

| # | 論点 | 状態 |
|---|---|---|
| K-1 | query-engine と別 engine にする必要があるか（責務分担） | ✅ CLOSED |
| K-2 | クエリインターフェース設計（scope / tags / id による取得） | ✅ CLOSED |
| K-3 | _index の扱い（動的集約 vs `_index.json` ファイル） | ✅ CLOSED |
| K-4 | 戻り値フォーマット（structured か rendered か・テキスト化の責務） | ✅ CLOSED |
| K-5 | core 実装の共有範囲（query-engine と共通化する部分） | ✅ CLOSED |
| K-6 | Steps / Guardrails | ✅ CLOSED |

---

## 論点 K-1: query-engine と別 engine にする必要があるか

### AI の立場（たたき台）

**別 engine にする。ただし core 実装は query-engine と共有する。**

Knowledge document も document.json なので、技術的には query-engine の `index_scan_dir` + `query` で読める。メカニズムは同じ。それでも別 Secondary Port として公開する理由:

**根拠1: 意味的クエリの語彙が違う**
- query-engine: 「このパスのこのブロックを取れ」（構造的・パス指定）
- knowledge-engine: 「order ドメインに関連する knowledge を集めろ」（意味的・スコープ指定）
- Orchestrator が後者を前者の語彙（path・blockKey）に翻訳するのは、knowledge の配置を Orchestrator が知ることになり結合が増す

**根拠2: knowledge 固有の絞り込みセマンティクス**
- tags（`has-udd:framework` / `domain:*`）による提供者・スコープ絞り込み
- 複数 knowledge document の横断集約（1つのドメインに複数 knowledge）
- これは query-engine の汎用クエリには無い knowledge 特化の関心事

**根拠3: Facade パターン（Knowledge 集約への単一窓口）**
- 「knowledge について知りたい」を1つのポートに集約する
- knowledge の物理配置（ファイルパス・_index）を隠蔽する

**ただし core は共有:**
- ディレクトリスキャン・JSON ロード・tags フィルタ等の実装は query-engine の core と共通化（`core/` に共通基盤）
- 「別 engine（別 Secondary Port・別 MCP ツール）= 別公開インターフェース」だが「実装は共有」

```
core/
├── _scan.py        ← ディレクトリスキャン・JSON ロード（共通）
├── query.py        ← query-engine（構造的アクセス）
└── knowledge.py    ← knowledge-engine（意味的アクセス・_scan を再利用）
```

### 合意

**別 engine（別 Secondary Port・別 MCP ツール）として公開。core 実装（ディレクトリスキャン・JSON ロード・tags フィルタ）は query-engine と共有。**
公開インターフェースは分離・実装は共有。「検索エンジンは同じでも収集したい情報が違う」を具現化。

---

## 論点 K-2: クエリインターフェース設計 ✅ CLOSED

### 合意内容

**Index も他エンジンと同じく `prompt` を返す。タグは prompt で自己記述され、Orchestrator は推論で当てない。**

#### タグ規約（namespace:value）

| namespace | value | 例 | 付与者 |
|---|---|---|---|
| `has-udd` | `framework` 等 | `has-udd:framework` | OSS（固定規約） |
| `domain` | subdomain 名 | `domain:order` | ユーザー（要望から構築可能） |
| `role` | roleKind | `role:qa` | Role 不変 knowledge の分類 |

#### インターフェース（2操作・全て prompt 付き）

| 操作 | 戻り値 | 用途 |
|---|---|---|
| `list_index(tags?, match?)` | `[{ id, tags, prompt }]`（タグで絞り込み可・タグごとにグルーピング） | 発見。タグ×メンバー prompt を読んで意味選択（要望スコープ） |
| `get_by_ids(ids)` | `{ prompt, value: [content…] }` | 選んだ knowledge / knowledgeRefs を取得 |

- Orchestrator の2経路に対応: 要望スコープ → `list_index(tags?)` で発見 → ids 選択 → `get_by_ids` ／ Role 不変 → `get_by_ids(knowledgeRefs)` 直接
- `get_by_scope` は `list_index(tags)` + `get_by_ids` に分解されるため独立操作にしない

#### prompt の出どころ（重要・機械的集約の限界）

エンジンは決定論的 Python（LLM なし）→ **要約はできない。できるのは収集・グルーピング（結合）のみ。**

→ **タグ単位の合成プロンプトは作らない。** 質は「ドキュメント単位の authored prompt」に置く:

```
機械的にやること:  スキャン・タグでグルーピング・prompt を並べる（結合のみ）
人間が書くこと:    各 knowledge の Overview prompt（KnowledgeSchema の x-prompt-query・一度だけ）
AI がやること:     並んだ prompt を読んで意味選択
```

- 各 knowledge document の prompt = KnowledgeSchema の Overview ブロックの x-prompt-query を**動的計算**（document に `_index` として保存しない・query-engine の index 動的計算と同一仕組み）
- 「domain:order が何を指すか」= そのタグを持つドキュメント群の prompt の集まり（合成された1文ではなく、メンバー prompt を機械的に並べる）
- 選択の質 = knowledge 著者が書く x-prompt-query の質
- 動的集約なので `_index.json` 不要・常に最新（「ディレクトリ index_scan が _index.json を不要にした」既存決定と一貫）
- overview/prompt がタグ不備の安全網（タグ = 粗いプレフィルタ / prompt = 精密な意味シグナル）

#### 動作トレース

```
要望: 「order checkout のリファインメント」
Orchestrator:
① list_index() → 各 knowledge の prompt を読み「domain:order が該当」と意味判断
② get_by_ids([選んだ ids]) → 注文ドメインの knowledge content 取得
③ ＋ Role 定義の knowledgeRefs → get_by_ids([...]) で不変 knowledge 取得
④ ①②③をまとめて delegation prompt で Role に渡す
```

### 合意

タグ規約（namespace:value）＋ Index も prompt を返す（自己記述）＋ タグ合成はせずドキュメント単位の authored prompt を機械的グルーピング ＋ 2操作（list_index / get_by_ids）。

---

## 論点 K-3: _index の扱い（動的集約 vs `_index.json` ファイル） ✅ CLOSED

### 合意内容

**動的集約（実行時スキャン）。`_index.json` ファイルは作らない。**

根拠:
1. query-engine の `index_scan_dir` と同一決定（「ディレクトリ index_scan が `_index.json` を不要にした」既存確定）
2. 同期ずれゼロ・管理コストゼロ（静的インデックスは再生成が必要で同期ずれリスク）
3. knowledge は数十〜数百オーダー。実行時スキャンのコストは許容範囲

```python
def list_index(tags=None, match="any"):
    docs = scan_directory(".has-udd/documents/knowledge/")   # core/_scan を再利用（K-1）
    result = []
    for doc in docs:
        if tags and not _match_tags(doc["tags"], tags, match):
            continue
        # prompt は document に保存せず schema から動的計算（_index は持たない）
        schema = schema_repository.load(doc["schemaRef"])    # KnowledgeSchema/v1（パッケージ内）
        overview_type = doc["content"]["overview"]["blockType"]          # "Overview"
        prompt = schema["$defs"][overview_type + "Block"]["x-prompt-query"]
        result.append({"id": doc["documentId"], "tags": doc["tags"], "prompt": prompt})
    return result
```

- `_index.json` ファイルは存在しない。各 knowledge document 内の `_index.blocks.overview.prompt`（document 生成時に x-prompt-query から自動導出済み）を機械的に集める
- **要掃除**: 初期メモ「knowledge/_index.json 自動生成」は廃止済み。古い記述を後で掃除する

---

## 論点 K-4: 戻り値フォーマット（structured か rendered か・テキスト化の責務） ✅ CLOSED

### 合意内容

**knowledge-engine は structured JSON content をそのまま返す（機械的・レンダリングしない）。テキスト化は委譲境界で Orchestrator が推論で行う。**

#### 混同していた2つを分離

| | format | 主体 | 性質 |
|---|---|---|---|
| engine の戻り値 | knowledge-engine → Orchestrator | Python engine | **機械的**（structured JSON） |
| delegation prompt | Orchestrator → Role | Orchestrator（AI） | **推論**（text 化） |

#### 確定事項

- **AI の情報交換は JSON 構造**。レンダリング（MD/HTML）は「SKILL.md のように MD でしか認識されない場合」と「人間が HTML で見る場合」だけに必要。AI（Role）は JSON content をそのまま読める
- engine がテキスト化（要約・整形）するのは LLM が要る = 決定論的 Python の責務外。**engine はテキスト化しない**
- → knowledge-engine は `{ prompt, value }`・**value = structured JSON content** を返す（query-engine と同じ）
- KnowledgeSchema の x-render は **HTML のみ維持**（人間向け。AI 向けレンダリングは不要）

#### テキスト委譲が疎結合な理由（アーキテクチャ的核心）

```
knowledge-engine（機械的）  → structured JSON content を返す
Orchestrator（AI 推論）     → JSON を delegation prompt（テキスト）に成型して載せる ← 疎結合の契約
Role（AI 推論）             → テキストの delegation prompt を必要な入力形に推論で成型して使う
```

- JSON 構造をそのまま渡すと Role がその構造に**結合**する。テキスト＋受け手の推論なら構造に依存せず意味だけ受け取る → **テキストが脱結合の境界**
- 「委譲元でやるか委譲先でやるかは同じ（どちらも AI 推論）」→ 契約を text にして境界で1回テキスト化するのが素直
- 責務の分離: engine は機械的に JSON を返す / テキスト化は委譲境界で AI が行う

### 合意

knowledge-engine は structured JSON を返す（レンダリングしない）／ KnowledgeSchema x-render は HTML のみ維持／ テキスト化は Orchestrator が delegation prompt 組成時に推論で行う（疎結合の text 契約）。

---

## 論点 K-5: core 実装の共有範囲（query-engine と共通化する部分） ✅ CLOSED

### 合意内容

has-udd リポジトリ全体を1つのヘキサゴナルアーキテクチャで管理（ドメイン層=JSON Schema・アダプタ層=Python）。core 共有は以下:

| モジュール | 層 | 共有 |
|---|---|---|
| `scan_directory`（ディレクトリスキャン・JSON ロード） | outbound（fs adapter） | query / knowledge / 全 engine |
| `result`（{prompt,value} / {error,prompt,message} 組成） | shared | 全 engine 共通の戻り値規約 |
| `tags`（namespace:value マッチ any/all） | shared | query のフィルタ・knowledge のスコープ両方 |
| query / knowledge 固有ロジック | application | 別 use case（薄い層） |

パッケージ構成（ヘキサゴナル慣用）:
```
src/has_udd/
├── domain/
│   ├── model/       # schema（JSON・配布しない・importlib.resources で解決）
│   ├── ports/       # document_repository / schema_repository
│   └── services/    # schema_integrity
├── application/     # engine の use case（query/render/knowledge/…）
├── adapters/{inbound(cli,mcp), outbound(fs,jinja,jsonschema)}/
└── shared/{result,tags}.py
```

**2ヘキサゴン**: engine は①エージェント系では Secondary Adapter・②ソースコードでは Application（ポート合成・役割は Core への相対概念）。詳細はメモリ `project-implementation-architecture.md`。

---

## 論点 K-6: Steps / Guardrails ✅ CLOSED

### 合意内容

**Steps（engine の実行手順）:**
```
Step 1: 入力を検証する
  SubStep 1-1: operation が list_index / get_by_ids のいずれかか確認する
  SubStep 1-2: get_by_ids なら ids が非空配列か・list_index なら tags 形式（namespace:value）を確認する

Step 2: knowledge ディレクトリをスキャンする
  SubStep 2-1: .has-udd/documents/knowledge/ を scan_directory で走査（core 共有・outbound/fs）
  SubStep 2-2: 各 document.json をロードする

Step 3: operation に応じて処理する
  SubStep 3-1（list_index）: tags でフィルタ → { id, tags, prompt } を集約（prompt は各 document の Overview blockType から schema の x-prompt-query を動的計算・_index は保存しない）
  SubStep 3-2（get_by_ids）: ids 一致の document の content を取得・存在しない id は skipped に記録

Step 4: 結果を { prompt, value } でラップする（shared/result）

Step 5: エラーを { error, prompt, message } でラップする（全例外を try/except）
```

**Guardrails（4カテゴリ）:**

入力検証:
- operation が `list_index` / `get_by_ids` 以外なら `INVALID_OPERATION` + prompt
- tags が namespace:value 形式でないものは警告して無視（致命にしない）

データアクセス:
- `.has-udd/documents/knowledge/` 配下のみ走査（パストラバーサル禁止）
- 存在しない id は skipped に記録し prompt で AI に通知（中断しない）

クエリ結果:
- 0件でも prompt 付きで返す（「該当 knowledge なし。タグや scope を見直してください」を AI に伝える）

Harness 原則:
- Python が走査・AI は生 JSON を読まない
- 全例外を捕捉し `{ error, prompt, message }` で返す（例外を AI に素通りさせない）

---

## 合意事項（全 K CLOSED）

| # | 合意 |
|---|---|
| K-1 | query-engine と別 engine（別ポート・別 MCP ツール）として公開・core 実装は共有 |
| K-2 | Index も prompt を返す（自己記述）・タグ規約 namespace:value・タグ合成せずドキュメント単位 authored prompt を機械グルーピング・2操作（list_index / get_by_ids） |
| K-3 | 動的集約・`_index.json` 廃止 |
| K-4 | structured JSON を返す（レンダリングしない）・x-render は HTML のみ・テキスト化は Orchestrator が delegation prompt 組成時に推論で |
| K-5 | core 共有（scan_directory=outbound / result・tags=shared）・ヘキサゴナル構成 |
| K-6 | Steps 5フェーズ・Guardrails 4カテゴリ |

---

## 次のアクション

`.has-udd/skills/harness-knowledge-engine.json`（document.json）作成。
※ 配置は `.has-udd/documents/skills/harness-knowledge-engine.json`（最新フォルダ構成）だが、ブレスト用サンプルは従来どおり作成。

---

## 合意事項

（論点解決後に記録）

---

## 次のアクション

K-1〜K-6 解決後 → `.has-udd/documents/skills/harness-knowledge-engine.json` 作成
