# has-udd 実装アーキテクチャ設計（ヘキサゴナル・フォルダ構造）

## 目的

has-udd を実装する際のソースコード構造・フォルダ構造を、ヘキサゴナルアーキテクチャに沿って確定する。
パッケージ側（OSS 本体）とユーザープロジェクト側（`.has-udd/`）の両方を扱う。

---

## 前提: has-udd の核となる構造認識

### ドメインコア = JSON Schema（コードではない）

has-udd は通常のシステムと異なり、**ドメインモデル本体が Python コードではなく `schemas/`（JSON Schema）**。

| 通常のシステム | has-udd |
|---|---|
| ドメインモデル = コード（クラス・振る舞い） | ドメインモデル = **schema（データ）** |
| 不変条件 = コードのバリデーション | 不変条件 = **JSON Schema + if/then/else** |
| 振る舞い = メソッド | 振る舞い = **Agent / Custom Skill document（データ）+ AI 推論** |
| インフラ = アダプター（コード） | インフラ = **engine（汎用 Python）** |

→ engine を汎用設計してきた（`blockType+"Block"`・セマンティック操作）のは「ドメイン知識は schema に・Python は汎用ハーネス」の必然。
→ **schema 設計こそがドメインモデリング**。

### Harness の本質

```
Harness（Python）= 汎用・不変・ドメインを知らない
Domain（schema）  = データ・ドメインモデルそのもの
Intelligence（AI）= schema が固定した構造に値を入れる・推論する
```

---

## 2ヘキサゴン・ポート合成（混乱回避の最重要認識）

engine は**2つのヘキサゴンに同時に属し、見るフレームで役割名が変わる**（矛盾ではない・相対概念）。

| | ①エージェントシステム | ②engine 実装（ソースコード） |
|---|---|---|
| Core | Orchestrator / Role | engine ロジック |
| engine の役割 | **Secondary Adapter** | **Application（use case）** |

```
ヘキサゴン①（エージェント）             ヘキサゴン②（engine 実装）
┌──────────────────┐        ┌────────────────────┐
│ Core: Orchestrator/Role │        │ Inbound Adapter: cli/mcp │
│   │ 呼ぶ              │  同じ   │   ↓ 呼ぶ               │
│ Secondary Port（契約）──┼─境界──┼→ Application: query/…  │
│   │                   │        │   ↓ 使う               │
│ Secondary Adapter ════╪═ ②全体 │ Domain + Outbound(fs)  │
└──────────────────┘        └────────────────────┘
```

- ①の Secondary Adapter の中身が、まるごと②というヘキサゴン（入れ子）
- ①の Secondary Port（engine の契約）と ②の Inbound Port（cli/mcp 入口）は**同じ境界の裏表**（ポート合成）
- Orchestrator が知るのは**ポート（engine の input/output 契約 = Interface ブロック）だけ**。②の内部は不可視
- 「Adapter / Application」は絶対属性でなく **Core への相対概念**。別ヘキサゴンで別役は正常
- 一般性: マイクロサービス（A→API→B）が正準例。ポート合成は定石
- 規律: 「どちらのヘキサゴンの話か」を常に明示・契約（ポート）を SSOT にする

---

## パッケージ側構成（OSS リポジトリ・配布される has-udd 本体）

```
has-udd/                                   # 1つのヘキサゴナルアーキテクチャ
├── src/has_udd/                           # has_udd = パッケージ名（import 根・名前空間）
│   ├── domain/                            # 【Domain 層】
│   │   ├── model/                         #   ドメインモデル（JSON Schema・配布しない）
│   │   │   ├── SkillSchema/v1.json
│   │   │   ├── AgentSchema/v1.json
│   │   │   ├── UsecaseSpecSchema/v1.json
│   │   │   ├── DomainModelSpecSchema/v1.json
│   │   │   ├── KnowledgeSchema/v1.json
│   │   │   └── CodingSchema/v1.json
│   │   ├── ports/                         #   Driven Port（document_repository / schema_repository）
│   │   └── services/                      #   Domain Service（schema_integrity = 集約横断検証）
│   │
│   ├── application/                       # 【Application 層】= engine の use case
│   │   ├── query.py                       #   harness-query-engine
│   │   ├── render.py                      #   harness-render-engine
│   │   ├── knowledge.py                   #   harness-knowledge-engine
│   │   ├── scaffold.py                    #   harness-scaffold-engine（旧 spec・create + validate）
│   │   └── audit.py                       #   harness-audit-engine（旧 contract・I/O trace + 契約適合）
│   │                                      #   （template/coding は Phase 3 まで延期）
│   │
│   ├── adapters/
│   │   ├── inbound/                       # 【Primary/Driving Adapter】2入口
│   │   │   ├── cli/                       #   typer
│   │   │   └── mcp/                       #   fastmcp
│   │   └── outbound/                      # 【Secondary/Driven Adapter】
│   │       ├── fs.py                      #   ファイル I/O（scan_directory 等）
│   │       ├── jinja_renderer.py          #   x-render テンプレート展開
│   │       └── jsonschema_validator.py    #   Draft202012Validator
│   │
│   └── shared/                            # 【横断的関心】
│       ├── result.py                      #   { prompt, value } / { error, prompt, message }
│       └── tags.py                        #   namespace:value マッチング
│
├── resources/                            # バンドル OSS document（engine skills / 既定 Role / framework knowledge の document.json）
├── tests/
└── pyproject.toml
```

**依存の向き（内向き）:** inbound → application → domain(ports/services) ← outbound（port 実装）。schema(JSON) を application が契約として読む。

**命名空間:** `src/has_udd/` の `has_udd` はパッケージ名（import 根）。`src/domain/` だと `domain` が世界に露出して衝突するため不可。`src/` は src レイアウト（誤 import 防止）。

### schema は配布しない（パッケージ内に閉じる）

- schema は `src/has_udd/domain/model/` に閉じ、ユーザープロジェクトに配布しない
- engine が `importlib.resources` でパッケージから解決
- AI は schema を直接読まない（Harness 原則）。engine が schema を読んで「空 skeleton + x-prompt-write」を生成 → AI が値を埋める。だから schema がユーザー側に居る必要がない
- **クラス/インスタンス分離**: schema（クラス・フレームワーク所有）/ document.json（インスタンス・ユーザープロジェクト）。document は `schemaRef` で参照するだけ
- 利点: ユーザーが内部 schema を触れない・同期ずれなし・`uvx has-udd@version` でバージョン固定
- ユーザーが schema を見たいときは engine コマンド（例 `has-udd schema show`）で提示

---

## ユーザープロジェクト側構成（has-udd init が生成）

**原則: `.has-udd/` が canonical（source + rendered）。source（document.json）は `documents/` に分離・集約フォルダは rendered。ツール固定パスへは deploy（コピー・symlink 非依存）。**

```
ユーザープロジェクト/
│
├── .has-udd/                          # canonical（has-udd の世界）
│   ├── config.json
│   │
│   ├── documents/                    # 【SOURCE】document.json（集約ごと・内部・あまり目に触れない）
│   │   ├── skills/<id>.json
│   │   ├── specs/<id>.json
│   │   ├── knowledge/<id>.json
│   │   ├── agents/<id>.json
│   │   └── coding/<id>.json
│   │
│   ├── skills/<name>/SKILL.md         # 【RENDERED】tool-recognized
│   ├── agents/<name>.md               # 【RENDERED】フラット .md
│   ├── specs/<id>.html                # 【RENDERED】人間向け HTML（集約フォルダ内で完結）
│   ├── knowledge/<id>.html            # 【RENDERED】人間向け HTML
│   │
│   ├── hooks/                         # Hooks 設定（Phase 5）
│   └── traces/                        # audit-engine .trace.json（.gitignore）
│
│   # deploy（コピー・rendered のみ・document.json 除外）→ ツール固定パス
├── .claude/skills/<name>/SKILL.md     # 各ツールは .has-udd を見ない・symlink 非推奨 → 実ファイルコピー
├── .claude/agents/<name>.md           # Phase 6: .github/ .agents/ にも複製
├── CLAUDE.md / AGENTS.md              # ルート（Primary Port・init 生成）
└── （coding rendered → 実コードファイルへ DocComment 注入・.has-udd 外）

# schema は .has-udd に置かない（パッケージ内 domain/model/）
```

### 確定事実（claude-code-guide 確認済み）

| 項目 | 事実 |
|---|---|
| Skill | `.claude/skills/<name>/SKILL.md`（フォルダ per skill・他ファイルは無視され安全） |
| Subagent | `.claude/agents/<name>.md`（フラット .md・非 .md 無視・サブフォルダは識別に無関係） |
| symlink | ドキュメント化されていない・非推奨 → **rendered は実ファイルで deploy** |

### 役割分担

| 領域 | 役割 |
|---|---|
| `.has-udd/documents/` | source 保管庫（document.json・内部） |
| `.has-udd/{skills,agents,specs,knowledge}/` | rendered（canonical・表） |
| `.claude/` `.github/` `.agents/` | deploy 先（ツール認識用コピー・生成物） |
| `CLAUDE.md` / `AGENTS.md` | Primary Port（ルート・ツール要求で強制） |

---

## 後で詳細化（ブロッカーではない）

1. **id/name 一貫性**: documentId が source ファイル名と rendered フォルダ名を駆動。命名を documentId で統一
2. **deploy 同期**: `.has-udd/` の rendered（canonical）→ `.claude/` のコピーは、document 変更→再 render 時に再 deploy（render の一部にする等）
3. **.gitignore 戦略**: documents/=commit / rendered・deploy コピー・traces=derived
4. **各集約フォルダ内の詳細構造**: その集約の設計時に詰める
5. **outbound の port 抽象度**: 最初から full ceremony にせず「application + 薄い fs アクセス」で始め、テスタビリティが要る所から port 化する漸進案
6. **multi-tool deploy**（Phase 6）: `.claude/` `.github/` `.agents/` への複製方式

---

## 技術スタック（確定済み）

| 用途 | ライブラリ |
|---|---|
| MCP サーバー | fastmcp（Anthropic 公式） |
| CLI | typer + rich |
| Schema バリデーション | jsonschema（Draft202012Validator） |
| JsonPath クエリ | jsonpath-ng.ext（filter 対応） |
| テンプレート展開 | jinja2 |
| 配布 | uvx has-udd |

---

## 実装ロードマップ

```
① 残りエンジンブレスト（knowledge ← ほぼ完了 / scaffold / audit）（template/coding は Phase 3 まで延期）
② プロジェクト構成設計（ヘキサゴナル骨格・shared 共通基盤・共通エラーハンドリング）
③ application/ 実装（全エンジン）
④ inbound adapter（cli=typer）実装
⑤ inbound adapter（mcp=fastmcp）実装
```
