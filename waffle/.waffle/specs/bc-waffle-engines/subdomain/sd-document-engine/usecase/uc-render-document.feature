Feature: uc-render-document

  Scenario: 同じDocumentを2回renderしても同一の成果物になる
    Given 変更されていないDocument
    When 同じDocumentを2回renderする
    Then 1回目と2回目の成果物は同一である

  Scenario: render_engineはschemaのx_render宣言をpart_rendererへ正しく配線する
    Given interfaceブロック(x-render宣言=table)を持つDocument
    When render engine経由でrenderする
    Then schemaのx-render宣言どおりに整形されたMarkdownテーブルが出力に含まれる

  Scenario: 存在しないパスはINVALID_PATH
    Given 実在しない対象パス
    When 本usecaseを実行する
    Then INVALID_PATHエラーが返る

  Scenario: 解決できないschemaRefはINVALID_SCHEMA_REF
    Given 解決できないschemaRef
    When 本usecaseを実行する
    Then INVALID_SCHEMA_REFエラーが返る

  Scenario: 検証済み Document を成果物に描画する
    Given 描画対象の Document
    When render する
    Then 成果物が生成され、生成パス一覧が返る

  Scenario: schemaRef を持たない Document は描画しない
    Given schemaRef の無い Document
    When render する
    Then MISSING_SCHEMA_REF エラーが返る

  Scenario: deploy すると canonical と deploy 先の両方に書く
    Given deploy 先を持つ Document
    When deploy を有効にして render する
    Then canonical と deploy 先の両方に成果物が書かれる

  Scenario: SkillSchemaをMarkdownにレンダリングする
    Given SkillSchemaのDocument
    When renderする
    Then 見出し・目的・パラメータ表・オペレーション選択・呼び出し例が全て出力に含まれる

  Scenario: frontmatterはx_frontmatterのドットパスを解決して生成する
    Given x-frontmatterを宣言するSchemaのDocument
    When renderする
    Then 出力冒頭にname/description等を含むYAML frontmatterが生成される

  Scenario: CodingSchemaはMarkdownとして描画できる
    Given CodingSchemaのDocument
    When renderする
    Then Markdown形式で見出しを含む出力が生成される

  Scenario: usecase_Specは基本フローをシーケンス図に受け入れシナリオをMarkdownに出す
    Given usecase SpecのDocument
    When renderする
    Then 出力にmermaidのsequenceDiagramとテストシナリオ節が含まれ、feature出力にも同じシナリオが含まれる

  Scenario: aggregate_Specは集約の構造とライフサイクルをMarkdownに出す
    Given aggregate SpecのDocument
    When renderする
    Then 出力にコマンド節・ドメインイベント名・mermaidのstateDiagram-v2が含まれる

  Scenario: 不正なJSONはINVALID_JSON
    Given 不正なJSONの対象ファイル
    When renderする
    Then INVALID_JSONエラーが返る
