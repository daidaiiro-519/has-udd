"""agg-schema（Schema集約）のunitTestScenariosのうち、SchemaRepositoryを介する
（portが必要な）ものに対応するネイティブテスト。

document_loading.load_schema() の「解決＋エラーマッピング」編成手順を、port test doubleで検証する。
"""
from waffle.application.services.document_loading import load_schema
from waffle.shared.result import Err


class _FakeSchemas:
    def __init__(self, schemas: dict):
        self.schemas = schemas

    def load(self, schema_ref):
        if schema_ref not in self.schemas:
            raise FileNotFoundError(schema_ref)
        return self.schemas[schema_ref]


def test_解決できないschemaRefはINVALID_SCHEMA_REFとして拒否される():
    """
    Given 解決できないschemaRef
    When 任意のoperation・commandを実行する
    Then INVALID_SCHEMA_REFエラーが返る
    """
    result = load_schema(_FakeSchemas({}), "Unknown/v1")
    assert isinstance(result, Err)
    assert result.details[0] == "INVALID_SCHEMA_REF"
