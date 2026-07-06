"""document_loading（application層の共通編成ヘルパー）の単体テスト。

document_loading.py自体はuc-*のような固有のspec文書を持たない、4usecase共有の
実装ヘルパー。検証している業務保証（INVALID_PATH/INVALID_JSON/INVALID_SCHEMA_REF）
自体はすでに各usecaseのguaranteeScenariosとしてSpecに宣言済みで、
tests/integration/側で宣言名と一致するテスト（Given/When/Then付き）として実アダプタ
経由に検証されている。ここはそれを補完し、共有ロジック自体をDocumentRepository/
SchemaRepository（単純なProtocol）のフェイク実装に差し替えて直接・一度だけ検証する
（シナリオ名との対応は無いため、他の補完テストと同様シナリオ名にはしない）。
"""
import json

from waffle.application.services.document_loading import load_document, load_schema
from waffle.shared.result import Err, Ok


class _FakeDocumentRepository:
    def __init__(self, documents: dict[str, dict] | None = None, raise_on_load: Exception | None = None):
        self._documents = documents or {}
        self._raise_on_load = raise_on_load

    def load(self, path: str) -> dict:
        if self._raise_on_load is not None:
            raise self._raise_on_load
        if path not in self._documents:
            raise FileNotFoundError(path)
        return self._documents[path]

    def save(self, path, document):
        raise NotImplementedError

    def write_text(self, path, text):
        raise NotImplementedError

    def read_text(self, path):
        raise NotImplementedError

    def list_json(self, directory):
        raise NotImplementedError


class _FakeSchemaRepository:
    def __init__(self, schemas: dict[str, dict] | None = None):
        self._schemas = schemas or {}

    def load(self, schema_ref: str) -> dict:
        if schema_ref not in self._schemas:
            raise FileNotFoundError(schema_ref)
        return self._schemas[schema_ref]


def test_load_documentはパストラバーサルをINVALID_PATHとして拒否する():
    result = load_document(_FakeDocumentRepository(), "../outside.json")
    assert isinstance(result, Err)
    assert result.details[0] == "INVALID_PATH"


def test_load_documentは存在しないファイルをINVALID_PATHとして拒否する():
    result = load_document(_FakeDocumentRepository(documents={}), "docs/missing.json")
    assert isinstance(result, Err)
    assert result.details[0] == "INVALID_PATH"


def test_load_documentは不正なJSONをINVALID_JSONとして拒否する():
    fake = _FakeDocumentRepository(raise_on_load=json.JSONDecodeError("bad", "{", 0))
    result = load_document(fake, "docs/broken.json")
    assert isinstance(result, Err)
    assert result.details[0] == "INVALID_JSON"


def test_load_documentは正常なdocumentをOkで返す():
    fake = _FakeDocumentRepository(documents={"docs/valid.json": {"name": "x"}})
    result = load_document(fake, "docs/valid.json")
    assert isinstance(result, Ok)
    assert result.value == {"name": "x"}


def test_load_schemaは解決できないschemaRefをINVALID_SCHEMA_REFとして拒否する():
    result = load_schema(_FakeSchemaRepository(), "Bogus/v1")
    assert isinstance(result, Err)
    assert result.details[0] == "INVALID_SCHEMA_REF"


def test_load_schemaは解決できたschemaをOkで返す():
    fake = _FakeSchemaRepository(schemas={"CodingSchema/v2": {"title": "x"}})
    result = load_schema(fake, "CodingSchema/v2")
    assert isinstance(result, Ok)
    assert result.value == {"title": "x"}
