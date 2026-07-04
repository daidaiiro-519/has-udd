# uc-validate-document の受け入れシナリオ（What の SSOT・実行可能）。
# 検証は JSON Schema 適合のみ（x-render lint は別責務）。副作用なし（status は書き換えない）。
Feature: document を schema 適合検証する (uc-validate-document)

  Background:
    Given validate engine

  # --- 適合（dogfood: waffle 自身の全 document） ---

  Scenario Outline: 既存 document は schema に適合する
    Given 対象は "<path>"
    When 検証する
    Then 成功する
    And status は "VALIDATED"

    Examples:
      | path                                                |
      | .waffle/documents/skills/harness-query-engine.json |
      | .waffle/documents/skills/harness-render-engine.json |
      | .waffle/documents/coding/tech-stack-python-hexagonal.json      |
      | .waffle/documents/coding/architecture-python-hexagonal.json    |
      | .waffle/documents/coding/coding-standard-python-hexagonal.json |
      | .waffle/documents/coding/test-standard-python-hexagonal.json   |
      | .waffle/documents/specs/bc-waffle-engines.json      |
      | .waffle/documents/specs/sd-harness-core.json        |
      | .waffle/documents/specs/sd-validation.json          |
      | .waffle/documents/specs/sd-rendering.json           |
      | .waffle/documents/specs/agg-document.json           |
      | .waffle/documents/specs/agg-schema.json             |
      | .waffle/documents/specs/uc-query-document.json      |
      | .waffle/documents/specs/uc-render-document.json     |
      | .waffle/documents/specs/uc-validate-document.json   |
      | .waffle/documents/specs/uc-scaffold-document.json   |
      | .waffle/documents/specs/uc-scan-source-code.json    |
      | .waffle/documents/specs/uc-lint-docstring.json    |

  # --- 不適合 ---

  Scenario: schema に適合しない document は違反詳細つきで失敗する
    Given 不適合な document の一時ファイルを対象にする
    When 検証する
    Then 違反詳細つきで失敗する

  # --- エラー / セキュリティ（頑健化） ---

  Scenario: schemaRef を持たない document は MISSING_SCHEMA_REF
    Given schemaRef なしの一時ファイルを対象にする
    When 検証する
    Then エラーコード "MISSING_SCHEMA_REF" で失敗する

  Scenario: 存在しないパスは INVALID_PATH
    Given 対象は "does/not/exist.json"
    When 検証する
    Then エラーコード "INVALID_PATH" で失敗する

  Scenario: パストラバーサルは拒否する (G6)
    Given 対象は "../etc/passwd.json"
    When 検証する
    Then エラーコード "INVALID_PATH" で失敗する

  Scenario: 不正な JSON は INVALID_JSON
    Given 不正な JSON の一時ファイルを対象にする
    When 検証する
    Then エラーコード "INVALID_JSON" で失敗する
