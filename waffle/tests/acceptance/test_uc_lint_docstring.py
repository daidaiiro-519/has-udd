"""uc-lint-docstring の受け入れテスト（ネイティブpytest）。"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.pydoclint_linter import PydoclintLinter
from waffle.adapters.outbound.python_ast_source_scanner import PythonAstSourceScanner
from waffle.application.usecases.lint_docstring_engine import LintDocstringEngine
from waffle.application.usecases.scan_source_code_engine import ScanSourceCodeEngine
from waffle.shared.result import Err, Ok


def _engine(executable: str = "pydoclint") -> LintDocstringEngine:
    scan_engine = ScanSourceCodeEngine(FsDocumentRepository(), PythonAstSourceScanner())
    return LintDocstringEngine(scan_engine, PydoclintLinter(executable))


def test_全要素が規約に適合するとき違反なしと判定する(tmp_path):
    """
    Given DocstringSchema の google kind に適合する docstring だけを持つコードベース
    When 適合判定を実行する
    Then 違反は空配列で返り、エラーにはならない
    """
    (tmp_path / "sample.py").write_text(
        'def greet(name):\n'
        '    """挨拶を返す。\n\n'
        '    Args:\n'
        '        name: 相手の名前。\n'
        '    """\n'
        '    return f"hello {name}"\n',
        encoding="utf-8",
    )

    result = _engine().run(str(tmp_path), "google")
    assert isinstance(result, Ok), result
    assert result.value == []


def test_docstring_が無い公開要素を検出する(tmp_path):
    """
    Given docstring を持たない公開関数を含むコードベース
    When 適合判定を実行する
    Then その要素について MISSING_DOC_COMMENT 違反が報告される
    """
    (tmp_path / "sample.py").write_text("def undocumented(x):\n    return x\n", encoding="utf-8")

    result = _engine().run(str(tmp_path), "google")
    assert isinstance(result, Ok), result
    violation = next(v for v in result.value if v["name"] == "undocumented")
    assert violation["code"] == "MISSING_DOC_COMMENT"


def test_Args_の引数名がシグネチャと不一致な要素を検出する(tmp_path):
    """
    Given Args セクションの引数名が実シグネチャと異なる関数を含むコードベース
    When 適合判定を実行する
    Then その要素について ARGS_MISMATCH 違反が報告される
    """
    (tmp_path / "sample.py").write_text(
        'def mismatched(a, b):\n'
        '    """要約。\n\n'
        '    Args:\n'
        '        a: first.\n'
        '        c: wrong name.\n'
        '    """\n'
        '    return a\n',
        encoding="utf-8",
    )

    result = _engine().run(str(tmp_path), "google")
    assert isinstance(result, Ok), result
    violation = next(v for v in result.value if v["name"] == "mismatched")
    assert violation["code"] == "ARGS_MISMATCH"


def test_対応する_kind_が無い言語は_UNSUPPORTED_KIND(tmp_path):
    """
    Given DocstringSchema に定義の無い言語のコードベース
    When 適合判定を実行する
    Then UNSUPPORTED_KIND エラーが返る
    """
    (tmp_path / "sample.py").write_text("def f():\n    pass\n", encoding="utf-8")

    result = _engine().run(str(tmp_path), "cobol")
    assert isinstance(result, Err), result
    assert result.details[0] == "UNSUPPORTED_KIND"


def test_対応するツールが実行環境に無いとき_TOOL_NOT_AVAILABLE(tmp_path):
    """
    Given kind に対応する lint ツールがインストールされていない環境
    When 適合判定を実行する
    Then TOOL_NOT_AVAILABLE エラーが返る
    """
    (tmp_path / "sample.py").write_text(
        'def greet(name):\n    """挨拶。\n\n    Args:\n        name: 名前。\n    """\n    pass\n',
        encoding="utf-8",
    )

    result = _engine(executable="no-such-lint-tool").run(str(tmp_path), "google")
    assert isinstance(result, Err), result
    assert result.details[0] == "TOOL_NOT_AVAILABLE"
