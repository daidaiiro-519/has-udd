"""agg-schema（Schema集約）のunitTestScenariosのうち、MigrationEngine経由でしか
実証できない集約レベルの不変条件（port はテストダブル）。

その他のagg-schema由来テスト（純粋なschemaロード検証）は tests/unit/domain/test_agg_schema.py。
"""
import json

from waffle.application.usecases.migration_engine import MigrationEngine
from waffle.shared.result import Err, Ok


class _FakeDocuments:
    def __init__(self, files: dict):
        self.files = files

    def load(self, path):
        if path not in self.files:
            raise FileNotFoundError(path)
        return json.loads(json.dumps(self.files[path]))

    def save(self, path, document):
        self.files[path] = document

    def write_text(self, path, text):
        self.files[path] = text

    def read_text(self, path):
        return self.files[path]

    def list_json(self, directory):
        if directory not in self.files:
            raise FileNotFoundError(directory)
        return sorted(self.files[directory])


class _FakeSchemas:
    def __init__(self, schemas: dict):
        self.schemas = schemas

    def load(self, schema_ref):
        if schema_ref not in self.schemas:
            raise FileNotFoundError(schema_ref)
        return self.schemas[schema_ref]


class _FakeValidator:
    def validate(self, document, schema):
        errors = []
        for field in schema.get("required", []):
            if field not in document:
                errors.append(f"'{field}' is a required property")
        return errors


def _engine(files=None, schemas=None):
    return MigrationEngine(_FakeDocuments(files or {}), _FakeSchemas(schemas or {}), _FakeValidator())


def test_公開済みの版は後方互換を壊さない():
    """
    Given PUBLISHED の Schema 版
    When 既存ブロックに必須フィールドを追加しようとする
    Then 後方互換を壊す変更として拒否される

    (実証: applyMigrationの実証的検証が、新schemaで要求される必須フィールドを
    欠いた移行結果を書き込まず拒否することを確認する)
    """
    files = {"docs/doc1.json": {"schemaRef": "Foo/v1", "name": "既存データ"}}
    v2_schema_with_new_required_field = {
        "required": ["name", "newRequiredField"],
        "properties": {"name": {"type": "string"}, "newRequiredField": {"type": "string"}},
    }
    engine = _engine(files, {"Foo/v2": v2_schema_with_new_required_field})
    partial_documents = {"docs/doc1.json": {"schemaRef": "Foo/v2", "name": "既存データ"}}
    result = engine.run("applyMigration", {
        "toSchemaRef": "Foo/v2", "partialDocuments": partial_documents, "answers": {},
    })
    assert isinstance(result, Ok)
    assert result.value["migrated"] == []
    assert result.value["rejected"][0]["documentPath"] == "docs/doc1.json"
    assert files["docs/doc1.json"] == {"schemaRef": "Foo/v1", "name": "既存データ"}


def test_移行は版を上げる方向にのみ行う():
    """
    Given v1 と v2 の Schema
    When v2 から v1 へ移行しようとする
    Then 拒否される
    """
    files = {"docs": []}
    engine = _engine(files, {"Foo/v1": {"required": [], "properties": {}}})
    result = engine.run("prepareMigration", {
        "fromSchemaRef": "Foo/v2", "toSchemaRef": "Foo/v1", "documentsDir": "docs",
    })
    assert isinstance(result, Err)
    assert result.details[0] == "INVALID_MIGRATION_DIRECTION"
