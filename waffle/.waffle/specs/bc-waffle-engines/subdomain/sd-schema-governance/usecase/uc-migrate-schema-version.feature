Feature: uc-migrate-schema-version

  Scenario: publishVersionは未公開のschemaをPUBLISHEDにする
    Given x-schema-statusが未設定のSchemaファイル
    When publishVersionを実行する
    Then x-schema-statusがPUBLISHEDになる

  Scenario: publishVersionは既に公開済みのschemaを拒否する
    Given x-schema-statusが既に設定されたSchemaファイル
    When publishVersionを実行する
    Then ALREADY_PUBLISHEDエラーが返る

  Scenario: deprecateVersionはPUBLISHEDをDEPRECATEDにする
    Given PUBLISHEDのSchemaファイル
    When deprecateVersionを実行する
    Then x-schema-statusがDEPRECATEDになる

  Scenario: deprecateVersionはPUBLISHED以外を拒否する
    Given PUBLISHED以外の状態のSchemaファイル
    When deprecateVersionを実行する
    Then INVALID_STATEエラーが返る

  Scenario: prepareMigrationは機械変換を適用しai-infer分のワークシートを作る
    Given rename/default/ai-infer宣言を持つ新schemaと旧schema形状のDocument
    When prepareMigrationを実行する
    Then 機械変換フィールドは適用され、ai-infer分だけがワークシートとして返る

  Scenario: applyMigrationはAI回答をマージし検証をパスすれば書き込む
    Given 機械変換済みの部分DocumentとAIが埋めたai-infer分の回答
    When applyMigrationを実行する
    Then マージ結果が新schemaで検証に通り書き込まれる

  Scenario: applyMigrationはAIの不正な回答を安全網で拒否する
    Given 新schemaのenum範囲外の値を含むAI回答
    When applyMigrationを実行する
    Then 書き込まれずrejectedとして報告される

  Scenario: value-mapとdiscriminator-remapで実際のspecKind移行を機械的に処理する
    Given value-map/discriminator-remap宣言を持つ新schemaと、旧documentType/specKindを持つDocument
    When prepareMigrationを実行する
    Then 値の対応表と旧content構造の照合により、AIの推論を介さず機械的に新しい値へ変換される

  Scenario: 存在しないパスはINVALID_PATH
    Given 実在しない対象パス
    When 本usecaseを実行する
    Then INVALID_PATHエラーが返る

  Scenario: 解決できないschemaRefはINVALID_SCHEMA_REF
    Given 解決できないschemaRef
    When 本usecaseを実行する
    Then INVALID_SCHEMA_REFエラーが返る
