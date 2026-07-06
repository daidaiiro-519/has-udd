Feature: agg-schema

  Scenario: 値フィールドに oneOf を持てない
    Given 値フィールドに oneOf を含む Schema
    When scaffoldability を検証する
    Then scaffold 不能として拒否される

  Scenario: 公開済みの版は後方互換を壊さない
    Given PUBLISHED の Schema 版
    When 既存ブロックに必須フィールドを追加しようとする
    Then 後方互換を壊す変更として拒否される

  Scenario: 移行は版を上げる方向にのみ行う
    Given v1 と v2 の Schema
    When v2 から v1 へ移行しようとする
    Then 拒否される

  Scenario: x-render は閉じた語彙にのみ従う
    Given 未知の部品種別、または必須属性が欠けた x-render 宣言を持つ Schema
    When x-render の適合を検証する
    Then 不適合として拒否される

  Scenario: rename宣言は_from_を要求する
    Given as=renameでfromを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される

  Scenario: default宣言は_value_を要求する
    Given as=defaultでvalueを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される

  Scenario: value_map宣言は_from_と_mapping_を要求する
    Given as=value-mapでmappingを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される

  Scenario: discriminator_remap宣言は_rules_を要求する
    Given as=discriminator-remapでrulesを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される

  Scenario: ai_infer宣言は_prompt_を要求する
    Given as=ai-inferでpromptを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される

  Scenario: 未知の種別は拒否される
    Given asが未知の種別であるx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される

  Scenario: schema自体がロードできる
    Given MigrationMetaSchema/v1
    When PackageSchemaRepositoryでロードする
    Then $defs.MigrationDeclarationが取得できる

  Scenario: 全schemaがx_schema_statusを宣言している
    Given Documentのschemaが指しうる型(DomainSpecSchema/PresentationSpecSchema/CodingSchema/SkillSchema)
    When 各schemaファイルのx-schema-statusを確認する
    Then 全てPUBLISHED/DEPRECATEDのいずれかを宣言している
