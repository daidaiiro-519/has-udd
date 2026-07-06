Feature: uc-scaffold-document

  Scenario: 既存documentへの再createはvaluesを破壊しない
    Given create済みかつfillで値を書き込み済みのdocumentId
    When 同じdocumentIdでcreateを再実行する
    Then fillで書き込んだvaluesは保持されたままである

  Scenario: 生成した骨格は自分の schema で valid
    Given engine 種別の Document（discriminator 指定済み）
    When create する
    Then 骨格は schema に適合し、status は schema の初期値である

  Scenario: 構造を変える値は拒否される
    Given 作成済みの Document
    When const フィールドへ値を書き込もうとする
    Then 書き込まれず skipped に記録される

  Scenario: 宣言済みの値フィールドに書き込まれる
    Given 作成済みの Document
    When 宣言済みの値フィールドへ値を書き込む
    Then written に記録され、ファイルに反映される

  Scenario: discriminator が無いと候補を案内する
    Given 分岐のある schema
    When discriminator を指定せずに create する
    Then MISSING_DISCRIMINATOR エラーが候補つきで返る

  Scenario: createはengine_skillの骨格を生成する
    Given schemaRef, documentId, discriminator(skillKind=engine)
    When createを実行する
    Then documentType/schemaRef/skillKind/statusが正しく設定され、content配下にinterface/invocationSpecがある骨格が生成される

  Scenario: createはx_source_targetに骨格を書き出す
    Given schemaRef, documentId, discriminator
    When createを実行する
    Then schemaのx-source-target宣言どおりのパスにファイルが書き出される

  Scenario: fillTemplateは値フィールドのpathとprompt_x_prompt_writeを持つ
    Given schemaRef, documentId, discriminator
    When createを実行する
    Then fillTemplateには値フィールドのpathとx-prompt-write由来のpromptを持つエントリが含まれる

  Scenario: customはengineと構成が異なる
    Given discriminator(skillKind=custom)
    When createを実行する
    Then engineとは異なりcontent配下にprocessingTargetを持つ骨格が生成される

  Scenario: 未知のschemaRefはINVALID_SCHEMA_REF
    Given 解決できないschemaRef
    When createを実行する
    Then INVALID_SCHEMA_REFエラーが返る
