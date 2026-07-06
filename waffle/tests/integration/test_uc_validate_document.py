"""uc-validate-document のguaranteeScenarios(operationGuaranteesと対)のうち、
リポジトリ解決契約(対象パス/schemaRef)に対応する統合テスト。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.validate_engine import ValidateEngine
from waffle.shared.result import Err


def _engine() -> ValidateEngine:
    return ValidateEngine(FsDocumentRepository(), PackageSchemaRepository(), JsonSchemaValidator())


def test_存在しないパスはINVALID_PATH():
    """
    Given 実在しない対象パス
    When 本usecaseを実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("does/not/exist.json")
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"


def test_解決できないschemaRefはINVALID_SCHEMA_REF():
    """
    Given 解決できないschemaRef
    When 本usecaseを実行する
    Then INVALID_SCHEMA_REFエラーが返る
    """
    import json
    import tempfile

    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump({"documentId": "x", "schemaRef": "NoSuchSchema/v1"}, f)
        path = f.name

    result = _engine().run(path)
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_SCHEMA_REF"
