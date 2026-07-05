# uc-validate-document の受け入れシナリオ（What の SSOT・実行可能）。
# 検証は JSON Schema 適合のみ（x-render lint は別責務）。副作用なし（status は書き換えない）。
Feature: document を schema 適合検証する (uc-validate-document)

  Background:
    Given validate engine

  # --- 適合（dogfood: waffle 自身の全 document） ---

  # status の期待値は schema の x-lifecycle が "validate" を状態遷移コマンドとして
  # 定義しているかで変わる: Spec家族系（DomainSpecSchema/PresentationSpecSchema）=VALIDATEDへ進む／CodingSchema・SkillSchema系=
  # maturityLifecycle に validate が無いため現状の status を維持する。
  Scenario Outline: 既存 document は schema に適合する
    Given 対象は "<path>"
    When 検証する
    Then 成功する
    And status は "<status>"

    Examples:
      | path                                                            | status    |
      | .waffle/documents/skills/harness-query-engine.json             | DRAFT     |
      | .waffle/documents/skills/harness-render-engine.json            | DRAFT     |
      | .waffle/documents/coding/tech-stack-python-hexagonal.json      | ACTIVE    |
      | .waffle/documents/coding/architecture-python-hexagonal.json    | ACTIVE    |
      | .waffle/documents/coding/coding-standard-python-hexagonal.json | ACTIVE    |
      | .waffle/documents/coding/test-standard-python-hexagonal.json   | ACTIVE    |
      | .waffle/documents/specs/bc-waffle-engines/bc-waffle-engines.json                                       | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/subdomain/sd-harness-core/sd-harness-core.json               | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/subdomain/sd-validation/sd-validation.json                   | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/subdomain/sd-rendering/sd-rendering.json                     | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/aggregate/agg-document.json                                  | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/aggregate/agg-schema.json                                    | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/subdomain/sd-harness-core/usecase/uc-query-document.json     | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/subdomain/sd-rendering/usecase/uc-render-document.json       | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/subdomain/sd-validation/usecase/uc-validate-document.json    | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/subdomain/sd-harness-core/usecase/uc-scaffold-document.json  | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/subdomain/sd-harness-core/usecase/uc-scan-source-code.json   | VALIDATED |
      | .waffle/documents/specs/bc-waffle-engines/subdomain/sd-validation/usecase/uc-lint-docstring.json       | VALIDATED |

  Scenario: SUPERSEDED は終端であり validate を受け付けない (Re-2 guard)
    Given SUPERSEDED 状態の一時ファイルを対象にする
    When 検証する
    Then エラーコード "INVALID_TRANSITION" で失敗する

  # --- 不適合 ---
  # 違反詳細つき失敗・MISSING_SCHEMA_REF は uc-validate-document の TestScenarios（tests/acceptance/test_uc_validate_document.py）に移行済み。

  # --- エラー / セキュリティ（頑健化） ---
  # パストラバーサル拒否(G6)は agg-document の UnitTestScenarios（tests/test_document_path_confinement.py）に移行済み。

  Scenario: 存在しないパスは INVALID_PATH
    Given 対象は "does/not/exist.json"
    When 検証する
    Then エラーコード "INVALID_PATH" で失敗する

  Scenario: 不正な JSON は INVALID_JSON
    Given 不正な JSON の一時ファイルを対象にする
    When 検証する
    Then エラーコード "INVALID_JSON" で失敗する
