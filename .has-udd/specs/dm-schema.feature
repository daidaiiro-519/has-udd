Feature: Schema 集約の不変条件

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
