"""uc-scaffold-document のguaranteeScenarios(operationGuaranteesと対)に対応する統合テスト。

リポジトリ解決契約(対象パス/schemaRef)と、create自体の冪等性保証(実document.jsonへの
書き込みを伴う)の両方を、実adapter経由で検証する。
"""
from pathlib import Path

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.scaffold_engine import ScaffoldEngine
from waffle.shared.result import Err, Ok

_SKILL_SCHEMA = "SkillSchema/v1"
_TEST_DOC_ID = "test-acceptance-poc-migration"
_TEST_DOC_PATH = f".waffle/documents/skills/{_TEST_DOC_ID}.json"


def _engine() -> ScaffoldEngine:
    return ScaffoldEngine(FsDocumentRepository(), PackageSchemaRepository())


def setup_function():
    Path(_TEST_DOC_PATH).unlink(missing_ok=True)


def teardown_function():
    Path(_TEST_DOC_PATH).unlink(missing_ok=True)


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


def test_既存documentへの再createはvaluesを破壊しない():
    """
    Given create済みかつfillで値を書き込み済みのdocumentId
    When 同じdocumentIdでcreateを再実行する
    Then fillで書き込んだvaluesは保持されたままである
    """
    create_result = _engine().run(
        "create",
        {"schemaRef": _SKILL_SCHEMA, "documentId": _TEST_DOC_ID, "discriminator": {"skillKind": "engine"}},
    )
    assert isinstance(create_result, Ok), create_result

    fill_result = _engine().run(
        "fill",
        {"documentPath": create_result.value["path"], "values": {"content.purpose.text": "ドメインを分析する"}},
    )
    assert isinstance(fill_result, Ok), fill_result

    recreate_result = _engine().run(
        "create",
        {"schemaRef": _SKILL_SCHEMA, "documentId": _TEST_DOC_ID, "discriminator": {"skillKind": "engine"}},
    )
    assert isinstance(recreate_result, Ok), recreate_result

    doc = FsDocumentRepository().load(create_result.value["path"])
    assert doc["content"]["purpose"]["text"] == "ドメインを分析する"
