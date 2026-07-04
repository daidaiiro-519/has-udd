# uc-scaffold-document の受け入れシナリオ（What の SSOT・実行可能）。
# Harness 原則: AI は「値」だけ生成し、engine が document.json の構造を組む。
# create=骨格+fillTemplate を生成し x-source-target に書く / fill=値だけ機械的に書き込む。
# 値の適合検証は uc-validate-document の責務（fill は構造保護のみ）。
Feature: document.json のスキャフォールド (uc-scaffold-document)

  Background:
    Given scaffold engine

  # --- create: 骨格生成（status=schema enum 先頭・自分の schema で valid） ---

  Scenario: create は engine skill の骨格を生成する
    When create を schemaRef "SkillSchema/v1" documentId "scaffold-demo" discriminator "skillKind=engine" で実行する
    Then 成功する
    And skeleton の "documentType" は "Skill"
    And skeleton の "schemaRef" は "SkillSchema/v1"
    And skeleton の "skillKind" は "engine"
    And skeleton の "status" は "DRAFT"
    And skeleton の content に "interface" がある
    And skeleton の content に "invocationSpec" がある

  Scenario: 生成した骨格は自分の schema で valid（status=DRAFT・空値）
    When create を schemaRef "SkillSchema/v1" documentId "scaffold-demo" discriminator "skillKind=engine" で実行する
    Then 成功する
    And 生成された骨格は validate を通る

  Scenario: create は x-source-target に骨格を書き出す
    When create を schemaRef "SkillSchema/v1" documentId "scaffold-demo" discriminator "skillKind=engine" で実行する
    Then 成功する
    And ファイル ".waffle/documents/skills/scaffold-demo.json" が存在する

  Scenario: fillTemplate は値フィールドの path と prompt(x-prompt-write) を持つ
    When create を schemaRef "SkillSchema/v1" documentId "scaffold-demo" discriminator "skillKind=engine" で実行する
    Then 成功する
    And fillTemplate に path "content.purpose.text" のエントリがあり prompt が付く

  Scenario: custom は engine と構成が異なる（processingTarget を持つ）
    When create を schemaRef "SkillSchema/v1" documentId "scaffold-demo-custom" discriminator "skillKind=custom" で実行する
    Then 成功する
    And skeleton の content に "processingTarget" がある

  Scenario: discriminator を省くと MISSING_DISCRIMINATOR（候補つき）
    When create を schemaRef "SkillSchema/v1" documentId "scaffold-demo" で実行する
    Then エラーコード "MISSING_DISCRIMINATOR" で失敗する

  Scenario: 未知の schemaRef は INVALID_SCHEMA_REF
    When create を schemaRef "NoSuchSchema/v1" documentId "scaffold-demo" discriminator "skillKind=engine" で実行する
    Then エラーコード "INVALID_SCHEMA_REF" で失敗する

  # --- fill: 値だけ書き込む（構造は engine が保護） ---

  Scenario: fill は宣言済み値フィールドに値を書き込む
    Given "scaffold-demo" の engine 骨格を作成済み
    When fill で値 "content.purpose.text=ドメインを分析する" を書き込む
    Then 成功する
    And written に "content.purpose.text" を含む
    And ファイルの "content.purpose.text" は "ドメインを分析する"

  Scenario: fill は const / 未知 path を拒否して skipped に記録する
    Given "scaffold-demo" の engine 骨格を作成済み
    When fill で値 "content.purpose.blockType=X" を書き込む
    Then 成功する
    And skipped に "content.purpose.blockType" を含む
    And written は空
