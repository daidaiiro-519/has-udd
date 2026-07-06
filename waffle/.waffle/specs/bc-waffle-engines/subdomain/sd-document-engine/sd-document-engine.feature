Feature: sd-document-engine

  Scenario: パステンプレートは変数を解決する
    Given 変数を含むパステンプレートと解決に必要な値
    When resolve する
    Then 全ての変数が値に置き換わった実パスが返る

  Scenario: 逆解析は実パスからテンプレート変数を復元する
    Given パステンプレートと、そのテンプレートから解決された実パス
    When reverse-parse する
    Then resolve時に使った値と同じ変数が復元される

  Scenario: テンプレートと一致しないパスは復元できない
    Given テンプレートの区切り構造と一致しない実パス
    When reverse-parse する
    Then 復元は失敗する
