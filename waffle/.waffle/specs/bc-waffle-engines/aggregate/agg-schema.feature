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
