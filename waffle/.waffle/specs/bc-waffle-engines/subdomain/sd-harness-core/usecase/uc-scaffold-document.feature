Feature: uc-scaffold-document

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
