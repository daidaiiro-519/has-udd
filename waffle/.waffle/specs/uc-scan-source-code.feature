Feature: uc-scan-source-code

  Scenario: 公開要素の docstring を構造化抽出する
    Given code_scan と対象コードベース、および google kind
    When 対象パスを走査する
    Then 各公開要素が {summary, body, args, returns, raises} を持つ構造で返る

  Scenario: docstring が無い要素も走査全体を失敗させない
    Given docstring を持たない公開関数を含む対象コードベース
    When 対象パスを走査する
    Then 走査は成功し、docstring が無い要素は summary 等が空の値で返る

  Scenario: 対応する kind が無い言語は UNSUPPORTED_KIND
    Given DocCommentSchema に定義の無い言語のコードベース
    When 対象パスを走査する
    Then UNSUPPORTED_KIND エラーが返る

  Scenario: 存在しないパスは INVALID_PATH
    When 存在しないパスを走査する
    Then INVALID_PATH エラーが返る
