# waffle

---

## 管轄範囲

- **管轄ディレクトリ**: waffle/
- **概要**: スキーマという型で文書を焼き上げる、構造検証＋意味ガイダンス内蔵のドキュメントエンジン「Waffle」。JSON Schemaでdocument.jsonを検証・query・render・scaffoldする。自己完結ディレクトリ（`git subtree split --prefix=waffle`で独立可能）。

---

## 運用ルール

| ルール | なぜ | 適用方法 |
|---|---|---|
| document.json操作は必ずCLI/MCP経由で行う | 自分自身がdogfood対象のため | create/fill/validate/render/query/check-*/scan-source-code/lint-docstringを使う（直接読み書きしない） |
| 既存document.jsonはscaffold fillで編集する | x-prompt-write宣言済みpathへ書き込めるため | 配列は query で現在値取得→組み立て→fill で丸ごと置き換え |
| schemaに新規必須トップレベルキー追加時のみEdit/Writeを許容する | fill/createは新規キーの後追いマージをしないため | check-schema-version-drift で検知されたキーのみEdit/Writeで追記 |
| document.jsonのパスは`.waffle/`を使う | has-udd汎用パス（`.has-udd/`）と混在させないため | `waffle/.waffle/documents/`配下に置く |
| repo rootからは`uv run --project waffle waffle <command>`で呼ぶ | waffleはrepo rootとは別プロジェクトのため | deploy先パスは`waffle/`配下を明示する |
| Skill/advisor間はテキストベース疎結合を保つ | 受け手の内部形式を事前に知らなくてよくするため（腐敗防止層と同型） | 入出力はテキストに統一し、構造化への成型は受け手側が行う |

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

---

## Skillフォローアップ

### Skillを呼び出した後に必ず行うフォローアップ

| 呼び出した後 | 次に行うこと | 返却フォーマット | テンプレート | 理由 |
|---|---|---|---|---|
| advisor（ddd-advisor/tech-lead-advisor/ux-advisor等） | 同じadvisorへ批評フェーズとして再検証させるか、対象の性質に応じて関連する他advisor（例: レイヤー配置ならtech-lead-advisor、ドメイン整合性ならddd-advisor）へ批評を依頼してから確定する | 各意見 → 統合見解 → 合意事項 → 次のアクション | `waffle/.waffle/agent/references/template-skill-critique.md` | advisorはWaffleが所有・出荷する成果物であり、単一advisorの一発出力を無検証で確定させない運用はWaffle自身のOrchestratorの責務。advisorが互いを呼ぶのではなくOrchestrator側が組合せを判断することで、Skill/advisor間のテキストベース疎結合原則を保てる |
