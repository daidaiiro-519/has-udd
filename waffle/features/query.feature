# uc-query-document の受け入れシナリオ（What の SSOT・実行可能）。
# 実装はこのシナリオが緑である限り自由に変えてよい。振る舞いが変わる時だけ仕様を先に更新する（UDD 規律）。
# 対象は has-udd 自身の query engine document（dogfood）。
Feature: document.json へのセマンティック・クエリ (uc-query-document)

  Background:
    Given query engine
    And 対象は ".has-udd/documents/skills/harness-query-engine.json"

  # --- Group 1: ファイル / ドキュメント単位（prompt は null） ---

  Scenario: scan は生テキストを返す
    When operation "scan" を実行する
    Then 成功する
    And prompt は null
    And value は "documentId" を含む

  Scenario: get_meta はメタ情報を返す
    When operation "get_meta" を実行する
    Then 成功する
    And value の "documentId" は "harness-query-engine"

  Scenario: index_scan は blockType と prompt を schema から動的算出する
    When operation "index_scan" を実行する
    Then 成功する
    And value の "interface.blockType" は "Interface"
    And value の "interface.prompt" は非空

  Scenario: index_scan_dir はディレクトリ横断で index を集約する
    Given 対象は ".has-udd/documents/skills"
    When operation "index_scan_dir" を実行する
    Then 成功する
    And value のキーに "harness-query-engine.json" を含むものがある

  # --- Group 2: ブロック（prompt = 対象 block の x-prompt-query） ---

  Scenario: get_block は block 全体と prompt を返す
    When operation "get_block" を params "blockKey=interface" で実行する
    Then 成功する
    And value の "blockType" は "Interface"
    And prompt は非空

  Scenario: get_field は block の1フィールドを返す
    When operation "get_field" を params "blockKey=purpose;field=text" で実行する
    Then 成功する
    And value は "document.json" を含む

  # --- Group 3: 配列操作 ---

  Scenario: filter_items は key==value で絞り込む（文字列 "true" は bool True に一致）
    When operation "filter_items" を params "blockKey=interface;arrayField=input;key=required;value=true" で実行する
    Then 成功する
    And value の "name" 集合は "operation,path"

  Scenario: filter_items は一致0件でも正常系で空配列を返す（NO_MATCH マーカーは持たない）
    When operation "filter_items" を params "blockKey=interface;arrayField=input;key=required;value=__none__" で実行する
    Then 成功する
    And value は空配列

  Scenario: get_by_id は単一オブジェクトを返す（リストではない・id は一意前提）
    When operation "get_by_id" を params "blockKey=steps;arrayField=items;idField=stepId;idValue=step-2" で実行する
    Then 成功する
    And value の "title" は "オペレーションを選ぶ"

  # --- Group 4: 再帰 ---

  Scenario: find_all は全階層を再帰収集する
    When operation "find_all" を params "fieldName=stepId" で実行する
    Then 成功する
    And value は "step-2" を含む

  # --- raw フォールバック ---

  Scenario: schemaRef を持たないファイルは raw で返す
    Given schemaRef なしの一時ファイルを対象にする
    When operation "get_meta" を実行する
    Then 成功する
    And 結果の "type" は "raw"

  # --- エラーコード ---

  Scenario: 未知 operation は INVALID_OPERATION
    When operation "bogus" を実行する
    Then エラーコード "INVALID_OPERATION" で失敗する

  Scenario: 必須パラメータ欠落は MISSING_PARAM
    When operation "get_block" を実行する
    Then エラーコード "MISSING_PARAM" で失敗する

  Scenario: 存在しない blockKey は NOT_FOUND
    When operation "get_block" を params "blockKey=nope" で実行する
    Then エラーコード "NOT_FOUND" で失敗する

  Scenario: 不正な正規表現は INVALID_PATTERN
    When operation "filter_pattern" を params "blockKey=interface;arrayField=input;field=name;pattern=[" で実行する
    Then エラーコード "INVALID_PATTERN" で失敗する

  Scenario: 存在しないパスは INVALID_PATH
    Given 対象は "does/not/exist.json"
    When operation "get_meta" を実行する
    Then エラーコード "INVALID_PATH" で失敗する

  # --- セキュリティ（頑健化 G6 / G7） ---

  Scenario: パストラバーサルは拒否する (G6)
    Given 対象は "../etc/passwd"
    When operation "scan" を実行する
    Then エラーコード "INVALID_PATH" で失敗する

  Scenario: index_scan_dir はプロジェクトルート外を拒否する (G7)
    Given 対象は "/etc"
    When operation "index_scan_dir" を実行する
    Then エラーコード "INVALID_PATH" で失敗する
