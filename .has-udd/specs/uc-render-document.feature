Feature: Document を成果物にレンダリング (uc-render-document)

  Scenario: 検証済み Document を成果物に描画する
    Given 描画対象の Document
    When render する
    Then 成果物が生成され、生成パス一覧が返る

  Scenario: schemaRef を持たない Document は描画しない
    Given schemaRef の無い Document
    When render する
    Then MISSING_SCHEMA_REF エラーが返る
