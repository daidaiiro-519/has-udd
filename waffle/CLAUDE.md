# Waffle — プロジェクトメモリ

スキーマという型で文書を焼き上げる、構造検証＋意味ガイダンス内蔵のドキュメントエンジン。
JSON Schemaでdocument.jsonを検証・query・render・scaffoldする。

## 確定した意思決定（ユーザー承認済み）

- **旧称は`has_udd`**（has-udd = Harness Agentic Scrum Usecase-Driven-Developmentの略）。
  頭字語は HAS（Harness Agentic Scrum＝agent system）と UDD（Usecase-Driven-Development＝
  engineが支える開発手法）の合成語だったため、UDD色はagent system側（has-uddという名前
  そのもの）に残し、engine部分には独立の名称「Waffle」を与えた。
  経緯は `docs/brainstorm/brainstorm-has-udd-oss-separation.md`（論点1〜5）を参照。
- **バンドルされているschema（SkillSchema/DomainSpecSchema/PresentationSpecSchema/
  CodingSchema/RenderMetaSchema/DocstringSchema）はWaffle自身の資産**
  （has-uddから借りた外部依存ではない）。schemaを外部化する設計は採用しない（ユーザー判断）。
  - `SpecSchema`は`DomainSpecSchema`に改名済み（spec は DDD より広い上位概念であり、
    UI層を扱う非DDDの`PresentationSpecSchema`と対で「Spec家族」を構成するため）。
    `DomainSpecSchema`=業務ロジック層（DDD管轄・specKind=bounded-context/subdomain/
    aggregate/usecase）、`PresentationSpecSchema`=プレゼンテーション層（非DDD管轄・
    specKind=screen/flow・ビジュアルはFigma等へのURL参照のみ）。
    経緯は`docs/brainstorm/brainstorm-schema-aggregate-zerobase.md`を参照。
  - **Schema集約（agg-schema）の対象は「Documentのschemaが指しうる型」のみ**
    （DomainSpecSchema/PresentationSpecSchema/CodingSchema/SkillSchema）。独自の識別
    （documentType）を持つ。`src/waffle/domain/model/`に置く。
  - **RenderMetaSchema/DocstringSchemaは集約ではない**（identityを持たない）。
    RenderMetaSchemaは他schemaのブロックに埋め込まれる値オブジェクト（x-render宣言）の
    型定義で`src/waffle/domain/value_objects/`に置く。DocstringSchemaはusecase
    (uc-scan-source-code)の出力データの形状定義であり、業務ロジック(domain)ではなく
    usecaseの入出力契約(application)の関心事なので`src/waffle/application/dto/`に置く。
  - **schemaのバージョン移行機構（x-migration語彙・MigrationEngine）は撤去済み**
    （実際にx-migrationを必要とした実schemaが無く、各schemaの実document数も少数のため、
    機械的な一括移行は過剰と判断。ドリフト検知(`uc-check-schema-version-drift`・
    `waffle check-schema-version-drift`)のみ維持し、schema進化への追従はAIが個別に
    判断して直す）。経緯は`docs/brainstorm/brainstorm-schema-versioning-migration.md`の
    後日談を参照。
  - **scripts/配下に一時的に置いていた3つのドリフト検知スクリプト
    （check_spec_referential_integrity.py/check_scenario_drift.py/
    check_schema_version_drift.py）は全て正式なusecase/engineに昇格し撤去済み**
    （`uc-check-spec-integrity`/`uc-check-scenario-drift`/`uc-check-schema-version-drift`。
    いずれもCLI/MCP経由で呼ぶ。「一時的な独立スクリプト」自体が二重実装によるドリフト源に
    なりうるため、reconcileの仕組みは全てengineとして一箇所に統合する）。
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
  `tests/`（pytest。unit/integration/acceptance/contractの4層。behaveは全撤去済み）。
- schema解決は`PackageSchemaRepository`（`adapters/outbound/schema_repo.py`）が
  `importlib.resources`でパッケージ内`domain/model/`（集約）・`domain/value_objects/`
  （値オブジェクト型定義）・`application/dto/`（usecase出力形状）の3箇所から行う
  （外部プロジェクトからの差し替え口は現状なし）。
