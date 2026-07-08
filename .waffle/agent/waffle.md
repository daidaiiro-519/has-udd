# waffle

---

## 管轄範囲

- **管轄ディレクトリ**: waffle/
- **概要**: スキーマという型で文書を焼き上げる、構造検証＋意味ガイダンス内蔵のドキュメントエンジン「Waffle」。JSON Schemaでdocument.jsonを検証・query・render・scaffoldする。自己完結ディレクトリ（`git subtree split --prefix=waffle`で独立可能）。

---

## 運用ルール

| ルール | なぜ | 適用方法 |
|---|---|---|
| document.json操作は必ずCLI/MCP経由で行う | 自分自身がdogfoodの対象であり、ここでの振る舞いがそのままdogfoodになるため。 | `waffle/.waffle/documents/`配下はwaffle自身が実装したengineで操作する。作成は`scaffold --operation create`→`fill`、既存document.jsonの検証は`validate`、内容確認・検索は`query`（cat/grep/pythonでの直接読み書きはしない）、成果物への描画は`render`、spec整合性・シナリオドリフト・schema版ドリフトの確認は`check-spec-integrity`/`check-scenario-drift`/`check-schema-version-drift`、ソースコードのdocstring確認は`scan-source-code`/`lint-docstring`を使う。 |
| 既存document.jsonはscaffold fillで編集する | `fill`は`documentPath`が指す既存document（作成直後に限らない）に対し、schemaが`x-prompt-write`を宣言した任意のpathへ値を書き込めるため、Edit/Writeで直接書き換える必要がない。 | 配列フィールドは丸ごと置き換えなので、`query`で現在値を取得→配列を組み立てる→`scaffold fill`で書き込む、という手順で値オブジェクトの追加・削除もCLI経由で行う。 |
| schemaに新規必須トップレベルキーが追加された場合のみEdit/Writeを許容する | `fill`は対象キーが既にdocument内に存在する前提でしか書けず、`create`も既存documentがあれば中身を保護して上書きしない（新規必須フィールドの後追いマージはしない）ため、この場合だけはCLI/MCP経由で対応できない。 | このgap自体は`check-schema-version-drift`のmissing_declared_fieldsチェックで機械検知される（「編集できない」ことと「気づけない」ことは別問題として扱う）。検知されたキーのみEdit/Writeで追記する。 |
| document.jsonのパス規約は`.waffle/`を使う（`.has-udd/`ではない） | Waffle自身を説明するspec/skill/knowledge documentは、has-udd汎用のパスと混在させず一元管理するため。 | Waffle自身のdocumentは`waffle/.waffle/documents/`配下に置く。repo root側`.has-udd/documents/`にはWaffle固有でない汎用skillのみ残す。 |
| repo rootからは`uv run --project waffle waffle <command>`の形で呼び出す | waffleパッケージはrepo rootとは別プロジェクト（`waffle/`配下）に閉じているため。 | `.claude/skills/`へdeployする際は`waffle/`配下へのパスを明示する（例: `waffle render --path waffle/.waffle/documents/skills/harness-query-engine.json`）。 |
| Skill/advisor間のインターフェースはテキストベース疎結合に統一する | 呼び出す側が相手の内部形式（他Skillの出力document型・schema等）を事前に知って合わせる必要をなくすため。DDDの「モデル変換装置（腐敗防止層）」と同型の原則。 | Skill・engine・advisor間の入出力はテキスト（意図）に統一し、構造化データへの成型は推論と文脈を持つ受け手側が行う。詳細経緯: `docs/brainstorm/brainstorm-platform-engineering-application.md`。 |

---

## 主要コマンド

| コマンド | 用途 |
|---|---|
| uv run pytest | unit/integration/acceptance/contractの4層テストを実行する |
| uv run waffle validate --path <document.jsonのパス> | document.jsonをschemaに適合検証する |
| uv run waffle render --path <document.jsonのパス> | document.jsonを成果物（SKILL.md等）へ描画し、deploy先（.claude/skills/等）へ配置する |
| uv run waffle query --operation <操作名> --path <document.jsonのパス> | document.jsonの内容を確認・検索する（get_block/get_field等16操作） |
| uv run waffle scaffold --operation create\|fill ... | document.jsonの骨格生成、および既存documentへの値の書き込みを行う |
| uv run waffle check-spec-integrity / check-scenario-drift / check-schema-version-drift | spec参照整合性・シナリオドリフト・schema版ドリフトを確認する |

---

## 委譲先

### 以下のscope配下で作業する場合は、必ず対応するOrchestrator documentのcontentを先に読むこと

| 対象ディレクトリ | 参照Orchestrator | 備考 |
|---|---|---|

委譲先なし（このOrchestratorの管轄範囲に、より狭いscopeを持つ子Orchestratorは存在しない）
