# uc-query-document の受け入れシナリオ（What の SSOT・実行可能）。
# 実装はこのシナリオが緑である限り自由に変えてよい。振る舞いが変わる時だけ仕様を先に更新する（UDD 規律）。
# 対象は waffle 自身の query engine document（dogfood）。
Feature: document.json へのセマンティック・クエリ (uc-query-document)

  Background:
    Given query engine
    And 対象は ".waffle/documents/skills/harness-query-engine.json"

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
    Given 対象は ".waffle/documents/skills"
    When operation "index_scan_dir" を実行する
    Then 成功する
    And value のキーに "harness-query-engine.json" を含むものがある

  # --- Group 2: ブロック（prompt = 対象 block の x-prompt-query） ---
  # get_block の基本成功シナリオは uc-query-document の TestScenarios（tests/acceptance/test_uc_query_document.py）に移行済み。

  Scenario: get_field は block の1フィールドを返す
    When operation "get_field" を params "blockKey=purpose;field=text" で実行する
    Then 成功する
    And value は "document.json" を含む

  # --- Group 3: 配列操作 ---
  # filter_items の基本成功・空一致シナリオは uc-query-document の TestScenarios に移行済み。

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
  # 未知operation(INVALID_OPERATION)・必須パラメータ欠落(MISSING_PARAM)・存在しないblockKey(NOT_FOUND)・
  # 不正な正規表現(INVALID_PATTERN)・存在しないパス(INVALID_PATH)は uc-query-document の TestScenarios に移行済み。

  # --- セキュリティ（頑健化 G6 / G7） ---
  # パストラバーサル拒否(G6)・index_scan_dirのルート外拒否(G7)は、Document集約横断の不変条件として
  # agg-document の UnitTestScenarios（tests/test_document_path_confinement.py）に移行済み。
