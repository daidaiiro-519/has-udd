Feature: uc-validate-document

  Scenario: 適合する Document は VALIDATED 判定になる
    Given schema に適合する Document
    When validate する
    Then VALIDATED 判定が返る

  Scenario: 不適合は違反詳細つきで失敗する
    Given schema に適合しない Document
    When validate する
    Then 違反詳細つきで失敗する
