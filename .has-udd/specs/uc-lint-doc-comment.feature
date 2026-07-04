Feature: uc-lint-doc-comment

  Scenario: 全要素が規約に適合するとき違反なしと判定する
    Given DocCommentSchema の google kind に適合する docstring だけを持つコードベース
    When 適合判定を実行する
    Then 違反は空配列で返り、エラーにはならない

  Scenario: docstring が無い公開要素を検出する
    Given docstring を持たない公開関数を含むコードベース
    When 適合判定を実行する
    Then その要素について MISSING_DOC_COMMENT 違反が報告される

  Scenario: Args の引数名がシグネチャと不一致な要素を検出する
    Given Args セクションの引数名が実シグネチャと異なる関数を含むコードベース
    When 適合判定を実行する
    Then その要素について ARGS_MISMATCH 違反が報告される

  Scenario: 対応する kind が無い言語は UNSUPPORTED_KIND
    Given DocCommentSchema に定義の無い言語のコードベース
    When 適合判定を実行する
    Then UNSUPPORTED_KIND エラーが返る
