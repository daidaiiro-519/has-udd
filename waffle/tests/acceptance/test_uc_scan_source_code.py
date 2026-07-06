"""uc-scan-source-code の受け入れテスト（ネイティブpytest）。"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.python_ast_source_scanner import PythonAstSourceScanner
from waffle.application.usecases.scan_source_code_engine import ScanSourceCodeEngine
from waffle.shared.result import Err, Ok


def _engine() -> ScanSourceCodeEngine:
    return ScanSourceCodeEngine(FsDocumentRepository(), PythonAstSourceScanner())


def test_公開要素の_docstring_を構造化抽出する(tmp_path):
    """
    Given 対象コードベースと google kind
    When 対象パスを走査する
    Then 各公開要素が {summary, body, args, returns, raises} を持つ構造で返る
    """
    (tmp_path / "sample.py").write_text(
        'def greet(name):\n'
        '    """挨拶を返す。\n\n'
        '    Args:\n'
        '        name: 相手の名前。\n\n'
        '    Returns:\n'
        '        挨拶文字列。\n'
        '    """\n'
        '    return f"hello {name}"\n',
        encoding="utf-8",
    )

    result = _engine().run(str(tmp_path), "google")
    assert isinstance(result, Ok), result
    greet = next(e for e in result.value if e["name"] == "greet")
    assert set(["summary", "body", "args", "returns", "raises"]).issubset(greet.keys())
    assert greet["summary"] == "挨拶を返す。"
    assert greet["args"] == [{"name": "name", "description": "相手の名前。"}]
    assert greet["returns"] == "挨拶文字列。"


def test_docstring_が無い要素も走査全体を失敗させない(tmp_path):
    """
    Given docstring を持たない公開関数を含む対象コードベース
    When 対象パスを走査する
    Then 走査は成功し、docstring が無い要素は summary 等が空の値で返る
    """
    (tmp_path / "sample.py").write_text("def undocumented(x):\n    return x\n", encoding="utf-8")

    result = _engine().run(str(tmp_path), "google")
    assert isinstance(result, Ok), result
    el = next(e for e in result.value if e["name"] == "undocumented")
    assert el["hasDocstring"] is False
    assert el["summary"] == ""
    assert el["args"] == []


def test_対応する_kind_が無い言語は_UNSUPPORTED_KIND(tmp_path):
    """
    Given DocstringSchema に定義の無い言語のコードベース
    When 対象パスを走査する
    Then UNSUPPORTED_KIND エラーが返る
    """
    (tmp_path / "sample.py").write_text("def f():\n    pass\n", encoding="utf-8")

    result = _engine().run(str(tmp_path), "cobol")
    assert isinstance(result, Err), result
    assert result.details[0] == "UNSUPPORTED_KIND"
