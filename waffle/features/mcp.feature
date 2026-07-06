# waffle MCP サーバ（inbound adapter）の受け入れシナリオ。
# CLI と並ぶ第2の front-door。engine の振る舞いは各 engine の .feature が担保。
# ここは「MCP ツール経由で engine が正しく呼ばれ dict を返す」スモークを固定。
Feature: waffle MCP サーバ (inbound adapter)

  Background:
    Given waffle MCP サーバ

  Scenario: query_document はブロックを取得する
    When MCP ツール "query_document" を引数 "operation=get_block;path=.waffle/documents/skills/harness-query-engine.json;blockKey=interface" で呼ぶ
    Then MCP出力の "value.blockType" は "Interface"

  Scenario: query_document のエラーは {error, message} を返す
    When MCP ツール "query_document" を引数 "operation=bogus;path=.waffle/documents/skills/harness-query-engine.json" で呼ぶ
    Then MCP出力の "error" は "INVALID_OPERATION"

  Scenario: validate_document は適合で status 判定を返す
    When MCP ツール "validate_document" を引数 "path=.waffle/documents/skills/harness-query-engine.json" で呼ぶ
    Then MCP出力の "status" は "DRAFT"

  Scenario: render_document は md フォーマットを返す
    When MCP ツール "render_document" を引数 "path=.waffle/documents/skills/harness-query-engine.json;deploy=false" で呼ぶ
    Then MCP出力の "format" は "md"

  Scenario: migrate_schema のエラーは {error, message} を返す
    When MCP ツール "migrate_schema" を引数 "operation=publishVersion" で呼ぶ
    Then MCP出力の "error" は "MISSING_PARAM"
