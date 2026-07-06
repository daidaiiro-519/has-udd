"""uc-scaffold-document のguaranteeScenarios(operationGuaranteesと対)のうち、
リポジトリ解決契約(対象パス/schemaRef)に対応する統合テスト。

「既存documentへの再createはvaluesを破壊しない」は別途このファイルへ移設予定(task #80)。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.scaffold_engine import ScaffoldEngine
from waffle.shared.result import Err


def _engine() -> ScaffoldEngine:
    return ScaffoldEngine(FsDocumentRepository(), PackageSchemaRepository())


def test_存在しないパスはINVALID_PATH():
    """
    Given 実在しない対象パス
    When 本usecaseを実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("fill", {"documentPath": "does/not/exist.json", "values": {}})
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"


def test_解決できないschemaRefはINVALID_SCHEMA_REF():
    """
    Given 解決できないschemaRef
    When 本usecaseを実行する
    Then INVALID_SCHEMA_REFエラーが返る
    """
    result = _engine().run(
        "create",
        {"schemaRef": "NoSuchSchema/v1", "documentId": "test-acceptance-poc-migration", "discriminator": {"skillKind": "engine"}},
    )
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_SCHEMA_REF"
