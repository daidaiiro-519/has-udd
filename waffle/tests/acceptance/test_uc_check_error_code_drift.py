"""uc-check-error-code-drift の受け入れテスト（ネイティブpytest）。"""
import json
from pathlib import Path

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_error_code_drift_engine import CheckErrorCodeDriftEngine
from waffle.shared.result import Ok


def _engine() -> CheckErrorCodeDriftEngine:
    return CheckErrorCodeDriftEngine(FsDocumentRepository())


def _write_spec(path: Path, document_id: str, codes: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({
        "documentId": document_id,
        "content": {"errors": {"items": [{"code": c, "condition": "x"} for c in codes]}},
    }, ensure_ascii=False), encoding="utf-8")


def _write_code(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def test_全specのエラーコードが実装に実在するとき差分なしと判定する(tmp_path):
    """
    Given 全usecase specが宣言するエラーコードが、@specタグでリンクされた実装ファイルに文字列として存在するspecツリー
    When エラーコード整合検査を実行する
    Then unlinked_specs・missing_error_codes両方が空配列で返る
    """
    specs_root = tmp_path / "specs"
    code_root = tmp_path / "code"
    _write_spec(specs_root / "uc-a.json", "uc-a", ["FOO_ERROR"])
    _write_code(code_root / "a_engine.py", '"""a engine.\n\n@spec uc-a\n"""\nFOO_ERROR = "FOO_ERROR"\n')

    result = _engine().run(str(specs_root), str(code_root))
    assert isinstance(result, Ok), result
    assert result.value == {"unlinked_specs": [], "missing_error_codes": []}


def test_リンク先ファイルが無いspecを検出する(tmp_path):
    """
    Given content.errors.items[].codeを宣言するが、対応する@specタグを持つファイルが無いusecase spec
    When エラーコード整合検査を実行する
    Then そのdocumentIdがunlinked_specsに含まれる
    """
    specs_root = tmp_path / "specs"
    code_root = tmp_path / "code"
    _write_spec(specs_root / "uc-a.json", "uc-a", ["FOO_ERROR"])
    code_root.mkdir(parents=True)

    result = _engine().run(str(specs_root), str(code_root))
    assert isinstance(result, Ok), result
    assert result.value["unlinked_specs"] == ["uc-a"]


def test_実装コードに存在しないエラーコードを検出する(tmp_path):
    """
    Given リンクされた実装ファイルに文字列として存在しないエラーコードを宣言するusecase spec
    When エラーコード整合検査を実行する
    Then その組がmissing_error_codesに含まれる
    """
    specs_root = tmp_path / "specs"
    code_root = tmp_path / "code"
    _write_spec(specs_root / "uc-a.json", "uc-a", ["FOO_ERROR"])
    code_file = code_root / "a_engine.py"
    _write_code(code_file, '"""a engine.\n\n@spec uc-a\n"""\n')

    result = _engine().run(str(specs_root), str(code_root))
    assert isinstance(result, Ok), result
    assert result.value["missing_error_codes"] == [
        {"usecase": "uc-a", "code": "FOO_ERROR", "files": [str(code_file)]}
    ]
