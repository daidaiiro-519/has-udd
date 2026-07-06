# waffle CLI（inbound adapter）の受け入れシナリオ。
# engine の振る舞いは各 engine の .feature が担保。ここは「引数成型・出力JSON整形・終了コード」を固定。
Feature: waffle CLI (inbound adapter)

  Background:
    Given waffle CLI

  Scenario: query はブロックを取得し value を JSON で返す
    When CLI "query --operation get_block --path .waffle/documents/skills/harness-query-engine.json --blockKey interface" を実行する
    Then 終了コードは 0
    And 出力JSONの "value.blockType" は "Interface"

  Scenario: query のエラーは {error, message} と非ゼロ終了で返す
    When CLI "query --operation bogus --path .waffle/documents/skills/harness-query-engine.json" を実行する
    Then 終了コードは 1
    And 出力JSONの "error" は "INVALID_OPERATION"

  Scenario: render --no-deploy は md フォーマットを返す
    When CLI "render --path .waffle/documents/skills/harness-query-engine.json --no-deploy" を実行する
    Then 終了コードは 0
    And 出力JSONの "format" は "md"

  Scenario: validate は適合で status 判定を返す
    When CLI "validate --path .waffle/documents/skills/harness-query-engine.json" を実行する
    Then 終了コードは 0
    And 出力JSONの "status" は "DRAFT"

  Scenario: scaffold create は骨格を返す
    When CLI "scaffold --operation create --schemaRef SkillSchema/v1 --documentId scaffold-demo --discriminator skillKind=engine" を実行する
    Then 終了コードは 0
    And 出力JSONの "skeleton.documentType" は "Skill"

  Scenario: migrate のエラーは {error, message} と非ゼロ終了で返す
    When CLI "migrate --operation publishVersion" を実行する
    Then 終了コードは 1
    And 出力JSONの "error" は "MISSING_PARAM"
