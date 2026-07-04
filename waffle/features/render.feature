# uc-render-document の受け入れシナリオ（What の SSOT・実行可能）。
# render は schema 適合検証をしない（検証は uc-validate-document の責務）。frontmatter のみ x-frontmatter 駆動。
Feature: document.json を成果物にレンダリング (uc-render-document)

  Background:
    Given render engine

  # --- 正常系（SkillSchema → Markdown） ---

  Scenario: SkillSchema を Markdown にレンダリングする
    Given 対象は ".waffle/documents/skills/harness-query-engine.json"
    When deploy なしでレンダリングする
    Then 成功する
    And 出力フォーマットは "md"
    And 出力に "# harness-query-engine" を含む
    And 出力に "## 目的" を含む
    And 出力に "| name | type | 必須 | 説明 | 例 |" を含む
    And 出力に "### Step 2: オペレーションを選ぶ" を含む
    And 出力に "| operation | 用途 | 必須引数 | 例 |" を含む
    And 出力に "waffle query --operation get_block" を含む

  Scenario: frontmatter は x-frontmatter のドットパスを解決して生成する
    Given 対象は ".waffle/documents/skills/harness-query-engine.json"
    When deploy なしでレンダリングする
    Then 成功する
    And 出力に "name:" を含む
    And 出力に "description:" を含む
    And 出力に "harness-query-engine" を含む

  Scenario: deploy すると canonical と deploy 先の両方に書く
    Given 対象は ".waffle/documents/skills/harness-query-engine.json"
    When レンダリングして deploy する
    Then 成功する
    And 出力パスは ".waffle/skills/harness-query-engine/SKILL.md"
    And deploy 先に ".claude/skills/harness-query-engine/SKILL.md" を含む

  # --- CodingSchema → Markdown ---

  Scenario: CodingSchema は Markdown として描画できる
    Given 対象は ".waffle/documents/coding/tech-stack-python-hexagonal.json"
    When deploy なしでレンダリングする
    Then 成功する
    And 出力フォーマットは "md"
    And 出力に "# " を含む

  # --- DomainSpecSchema（UDD ループ: TestScenarios → .feature） ---

  Scenario: usecase Spec は基本フローをシーケンス図に・TestScenarios を Markdown に出す
    Given 対象は ".waffle/documents/specs/bc-waffle-engines/subdomain/sd-harness-core/usecase/uc-query-document.json"
    When deploy なしでレンダリングする
    Then 成功する
    And 出力フォーマットは "md"
    And 出力に "# uc-query-document" を含む
    And 出力に "sequenceDiagram" を含む
    And 出力に "mermaid" を含む
    And 出力に "## テストシナリオ" を含む
    And 出力に "Scenario: 未知の operation はエラーを返す" を含む
    And feature出力に "Feature: uc-query-document" を含む
    And feature出力に "Scenario: 未知の operation はエラーを返す" を含む

  Scenario: aggregate Spec は集約の構造とライフサイクルを Markdown に出す
    Given 対象は ".waffle/documents/specs/bc-waffle-engines/aggregate/agg-document.json"
    When deploy なしでレンダリングする
    Then 成功する
    And 出力に "## コマンド" を含む
    And 出力に "DocumentRendered" を含む
    And 出力に "stateDiagram-v2" を含む

  # --- エラー / セキュリティ（頑健化） ---

  Scenario: schemaRef を持たない document は MISSING_SCHEMA_REF
    Given schemaRef なしの一時ファイルを対象にする
    When deploy なしでレンダリングする
    Then エラーコード "MISSING_SCHEMA_REF" で失敗する

  Scenario: 存在しないパスは INVALID_PATH
    Given 対象は "does/not/exist.json"
    When deploy なしでレンダリングする
    Then エラーコード "INVALID_PATH" で失敗する

  Scenario: パストラバーサルは拒否する (G6)
    Given 対象は "../etc/passwd.json"
    When deploy なしでレンダリングする
    Then エラーコード "INVALID_PATH" で失敗する

  Scenario: 不正な JSON は INVALID_JSON
    Given 不正な JSON の一時ファイルを対象にする
    When deploy なしでレンダリングする
    Then エラーコード "INVALID_JSON" で失敗する
