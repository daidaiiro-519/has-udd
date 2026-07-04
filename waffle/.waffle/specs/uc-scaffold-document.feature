Feature: uc-scaffold-document

  Scenario: 生成した骨格は自分の schema で valid
    Given engine 種別の Document（discriminator 指定済み）
    When create する
    Then 骨格は schema に適合し、status は schema の初期値である

  Scenario: 構造を変える値は拒否される
    Given 作成済みの Document
    When const フィールドへ値を書き込もうとする
    Then 書き込まれず skipped に記録される
