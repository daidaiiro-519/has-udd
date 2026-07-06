Feature: uc-validate-document

  Scenario: 適合する Document は VALIDATED 判定になる
    Given schema に適合する Document
    When validate する
    Then VALIDATED 判定が返る

  Scenario: 不適合は違反詳細つきで失敗する
    Given schema に適合しない Document
    When validate する
    Then 違反詳細つきで失敗する

  Scenario: schemaRef を持たない Document は検証できない
    Given schemaRef の無い Document
    When validate する
    Then MISSING_SCHEMA_REF エラーが返る

  Scenario Outline: 既存documentはschemaに適合する
    Given waffle自身のdocument
    When validateする
    Then 成功し、schemaのlifecycleに応じた正しいstatusになる

  Scenario: SUPERSEDEDは終端でありvalidateを受け付けない
    Given SUPERSEDED状態のDocument
    When validateする
    Then INVALID_TRANSITIONエラーが返る

  Scenario: 存在しないパスはINVALID_PATH
    Given 存在しない対象パス
    When validateする
    Then INVALID_PATHエラーが返る

  Scenario: 不正なJSONはINVALID_JSON
    Given 不正なJSONの対象ファイル
    When validateする
    Then INVALID_JSONエラーが返る
