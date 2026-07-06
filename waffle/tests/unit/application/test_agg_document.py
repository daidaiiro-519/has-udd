"""agg-document（Document集約）のunitTestScenariosのうち、DocumentRepositoryを介する
（portが必要な）ものに対応するネイティブテスト。

パス解決の核心（G6/G7）はport不要の純粋ロジックなのでtests/unit/domain/test_agg_document.py。
ここでは document_loading.load_document() の「読込＋エラーマッピング」編成手順を、
port test doubleで検証する。
"""
from waffle.application.services.document_loading import load_document
from waffle.shared.result import Err, Ok


class _FakeDocuments:
    def __init__(self, files: dict):
        self.files = files

    def load(self, path):
        if path not in self.files:
            raise FileNotFoundError(path)
        return self.files[path]


def test_存在しないパスはINVALID_PATHとして拒否される():
    """
    Given 実在しない対象パス
    When 任意の operation・command を実行する
    Then INVALID_PATH エラーが返る
    """
    result = load_document(_FakeDocuments({}), "does/not/exist.json")
    assert isinstance(result, Err)
    assert result.details[0] == "INVALID_PATH"


def test_schemaRefを持たないDocumentはMISSING_SCHEMA_REFとして拒否される():
    """
    Given schemaRef を持たない Document
    When schema 解決を要する operation・command を実行する
    Then MISSING_SCHEMA_REF エラーが返る
    """
    from waffle.application.services.document_loading import require_schema_ref

    documents = _FakeDocuments({"docs/no-ref.json": {"name": "x"}})
    loaded = load_document(documents, "docs/no-ref.json")
    assert isinstance(loaded, Ok)

    result = require_schema_ref(loaded.value)
    assert isinstance(result, Err)
    assert result.details[0] == "MISSING_SCHEMA_REF"
