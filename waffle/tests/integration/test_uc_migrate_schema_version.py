"""uc-migrate-schema-version のguaranteeScenarios(operationGuaranteesと対)のうち、
リポジトリ解決契約(対象パス/schemaRef)に対応する統合テスト。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.migration_engine import MigrationEngine
from waffle.shared.result import Err


def _engine() -> MigrationEngine:
    return MigrationEngine(FsDocumentRepository(), PackageSchemaRepository(), JsonSchemaValidator())


def test_存在しないパスはINVALID_PATH():
    """
    Given 実在しない対象パス
    When 本usecaseを実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("publishVersion", {"schemaPath": "does/not/exist.json"})
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"


def test_解決できないschemaRefはINVALID_SCHEMA_REF():
    """
    Given 解決できないschemaRef
    When 本usecaseを実行する
    Then INVALID_SCHEMA_REFエラーが返る
    """
    result = _engine().run("prepareMigration", {
        "fromSchemaRef": "NoSuchSchema/v1", "toSchemaRef": "NoSuchSchema/v2", "documentsDir": "docs",
    })
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_SCHEMA_REF"
