# AI コーディングツール フォルダ構成調査

**調査日:** 2026-06-21
**調査方法:** 各ツールの公式ドキュメントを WebFetch / WebSearch で直接取得
**目的:** has-udd のフォルダ構成を各ツールと互換性を持たせるための情報収集

---

## Claude Code

**出典:** https://code.claude.com/docs/en/settings

```
.claude/                        ← プロジェクトスコープ
├── settings.json               プロジェクト共有設定（git 管理）
├── settings.local.json         個人用ローカル設定（gitignore）
├── CLAUDE.md                   プロジェクトメモリ（git 管理）
├── CLAUDE.local.md             個人用メモリ（gitignore）
├── agents/                     サブエージェント定義
├── skills/                     カスタムスキル
│   └── {name}/SKILL.md
├── commands/                   カスタムコマンド
├── plans/                      プラン実行ファイル
└── .mcp.json                   MCP サーバー設定

~/.claude/                      ← ユーザースコープ
├── agents/
└── settings.json
```

**rules/ フォルダ:** なし（`settings.json` の permissions で制御）

---

## Kiro（AWS）

**出典:** https://kiro.dev/docs/steering/

```
.kiro/                          ← プロジェクトスコープ
├── steering/                   コンテキスト注入（*.md）
│   ├── product.md
│   ├── tech.md
│   ├── structure.md
│   ├── api-standards.md
│   ├── testing-standards.md
│   └── code-conventions.md
├── agents/                     カスタムエージェント（*.json）
├── hooks/                      フック設定
├── skills/                     スキル定義
│   └── {name}/SKILL.md         ← Claude Code と完全同一規約
├── specs/                      仕様書
│   └── {feature-name}/
│       ├── requirements.md
│       ├── design.md
│       └── tasks.md
└── prompts/                    再利用可能なプロンプト（*.md）

~/.kiro/                        ← ユーザースコープ
├── steering/
│   └── AGENTS.md
└── skills/
```

**rules/ フォルダ:** なし（`steering/` がルール定義に相当）

---

## GitHub Copilot

**出典:** https://docs.github.com/en/copilot/concepts/agents/about-agent-skills
         https://docs.github.com/en/copilot/reference/hooks-reference

```
.github/                        ← プロジェクトスコープ
├── copilot-instructions.md     全リクエストに適用されるリポジトリ指示
├── instructions/
│   └── NAME.instructions.md   パス特定の指示（*.instructions.md 必須）
├── agents/
│   └── {name}.agent.md        カスタムエージェント定義
├── skills/                     プロジェクト固有スキル（以下と等価）
│   └── {name}/SKILL.md
├── hooks/
│   └── *.json                 フック設定（JSON）
└── workflows/
    └── copilot-setup-steps.yml

# スキルは以下3パスすべてで等価に読み込まれる（公式確認）
.github/skills/{name}/SKILL.md
.claude/skills/{name}/SKILL.md   ← Claude Code パスも公式サポート
.agents/skills/{name}/SKILL.md   ← 汎用パスも公式サポート

~/.copilot/                     ← ユーザースコープ
├── skills/
├── hooks/
└── settings.json

# プロジェクトルート直下（複数ツール共通）
AGENTS.md                       Copilot / Codex / Kiro が読む
CLAUDE.md                       Claude Code / Copilot が読む
GEMINI.md                       Gemini / Copilot が読む
```

**hooks のイベント種別:**
`sessionStart`, `sessionEnd`, `userPromptSubmitted`, `preToolUse`, `postToolUse`,
`postToolUseFailure`, `agentStop`, `subagentStart`, `subagentStop`, `errorOccurred`, `preCompact`
（CLI 限定: `notification`, `permissionRequest`）

**rules/ フォルダ:** なし（`copilot-instructions.md` や `instructions/*.instructions.md` が相当）

---

## Codex（OpenAI）

**出典:** https://developers.openai.com/codex/rules
         https://developers.openai.com/codex/hooks
         https://developers.openai.com/codex/skills
         https://developers.openai.com/codex/subagents

```
~/.codex/                       ← ユーザースコープ
├── config.toml
├── AGENTS.md
├── rules/
│   └── default.rules           Starlark 形式の実行ポリシー
├── agents/
│   └── {name}.toml             カスタムエージェント定義
└── log/

.codex/                         ← プロジェクトスコープ（信頼済みの場合）
├── config.toml
├── hooks.json                  または config.toml の [hooks] テーブル
├── rules/
│   └── *.rules                 Starlark 形式（プロジェクト固有ルール）
└── agents/
    └── {name}.toml

# スキルは .codex/ ではなく .agents/ 配下（注意）
.agents/skills/                 ← プロジェクトスコープ（$REPO_ROOT）
    └── {name}/SKILL.md
~/.agents/skills/               ← ユーザースコープ
/etc/codex/skills/              ← システムスコープ

# プロジェクト内（ディレクトリ階層で検索）
AGENTS.md
AGENTS.override.md              最優先オーバーライド
```

**rules/ の詳細（Codex 固有機能）:**
- 言語: **Starlark**（Python に似た安全なスクリプト言語）
- 役割: AI が実行するコマンドの許可/拒否ポリシー
- `prefix_rule()` 関数で `allow` / `prompt` / `forbidden` を定義
- `codex execpolicy check` コマンドで複数 rules ファイルを検証可能

**hooks のイベント種別:**
`SessionStart`, `SubagentStart`, `PreToolUse`, `PermissionRequest`, `PostToolUse`,
`PreCompact`, `PostCompact`, `UserPromptSubmit`, `SubagentStop`, `Stop`

---

## ツール間の共通規約まとめ

### スキルパス（公式に等価とされるパス）

| パス | Claude Code | Kiro | Copilot | Codex |
|---|---|---|---|---|
| `.claude/skills/{name}/SKILL.md` | ✅ ネイティブ | — | ✅ **公式サポート** | — |
| `.kiro/skills/{name}/SKILL.md` | — | ✅ ネイティブ | — | — |
| `.github/skills/{name}/SKILL.md` | — | — | ✅ ネイティブ | — |
| `.agents/skills/{name}/SKILL.md` | — | — | ✅ **公式サポート** | ✅ ネイティブ |

→ **`.agents/skills/` は Copilot と Codex の両方が公式にネイティブ/サポート**

### フック設定パス

| | Claude Code | Kiro | Copilot | Codex |
|---|---|---|---|---|
| パス | `.claude/settings.json` | `.kiro/hooks/` | `.github/hooks/*.json` | `.codex/hooks.json` |
| 形式 | JSON | JSON | JSON | JSON / TOML |

### rules/ フォルダ

| ツール | rules/ あり | 内容 |
|---|---|---|
| Codex | ✅ `.codex/rules/*.rules` | Starlark 形式の実行ポリシー（allow/deny） |
| Claude Code | ❌ | `settings.json` の `permissions` で代替 |
| Kiro | ❌ | `steering/*.md` で代替 |
| Copilot | ❌ | `copilot-instructions.md` / `instructions/` で代替 |

### 共通ルートファイル

| ファイル | 読むツール |
|---|---|
| `AGENTS.md` | Copilot / Codex / Kiro |
| `CLAUDE.md` | Claude Code / Copilot |
| `GEMINI.md` | Gemini / Copilot |

---

## has-udd フォルダ設計への示唆（シンボリックリンク戦略）

### SSOT 設計

```
.has-udd/                       ← SSOT（Single Source of Truth）
├── schemas/                    ContractSkills（Schema ファイル）
│   ├── UsecaseSpec/v1.json
│   └── SBISpec/v1.json
├── documents/                  document.json（成果物）
│   └── uc-order-create.json
├── skills/                     HarnessSkills / QuerySkills / RenderSkills 定義
│   └── {name}/SKILL.md
├── hooks/                      フック設定
│   └── hooks.json
└── config.json                 has-udd 設定
```

### シンボリックリンクによる各ツール連携

```bash
# skills — 各ツールが読むパスへシンボリックリンク
.claude/skills     →  .has-udd/skills   （Claude Code + Copilot）
.kiro/skills       →  .has-udd/skills   （Kiro）
.agents/skills     →  .has-udd/skills   （Copilot + Codex ← これ1本で2ツールカバー）
.github/skills     →  .has-udd/skills   （Copilot ← .agents/ があれば省略可）

# hooks — 各ツールが読むパスへシンボリックリンク
.kiro/hooks        →  .has-udd/hooks    （Kiro）
.github/hooks      →  .has-udd/hooks    （Copilot）
.codex/hooks.json  →  .has-udd/hooks/hooks.json  （Codex）

# コンテキストファイル — has-udd が生成・更新
AGENTS.md          has-udd が自動生成（Copilot / Codex / Kiro が読む）
CLAUDE.md          has-udd が自動生成（Claude Code / Copilot が読む）

# ドキュメント出力
docs/              RenderSkills 出力 MD（全ツールが自然に読める）
```

### ポイント

1. `.agents/skills/` → `.has-udd/skills/` の1本のシンボリックリンクで **Copilot + Codex の両方をカバー**（最もコスパが高い）
2. Claude Code は `.claude/skills/` を読むため別途シンボリックリンクが必要
3. `rules/` は Codex 固有機能（Starlark 言語）。has-udd が Codex 向けに `.codex/rules/` を生成する設計が選択肢に入る
