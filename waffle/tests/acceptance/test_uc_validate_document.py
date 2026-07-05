"""uc-validate-document の受け入れテスト（ネイティブpytest）。

.waffle/specs/.../uc-validate-document.feature は参照専用の仕様書であり、実行対象ではない。
"""
import json
import tempfile
from pathlib import Path

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.validate_engine import ValidateEngine
from waffle.shared.result import Err, Ok


def _engine() -> ValidateEngine:
    return ValidateEngine(FsDocumentRepository(), PackageSchemaRepository(), JsonSchemaValidator())


def test_適合する_Document_は_VALIDATED_判定になる():
    """
    Given schema に適合する Document
    When validate する
    Then VALIDATED 判定が返る
    """
    result = _engine().run(".waffle/documents/skills/harness-query-engine.json")
    assert isinstance(result, Ok), result


def test_不適合は違反詳細つきで失敗する():
    """
    Given schema に適合しない Document
    When validate する
    Then 違反詳細つきで失敗する
    """
    valid = json.loads(Path(".waffle/documents/skills/harness-query-engine.json").read_text())
    invalid = dict(valid)
    invalid.pop("documentId")
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(invalid, f)
        path = f.name

    result = _engine().run(path)
    assert isinstance(result, Err), result
    assert len(result.details) >= 1


def test_schemaRef_を持たない_Document_は検証できない():
    """
    Given schemaRef の無い Document
    When validate する
    Then MISSING_SCHEMA_REF エラーが返る
    """
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump({"documentId": "no-schema-ref"}, f)
        path = f.name

    result = _engine().run(path)
    assert isinstance(result, Err), result
    assert result.details[0] == "MISSING_SCHEMA_REF"
