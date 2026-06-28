# ブレインストーミング: has-udd マルチツール互換性設計

**目的:** .has-udd/ を SSOT として Claude Code / Kiro / GitHub Copilot / Codex の各ツールと互換性を持つフォルダ構成・ファイル形式を確定する
**モード:** アイデア発散

---

## 前提: 調査結果サマリー

### Skills — 拡張子は全ツール統一 ✅

| ツール | パス | 拡張子 |
|---|---|---|
| Claude Code | `.claude/skills/{name}/SKILL.md` | `.md` |
| Kiro | `.kiro/skills/{name}/SKILL.md` | `.md` |
| Copilot | `.github/skills/{name}/SKILL.md` | `.md` |
| Codex | `.agents/skills/{name}/SKILL.md` | `.md` |

**重要:** Copilot は `.github/skills/` `.claude/skills/` `.agents/skills/` の3パスを**等価**として公式サポート。

### Agents — 拡張子・形式がバラバラ ❌

| ツール | パス | 拡張子 | 形式 |
|---|---|---|---|
| Claude Code | `.claude/agents/{name}.md` | `.md` | Markdown + YAML frontmatter |
| Kiro | `.kiro/agents/*.json` | `.json` | JSON |
| Copilot | `.github/agents/{name}.agent.md` | `.agent.md` | Markdown（別 suffix） |
| Codex | `.codex/agents/{name}.toml` | `.toml` | TOML |

**Claude Code の agent.md frontmatter 必須フィールド:** `name`, `description`
**任意フィールド:** `tools`, `model`, `permissionMode`, `maxTurns`, `skills`, `mcpServers`, `hooks`, `memory`, `background`, `effort`, `isolation`, `color`, `initialPrompt`

### Hooks — JSON ベースで近いがイベント名・パス・スキーマが違う △

| ツール | パス | 形式 | イベント名規約 |
|---|---|---|---|
| Claude Code | `.claude/settings.json` 内 `hooks` キー | JSON（埋め込み） | camelCase |
| Kiro | `.kiro/hooks/*.json` | JSON | 要確認 |
| Copilot | `.github/hooks/*.json` | JSON | camelCase (`preToolUse`) |
| Codex | `.codex/hooks.json` または `config.toml [hooks]` | JSON / TOML | PascalCase (`PreToolUse`) |

### rules/ フォルダ — Codex のみ固有機能

| ツール | rules/ | 内容 |
|---|---|---|
| Codex | `.codex/rules/*.rules` | Starlark 言語のコマンド実行ポリシー（allow/deny） |
| 他3ツール | なし | instructions.md / steering/ / settings.json で代替 |

---

## 論点 1: Skills の互換戦略

**問い:** .has-udd/skills/ を SSOT として各ツールに認識させる最善の方法は何か？

### AI 初期見解
**見解:** シンボリックリンク3本で全4ツールをカバーできる。追加実装コストゼロ。
**根拠:**
- Skills は全ツールが `SKILL.md`（フォルダ形式 `<name>/SKILL.md`）を使う唯一の統一フォーマット
- has-udd 側のレンダリング出力 `.has-udd/skills/<name>/SKILL.md` をそのまま指せる（同一フォーマット = init 時シンボリックリンク）
- Copilot が `.claude/skills/` と `.agents/skills/` を公式にサポートするため、`.agents/` シンボリックリンク1本で Copilot + Codex を同時カバーできる
- `.claude/skills/` → `.has-udd/skills/`、`.kiro/skills/` → `.has-udd/skills/`、`.agents/skills/` → `.has-udd/skills/` の3本で完結

**シンボリックリンク構成案:**
```bash
.claude/skills   →  .has-udd/skills   # Claude Code + Copilot
.kiro/skills     →  .has-udd/skills   # Kiro
.agents/skills   →  .has-udd/skills   # Codex + Copilot（重複だが問題なし）
```

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

### AI 再考見解
**見解:** {{修正・深化された立場}}

### 合意決定
**決定:** symlink は採用しない。**deploy は render に内蔵した自動 copy**（同一フォーマット=verbatim copy / 別フォーマット=transform copy）。deploy 先（.claude/ 等）は .gitignore。
**理由:** (1) claude-code-guide 確認で symlink は未ドキュメント（`.claude/skills` 全体 symlink は探索が辿らず不可・個別/per-skill は動くが将来壊れうる）。(2) 更新漏れは「symlink でしか防げない」のでなく **deploy を render に内蔵すれば copy でも防げる**（render で canonical 生成と同時に deploy）。(3) agents 等は別フォーマット変換 copy が不可避なので、全て copy の1機構に統一する方がシンプル。
**次のアクション:** render engine に deploy 内蔵（x-render-target.deploy）。本決定で本論点 CLOSED。
（注: 上の「シンボリックリンク構成案」は初期見解であり superseded）

---

## 論点 2: Agents の互換戦略

**問い:** 拡張子・形式がバラバラな Agents を .has-udd/ で SSOT 管理する方法は何か？

### AI 初期見解
**見解:** SSOT フォーマットを Claude Code 形式（Markdown + YAML frontmatter）に統一し、has-udd が各ツール向けに変換生成（transpile）する。シンボリックリンクでは解決できない。

**根拠:**
- Claude Code と Copilot はどちらも Markdown ベースだが、ファイルサフィックスが `.md` vs `.agent.md` で異なる
- Kiro は JSON、Codex は TOML — フォーマット自体が異なるため変換が必須
- SSOT として最もリッチなフィールド定義を持つのが Claude Code（15フィールド）なので Claude Code 形式を基底にするのが自然

**変換マッピング案:**
```
.has-udd/agents/{name}.md      ← RENDERED（document.json → Claude Code 形式 .md にレンダリング）
  ↓ has-udd が変換生成（Phase 6・別フォーマット変換）
.kiro/agents/{name}.json       ← Kiro 向け
.github/agents/{name}.agent.md ← Copilot 向け（.agent.md リネーム）
.codex/agents/{name}.toml      ← Codex 向け（TOML 変換）
```

注: `.has-udd/agents/<name>.md` はフラット配置（Skill のフォルダ形式とは異なる）。SOURCE は `.has-udd/documents/agent/<id>.json`、これを has-udd がレンダリングして上記 `.md` を生成する。

**Claude Code は `.claude/agents/` をシンボリックリンクで代替できるか？**
→ `.claude/agents/` → `.has-udd/agents/` でシンボリックリンク可能（同一フォーマット・Claude Code は `.md` をフラットに読む）
→ Copilot の `.github/agents/{name}.agent.md` は別フォーマットのため has-udd が変換生成する必要がある

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

### AI 再考見解
**見解:** {{修正・深化された立場}}

### 合意決定
**決定:** {{合意した内容}}
**理由:** {{なぜこれに合意したか}}
**次のアクション:** {{次のステップ}}

---

## 論点 3: Hooks の互換戦略

**問い:** イベント名・パス・スキーマが異なる Hooks を .has-udd/ で SSOT 管理する方法は何か？

### AI 初期見解
**見解:** Hooks は形式が近い（JSON ベース）が、イベント名の大文字規約とスキーマが異なる。SSOT を独自フォーマットで定義し、has-udd が各ツール向けに変換生成するのが現実的。ただし **hooks の互換は優先度を下げてよい**と考える。

**根拠:**
- Hooks はプロジェクト固有の CI/CD・検証スクリプトを呼ぶことが多く、内容がツール中立で書けるケースは限られる
- イベント種別のマッピングは概念的に近い（preToolUse ≒ PreToolUse）が、引数スキーマが異なる
- Skills と Agents が揃えば基本的なワークフローは動く。Hooks は「あれば便利」レベル

**最小限の互換案:**
```
.has-udd/hooks/hooks.json       ← SSOT（Copilot の camelCase 形式ベース）
  ↓ has-udd が変換生成
.kiro/hooks/has-udd.json        ← Kiro 向け
.codex/hooks.json               ← Codex 向け（PascalCase に変換）
# Claude Code は settings.json 内のため別途注入
```

**論点:** has-udd 自身が定義する Hooks を優先し、ユーザーカスタム Hooks は各ツールで直接書いてもらう割り切りもあり。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

### AI 再考見解
**見解:** {{修正・深化された立場}}

### 合意決定
**決定:** {{合意した内容}}
**理由:** {{なぜこれに合意したか}}
**次のアクション:** {{次のステップ}}

---

## 論点 4: .has-udd/ の SSOT フォルダ構成確定

**問い:** 上記を踏まえ .has-udd/ 配下の最終フォルダ構成をどう定めるか？

### AI 初期見解
**見解:** 以下の構成を提案する。

**注: JSON Schema（ドメインコア）は配布物に含めない。** Schema はパッケージ内 `src/has_udd/domain/model/` に置かれ、`.has-udd/` 配下には `schemas/` を作らない。

```
.has-udd/
├── documents/                  SOURCE: document.json（5集約: Skill/Spec/Knowledge/Agent/Coding）
│   └── {type}/
│       └── <id>.json
├── skills/                     RENDERED: SKILL.md（フォルダ形式・engine Skills / custom）
│   └── {name}/
│       └── SKILL.md
├── agents/                     RENDERED: Agent 定義（document.json → Claude Code 形式 .md・フラット）
│   └── {name}.md
├── specs/                      RENDERED（人間向け）: Spec の HTML
│   └── <id>.html
├── knowledge/                  RENDERED（人間向け）: Knowledge の HTML
│   └── <id>.html
├── traces/                     I/O トレース出力（harness-audit-engine）
├── hooks/                      Hooks 定義（Copilot camelCase 形式を SSOT）
│   └── hooks.json
└── config.json                 has-udd 設定

# シンボリックリンク（has-udd init 時に生成・同一フォーマット）
.claude/skills     →  .has-udd/skills
.claude/agents     →  .has-udd/agents
.kiro/skills       →  .has-udd/skills
.agents/skills     →  .has-udd/skills   ← Codex + Copilot

# has-udd が変換生成するファイル（別フォーマット・シンボリックリンク不可・Phase 6）
.kiro/agents/{name}.json
.github/agents/{name}.agent.md
.codex/agents/{name}.toml
.github/hooks/*.json
.codex/hooks.json

# Primary Port（リポジトリルート・has-udd が生成）
CLAUDE.md          （Claude Code / Copilot が読む）
AGENTS.md          （Copilot / Codex / Kiro が読む）
```

**根拠:**
- ドメインコア（JSON Schema）はパッケージ内（`src/has_udd/domain/model/`）にあり配布されない。`.has-udd/` には `schemas/` を置かない
- SOURCE は `documents/{type}/<id>.json` のみ。skills/agents/specs/knowledge はすべて document.json からのレンダリング出力（RENDERED）
- 同一フォーマット（Skills / Claude Code Agents）はシンボリックリンク、別フォーマット（Kiro / Copilot / Codex）は変換生成で対応する役割分担が最もシンプル
- Primary Port は CLAUDE.md / AGENTS.md（リポジトリルート）

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

### AI 再考見解
**見解:** {{修正・深化された立場}}

### 合意決定
**決定:** {{合意した内容}}
**理由:** {{なぜこれに合意したか}}
**次のアクション:** {{次のステップ}}

---

## 論点 5: rules/ の扱い（Codex 固有機能）

**問い:** Codex 固有の `.codex/rules/` を has-udd でどう扱うか？

### AI 初期見解
**見解:** has-udd の OSS スコープ外とし、ユーザーが直接 `.codex/rules/` に書く運用でよい。

**根拠:**
- `rules/` は Codex 限定の **Starlark 言語**で書くコマンド実行ポリシー（allow/deny）
- 他3ツールに対応する概念がなく、has-udd で抽象化しても意味が薄い
- 「AI が実行できるコマンドの許可リスト」はプロジェクトのセキュリティポリシーであり、has-udd が自動生成すべきものではない
- ただし has-udd の `config.json` に `codex.rulesPath` などで参照先を設定できるようにする拡張は将来的にあり得る

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

### AI 再考見解
**見解:** {{修正・深化された立場}}

### 合意決定
**決定:** {{合意した内容}}
**理由:** {{なぜこれに合意したか}}
**次のアクション:** {{次のステップ}}
