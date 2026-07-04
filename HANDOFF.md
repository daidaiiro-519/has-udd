# ハンドオフドキュメント — Waffle切り出し作業（2026-07-04）

**対象ブランチ:** `impl/bootstrap-core-loop`
**このドキュメント時点の最新コミット:** `c7776db`（ブレスト: 論点3フォローアップ(パス規約.waffle化・重複解消)を追記）

> ⚠️ **同じブランチで並行して作業している場合の注意**: このブランチに対して複数セッション/
> 複数人が同時に作業している可能性があります。pullした際に競合が起きた場合は、下記の
> 「今回変更した内容」を踏まえた上で、どちらの変更を残すか判断してください。特に
> `pyproject.toml`・`.gitignore`・`waffle/`配下のファイルパスは今回大きく変わっているため、
> 古い状態（`src/has_udd/`・`.has-udd/documents/`前提）をベースにした変更とはほぼ確実に
> 競合します。

---

## 1. 何をしたセッションか

大きく3本の作業が連続して発生した:

1. **LoomDB×has-udd文書DB連携のブレインストーム**（`docs/brainstorm/brainstorm-loomdb-has-udd-document-db.md`・全5論点合意済み）→ `docs/brainstorm/design-loomdb-document-conversion-poc.md`にPoC計画（Phase1〜4）を具体化。**まだ実装には着手していない**（計画段階）。
2. **ハーネスのイベントログ解析デバッグツール構想のブレインストーム**（`docs/brainstorm/brainstorm-context-observability.md`・全4論点合意済み）。こちらも**構想段階**（実装は未着手）。
3. **has-uddのOSS分離（engine / agent system）ブレインストーム→実行**（`docs/brainstorm/brainstorm-has-udd-oss-separation.md`・全5論点合意済み）。**こちらは実際にコードを変更済み**。今回の競合の主因はほぼ確実にこれ。

## 2. 実際にコードを変更した内容（3番目の作業）

### 2.1 パッケージ名の改名: `has_udd` → `waffle`

- `has-udd`という名前は `Harness Agentic Scrum Usecase-Driven-Development` の略で、
  HAS（agent system）とUDD（engineが支える開発手法）の合成語だった。
- engine部分（document.jsonのスキーマ検証・query・render・scaffold）を独立OSS
  **「Waffle」**として改名。「has-udd」という名前と、それが指す開発手法（UDD）は
  agent system側に残す。

### 2.2 ディレクトリの自己完結化

`loomdb/`と同じ位置付けで、engine一式を`waffle/`ディレクトリに集約:

```
旧: src/has_udd/ ・ tests/ ・ features/ ・ pyproject.toml ・ uv.lock（すべてrepo root直下）
新: waffle/src/waffle/ ・ waffle/tests/ ・ waffle/features/ ・ waffle/pyproject.toml ・ waffle/uv.lock
```

- repo rootからの呼び出しは **`uv run --project waffle waffle <command>`** の形に統一
  （旧: `has-udd <command>`のグローバルコマンド前提だったが、`waffle/`配下への切り出しに伴い変更）
- `waffle/README.md`・`waffle/LICENSE`・`waffle/CLAUDE.md`を新規作成（loomdbと同水準のOSS体裁）

### 2.3 document.jsonのパス規約: `.has-udd/` → `.waffle/`

これが最も広範囲に影響する変更:

- schemaの`x-source-target`/`x-render-target`（`SkillSchema`/`SpecSchema`/`CodingSchema`）が
  指すパス規約を`.has-udd/documents/...`から**`.waffle/documents/...`**に変更
  （schemaがWaffle自身の資産である以上、パス規約もWaffleのものであるべき、という判断）
- Waffle自身を説明するspec/skill document 14件（`harness-query-engine`・`harness-render-engine`・
  `stack`・`python-hexagonal`・`bc-waffle-engines`（旧`bc-has-udd-engines`から改名）・
  `sd-harness-core`・`sd-validation`・`sd-rendering`・`agg-document`・`agg-schema`・
  `uc-query-document`・`uc-render-document`・`uc-validate-document`・`uc-scaffold-document`）を、
  **`waffle/.waffle/documents/`に一元管理**（以前は repo root `.has-udd/documents/` と
  `waffle/.has-udd/documents/`（テストフィクスチャ）に重複コピーがあったが解消済み）
- **repo root側 `.has-udd/documents/` には、Waffle固有でない汎用skill
  （`analyze-domain-model.json`）だけが残る**
- `.claude/skills/harness-query-engine/SKILL.md`・`.claude/skills/harness-render-engine/SKILL.md`
  は、新しい正本（`waffle/.waffle/documents/`）から再レンダリングして更新済み

### 2.4 その他の細かい変更

- gen-gapマーカー: `has-udd:impl-start/end` → `waffle:impl-start/end`
  （`waffle/src/waffle/shared/tags.py`の`GENGAP_START`/`GENGAP_END`定数と、
  `CodingSchema/v1.json`の`x-coding-contract.genGap`の両方）
- CLIヘルプ文言・MCPサーバ名（`FastMCP("has-udd")` → `FastMCP("waffle")`）
- `.gitignore`に `waffle/.claude/` ・ `waffle/.waffle/skills/` を追加
  （behaveのrender.featureがテスト実行時にwaffle/配下へ副生成する成果物）

## 3. 現在の状態（このコミット時点で確認済み）

- `waffle/`単体: `uv run pytest` 15件 green・`uv run behave` 65シナリオ green
- repo rootからの実呼び出し: `uv run --project waffle waffle validate/render --path ...` で動作確認済み
- `git status`はclean（このドキュメント作成前時点）

## 4. 競合が起きた場合の判断基準

- `pyproject.toml`・`src/has_udd/`（または`src/waffle/`）・`tests/`・`features/`が
  repo root直下にある変更 → **古い構造**。`waffle/`配下への移動後の状態を優先してください。
- `.has-udd/documents/skills/harness-query-engine.json`等、上記14件のいずれかを
  repo root直下で編集している変更 → 内容は`waffle/.waffle/documents/`側に移す形で
  マージしてください（repo rootには残さない）。
- `.claude/skills/harness-query-engine/SKILL.md`・`.claude/skills/harness-render-engine/SKILL.md`
  → 手編集せず、`waffle/.waffle/documents/`のsourceを直して
  `uv run --project waffle waffle render --path waffle/.waffle/documents/skills/<name>.json`で
  再生成するのが正しい手順です。

## 5. 関連ドキュメント（背景を追うなら）

- `docs/brainstorm/brainstorm-has-udd-oss-separation.md` — 今回の改名・分離の意思決定の全経緯（論点1〜5＋追記）
- `waffle/CLAUDE.md` — Waffle側のプロジェクトメモリ（確定した意思決定）
- `CLAUDE.md`（repo root） — has-udd全体の構造メモ（loomdb/・waffle/への導線）
