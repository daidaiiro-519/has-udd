Feature: agg-document

  Scenario: status は逆行できない
    Given RENDERED 状態の Document
    When validate へ戻そうとする
    Then 状態遷移は拒否され、状態は RENDERED のままである

  Scenario: 未検証では render できない
    Given CREATED 状態の Document
    When render する
    Then 拒否され、状態は CREATED のままである

  Scenario: SUPERSEDED は終端
    Given SUPERSEDED 状態の Document
    When 任意のコマンドを実行する
    Then 拒否される

  Scenario: Coding/Skill の status も逆行できない
    Given ACTIVE 状態の Coding document
    When DRAFT へ戻そうとする
    Then 状態遷移は拒否され、状態は ACTIVE のままである

  Scenario: DEPRECATED は終端
    Given DEPRECATED 状態の Skill document
    When 任意のコマンドを実行する
    Then 拒否される

  Scenario: パストラバーサルを含むパスは拒否される
    Given '..' を含む対象パス
    When 任意の operation・command を実行する
    Then INVALID_PATH エラーが返り、プロジェクトルート外へはアクセスしない

  Scenario: ディレクトリ横断はプロジェクトルート外を拒否する
    Given プロジェクトルート外を指すディレクトリパス
    When index_scan_dir を実行する
    Then INVALID_PATH エラーが返る
