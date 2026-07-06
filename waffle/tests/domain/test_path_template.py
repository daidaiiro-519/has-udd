"""path_template（順方向resolve・逆方向reverse_parse）の単体テスト。

sd-harness-core(subdomain)のドメインサービス「パステンプレート解決」に対応するネイティブテスト。
"""
from waffle.domain.services import path_template


def test_パステンプレートは変数を解決する():
    """
    Given 変数を含むパステンプレートと解決に必要な値
    When resolve する
    Then 全ての変数が値に置き換わった実パスが返る
    """
    template = ".waffle/documents/specs/{contextRef}/aggregate/{documentId}.json"
    path = path_template.resolve(template, contextRef="bc-waffle-engines", documentId="agg-document")
    assert path == ".waffle/documents/specs/bc-waffle-engines/aggregate/agg-document.json"


def test_逆解析は実パスからテンプレート変数を復元する():
    """
    Given パステンプレートと、そのテンプレートから解決された実パス
    When reverse-parse する
    Then resolve時に使った値と同じ変数が復元される
    """
    template = ".waffle/documents/specs/{contextRef}/aggregate/{documentId}.json"
    path = ".waffle/documents/specs/bc-waffle-engines/aggregate/agg-document.json"
    assert path_template.reverse_parse(template, path) == {
        "contextRef": "bc-waffle-engines", "documentId": "agg-document",
    }


def test_テンプレートと一致しないパスは復元できない():
    """
    Given テンプレートの区切り構造と一致しない実パス
    When reverse-parse する
    Then 復元は失敗する
    """
    template = ".waffle/documents/specs/{contextRef}/subdomain/{documentId}/{documentId}.json"
    other_kind_path = ".waffle/documents/specs/bc-waffle-engines/aggregate/agg-document.json"
    assert path_template.reverse_parse(template, other_kind_path) is None


def test_reverse_parse_duplicate_variable_name_self_contained():
    """subdomain の自己格納パターン（フォルダ名=ファイル名=documentId）は同名変数が2回登場する。"""
    template = ".waffle/documents/specs/{contextRef}/subdomain/{documentId}/{documentId}.json"
    path = ".waffle/documents/specs/bc-waffle-engines/subdomain/sd-harness-core/sd-harness-core.json"
    assert path_template.reverse_parse(template, path) == {
        "contextRef": "bc-waffle-engines", "documentId": "sd-harness-core",
    }


def test_reverse_parse_duplicate_variable_name_requires_consistent_value():
    """同名変数の2回目はバックリファレンス＝両方の値が食い違うパスは不一致になる。"""
    template = ".waffle/documents/specs/{contextRef}/subdomain/{documentId}/{documentId}.json"
    inconsistent_path = ".waffle/documents/specs/bc-waffle-engines/subdomain/sd-a/sd-b.json"
    assert path_template.reverse_parse(template, inconsistent_path) is None
