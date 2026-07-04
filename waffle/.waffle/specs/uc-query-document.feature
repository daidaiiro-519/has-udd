Feature: uc-query-document

  Scenario: ブロックを丸ごと取得する
    Given query engine と対象 Document
    When operation get_block を blockKey interface で実行する
    Then value は対象ブロックであり、prompt に読み方の指針が付く

  Scenario: 条件に一致する配列要素だけを絞り込む
    Given query engine と対象 Document
    When operation filter_items で required=true を指定する
    Then value には required な要素だけが含まれる

  Scenario: 一致が無くても正常系で空配列を返す
    When 一致しないフィルタ条件で filter_items を実行する
    Then value は空配列で、エラーにはならない

  Scenario: 未知の operation はエラーを返す
    When 未知の operation を実行する
    Then INVALID_OPERATION エラーが返る
