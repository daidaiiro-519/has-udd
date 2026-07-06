Feature: uc-check-scenario-drift

  Scenario: 存在しないspec.jsonはINVALID_PATH
    When 存在しないspec.jsonのパスでドリフト検査を実行する
    Then INVALID_PATHエラーが返る

  Scenario: 存在しないテストファイルはINVALID_PATH
    When 存在しないテストファイルのパスでドリフト検査を実行する
    Then INVALID_PATHエラーが返る

  Scenario: 全シナリオに対応するテストがあり孤立も無いとき整合していると判定する
    Given 宣言する全シナリオに対応するtest_*関数を持つテストファイル
    When ドリフト検査を実行する
    Then missing_in_tests・orphaned_in_tests両方が空配列で返る

  Scenario: 宣言されたシナリオに対応するテストが無いことを検出する
    Given シナリオを宣言するが対応するtest_*関数を持たないテストファイル
    When ドリフト検査を実行する
    Then missing_in_testsにそのシナリオ名（sanitize後）が含まれる

  Scenario: 宣言に対応しないテスト関数を検出する
    Given どのシナリオ宣言にも対応しないtest_*関数を含むテストファイル
    When ドリフト検査を実行する
    Then orphaned_in_testsにその関数名が含まれる

  Scenario: 構文解析できないテストファイルはINVALID_SOURCE
    Given 構文が壊れたテストファイル
    When ドリフト検査を実行する
    Then INVALID_SOURCEエラーが返る
