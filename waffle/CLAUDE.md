# Waffle — プロジェクトメモリ

スキーマという型で文書を焼き上げる、構造検証＋意味ガイダンス内蔵のドキュメントエンジン。
JSON Schemaでdocument.jsonを検証・query・render・scaffoldする。

## ★最優先ルール: document.json操作は必ずCLI/MCP経由（自分自身がdogfoodの対象）

waffleの`.waffle/documents/`配下は**waffle自身が実装したengineで操作する**。
Claude Code自身がwaffleの最初のユーザーであり、ここでの振る舞いがそのままdogfoodになる。

- **document.jsonの作成**: `uv run waffle scaffold --operation create ...` →
  `--operation fill ...`（Write/Editで直接JSONを書き起こさない）
- **既存document.jsonの検証**: `uv run waffle validate --path ...`
- **document.jsonの内容確認・検索**: `uv run waffle query --operation get_block/get_field/...`
  （`cat`/`grep`/`python -c "json.load(...)"`で直接読まない）
- **成果物への描画**: `uv run waffle render --path ...`
- **spec整合性・シナリオドリフト・schema版ドリフトの確認**:
  `uv run waffle check-spec-integrity` / `check-scenario-drift` / `check-schema-version-drift`
- **ソースコードのdocstring確認**: `uv run waffle scan-source-code` / `lint-docstring`

**訂正（実測済み）**: 「既存document.jsonは編集engineが無い」は誤りだった。`scaffold fill`は
`documentPath`が指す既存document（作成直後に限らない）に対して、schemaが`x-prompt-write`を
宣言した任意のpathへ値を書き込める。**配列フィールドは丸ごと置き換え**なので、
`content.valueObjects.items`のような配列パスに「既存要素＋追加/削除後の完全な配列」を
valuesとして渡せば、値オブジェクトの追加・削除・entity属性の追加等はfillで行える
（`waffle query`で現在値を取得→配列を組み立てる→`waffle scaffold fill`で書き込む、という手順）。
実測でCLI経由の追加・削除・再validateまで動作確認済み。

**残る唯一の例外**: schema自体が更新され、既存document.jsonに元から無かった
トップレベルキーを新規追加する場合のみ。`fill`は対象キーが既にdocument内に存在する
前提（`path in doc`）でしか書けず、`create`も既存documentがあれば中身を保護して
上書きしない（新規必須フィールドの後追いマージはしない）。この場合に限りEdit/Writeを
許容するが、頻発するようなら編集用usecaseの新設を検討する。

## 確定した意思決定（詳細は各ブレストdocを参照。ここでは結論だけ）

- 旧称`has_udd`から改名。経緯: `docs/brainstorm/brainstorm-has-udd-oss-separation.md`
- バンドルschema（SkillSchema/DomainSpecSchema/PresentationSpecSchema/CodingSchema/
  RenderMetaSchema/DocstringSchema）はWaffle自身の資産（外部化しない）。
  - `DomainSpecSchema`=業務ロジック層（DDD・bounded-context/subdomain/aggregate/usecase）、
    `PresentationSpecSchema`=プレゼンテーション層（非DDD・screen/flow）。
    経緯: `docs/brainstorm/brainstorm-schema-aggregate-zerobase.md`
  - Schema集約（agg-schema）の対象は「Documentのschemaが指しうる型」のみ。`src/waffle/domain/model/`。
  - RenderMetaSchema（値オブジェクト・`domain/value_objects/`）とDocstringSchema
    （usecase出力DTO・`application/dto/`）は集約ではない。
  - schemaのバージョン移行機構（x-migration/MigrationEngine）は撤去済み。ドリフト検知
    （`uc-check-schema-version-drift`）のみ維持。経緯: `docs/brainstorm/brainstorm-schema-versioning-migration.md`
  - reconcile系（spec整合性/シナリオドリフト/schema版ドリフト）は全て正式engine化済み
    （一時スクリプトは全廃）。
- この`waffle/`ディレクトリは自己完結（`git subtree split --prefix=waffle`で独立可能）。
- `document.json`のパス規約は`.waffle/`（`.has-udd/`ではない）。Waffle自身を説明する
  spec/skill documentは`waffle/.waffle/documents/`に一元管理。repo root側`.has-udd/documents/`
  にはWaffle固有でない汎用skillのみ残る。
- repo rootからの呼び出しは `uv run --project waffle waffle <command>` の形。
  `.claude/skills/`へdeployする際は`waffle/`配下へのパスを明示（例:
  `waffle render --path waffle/.waffle/documents/skills/harness-query-engine.json`）。

## 構造メモ

- ワークスペース: `src/waffle/`（domain/application/adapters・ヘキサゴナル）・
  `tests/`（pytest。unit/integration/acceptance/contractの4層。behaveは全撤去済み）。
- schema解決は`PackageSchemaRepository`（`adapters/outbound/schema_repo.py`）が
  `importlib.resources`でパッケージ内`domain/model/`（集約）・`domain/value_objects/`
  （値オブジェクト型定義）・`application/dto/`（usecase出力形状）の3箇所から行う
  （外部プロジェクトからの差し替え口は現状なし）。
