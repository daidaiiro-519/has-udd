"""uc-query-document のguaranteeScenarios(operationGuaranteesと対)に対応する統合テスト。

対象パス/schemaRefのリポジトリ解決契約は、query engineが実DocumentRepository/
SchemaRepositoryを介して解決を試みることそのものを検証するため、実adapter経由で検証する
（domain-model.mdの実例が示す通り、リポジトリのLoad呼び出しとその失敗処理はアプリケーション層の
関心事であり、集約の不変条件ではない）。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.query_engine import QueryEngine
from waffle.shared.result import Err

_TARGET = ".waffle/documents/specs/bc-waffle/subdomain/sd-document-management/usecase/uc-query-document.json"


def _engine() -> QueryEngine:
    return QueryEngine(FsDocumentRepository(), PackageSchemaRepository())


def test_存在しないパスはINVALID_PATH():
    """
    Given 実在しない対象パス
    When 本usecaseを実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("get_block", "does/not/exist.json", {"blockKey": "title"})
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

    result = _engine().run("get_meta", path)
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_SCHEMA_REF"
