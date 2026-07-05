Feature: uc-render-document

  Scenario: 検証済み Document を成果物に描画する
    Given 描画対象の Document
    When render する
    Then 成果物が生成され、生成パス一覧が返る

  Scenario: schemaRef を持たない Document は描画しない
    Given schemaRef の無い Document
    When render する
    Then MISSING_SCHEMA_REF エラーが返る

  Scenario: 存在しないパスは描画しない
    When 存在しないパスを対象に render する
    Then INVALID_PATH エラーが返る

  Scenario: deploy すると canonical と deploy 先の両方に書く
    Given deploy 先を持つ Document
    When deploy を有効にして render する
    Then canonical と deploy 先の両方に成果物が書かれる

  Scenario: 同じDocumentを2回renderしても同一の成果物になる
    Given 変更されていないDocument
    When 同じDocumentを2回renderする
    Then 1回目と2回目の成果物は同一である
