"""uc-validate-document の受け入れテスト（ネイティブpytest）。

.waffle/specs/.../uc-validate-document.feature は参照専用の仕様書であり、実行対象ではない。
"""
import json
import tempfile
from pathlib import Path

import pytest

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.validate_engine import ValidateEngine
from waffle.shared.result import Err, Ok

_DOGFOOD_DOCUMENTS = [
    (".waffle/documents/skills/harness-query-engine.json", "DRAFT"),
    (".waffle/documents/skills/harness-render-engine.json", "DRAFT"),
    (".waffle/documents/coding/tech-stack-python-hexagonal.json", "ACTIVE"),
    (".waffle/documents/coding/architecture-python-hexagonal.json", "ACTIVE"),
    (".waffle/documents/coding/coding-standard-python-hexagonal.json", "ACTIVE"),
    (".waffle/documents/coding/test-standard-python-hexagonal.json", "ACTIVE"),
    (".waffle/documents/specs/bc-waffle-engines/bc-waffle-engines.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/subdomain/sd-document-engine/sd-document-engine.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/subdomain/sd-validation/sd-validation.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/subdomain/sd-reconciliation/sd-reconciliation.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/aggregate/agg-document.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/aggregate/agg-schema.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/subdomain/sd-document-engine/usecase/uc-query-document.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/subdomain/sd-document-engine/usecase/uc-render-document.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/subdomain/sd-validation/usecase/uc-validate-document.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/subdomain/sd-document-engine/usecase/uc-scaffold-document.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/subdomain/sd-reconciliation/usecase/uc-scan-source-code.json", "VALIDATED"),
    (".waffle/documents/specs/bc-waffle-engines/subdomain/sd-reconciliation/usecase/uc-lint-docstring.json", "VALIDATED"),
]


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


@pytest.mark.parametrize("path,expected_status", _DOGFOOD_DOCUMENTS)
def test_既存documentはschemaに適合する(path, expected_status):
    """
    Given waffle自身のdocument
    When validateする
    Then 成功し、schemaのlifecycleに応じた正しいstatusになる
    """
    result = _engine().run(path)
    assert isinstance(result, Ok), result
    assert result.value["status"] == expected_status


def test_SUPERSEDEDは終端でありvalidateを受け付けない():
    """
    Given SUPERSEDED状態のDocument
    When validateする
    Then INVALID_TRANSITIONエラーが返る
    """
    document = json.loads(
        Path(".waffle/documents/specs/bc-waffle-engines/aggregate/agg-document.json").read_text()
    )
    document["status"] = "SUPERSEDED"
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(document, f)
        path = f.name

    result = _engine().run(path)
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_TRANSITION"


def test_存在しないパスはINVALID_PATH():
    """
    Given 存在しない対象パス
    When validateする
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("does/not/exist.json")
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"


def test_不正なJSONはINVALID_JSON():
    """
    Given 不正なJSONの対象ファイル
    When validateする
    Then INVALID_JSONエラーが返る
    """
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        f.write("{not valid json")
        path = f.name

    result = _engine().run(path)
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_JSON"
