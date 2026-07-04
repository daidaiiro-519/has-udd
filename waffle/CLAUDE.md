# Waffle — プロジェクトメモリ

スキーマという型で文書を焼き上げる、構造検証＋意味ガイダンス内蔵のドキュメントエンジン。
JSON Schemaでdocument.jsonを検証・query・render・scaffoldする。

## 確定した意思決定（ユーザー承認済み）

- **旧称は`has_udd`**（has-udd = Harness Agentic Scrum Usecase-Driven-Developmentの略）。
  頭字語は HAS（Harness Agentic Scrum＝agent system）と UDD（Usecase-Driven-Development＝
  engineが支える開発手法）の合成語だったため、UDD色はagent system側（has-uddという名前
  そのもの）に残し、engine部分には独立の名称「Waffle」を与えた。
  経緯は `docs/brainstorm/brainstorm-has-udd-oss-separation.md`（論点1〜5）を参照。
- **バンドルされているschema（SkillSchema/SpecSchema/CodingSchema/RenderMetaSchema）は
  Waffle自身の資産**（has-uddから借りた外部依存ではない）。`src/waffle/domain/model/`に
  同梱を維持する。schemaを外部化する設計は採用しない（ユーザー判断）。
- この`waffle/`ディレクトリは`loomdb/`と同じく**自己完結**しており、
  `git subtree split --prefix=waffle`でそのまま独立リポジトリに切り出せる想定。
- `document.json`のパス規約（`x-source-target`/`x-render-target`）は`.has-udd/`ではなく
  **`.waffle/`**（schema自身がWaffleの資産である以上、規約もWaffle自身のもの。
  `.git/`が道具の名前を冠するのと同じ発想）。Waffle自身を説明するspec/skill document
  （harness-query-engine・harness-render-engine・stack・python-hexagonal・
  bc-waffle-engines等、計14件）は`waffle/.waffle/documents/`に**一元管理**し、
  repo root側`.has-udd/documents/`との重複コピーは解消済み（旧トレードオフは解消）。
  repo root側`.has-udd/documents/`には、Waffle固有でない汎用skill
  （`analyze-domain-model.json`等・has-udd/agent system自身の資産）だけが残る。
- repo rootからの呼び出しは `uv run --project waffle waffle <command>` の形。
  Waffle自身のdocumentを`.claude/skills/`へdeployする際は、`waffle/`配下へのパスを明示して
  呼ぶ（例: `waffle render --path waffle/.waffle/documents/skills/harness-query-engine.json`）。

## 構造メモ

- ワークスペース: `src/waffle/`（domain/application/adapters・ヘキサゴナル）・
  `tests/`（pytest）・`features/`（behave）。
- schema解決は`PackageSchemaRepository`（`adapters/outbound/schema_repo.py`）が
  `importlib.resources`でパッケージ内`domain/model/`から行う（外部プロジェクトからの
  差し替え口は現状なし）。
