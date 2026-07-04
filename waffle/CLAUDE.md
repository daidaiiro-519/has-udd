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
- `waffle/.has-udd/documents/`には、Waffle自身を説明するspec/skill document
  （harness-query-engine・harness-render-engine・stack・python-hexagonal・
  bc-has-udd-engines等、計14件）のコピーを、Waffle単体でテストが完結するように
  フィクスチャとして保持する。これらはrepo root側`.has-udd/documents/`にある
  「本物」（`.claude/skills/`へのdeploy元）と重複するが、render先パス解決の
  複雑化を避けるための現実的な選択（既知のトレードオフ。source更新時は両方に反映）。
- repo rootからの呼び出しは `uv run --project waffle waffle <command>` の形。

## 構造メモ

- ワークスペース: `src/waffle/`（domain/application/adapters・ヘキサゴナル）・
  `tests/`（pytest）・`features/`（behave）。
- schema解決は`PackageSchemaRepository`（`adapters/outbound/schema_repo.py`）が
  `importlib.resources`でパッケージ内`domain/model/`から行う（外部プロジェクトからの
  差し替え口は現状なし）。
