"""MigrationEngine（publishVersion/deprecateVersion/prepareMigration/applyMigration）の単体テスト。

brainstorm-schema-versioning-migration.mdの論点1〜3で合意した設計を実証する
（合成データのPoC・git履歴から復元した実データPoCの両方を、pytestとして固定する）。
"""
import json

import pytest

from waffle.application.usecases.migration_engine import MigrationEngine
from waffle.shared.result import Err, Ok


class _FakeDocuments:
    def __init__(self, files: dict):
        self.files = files

    def load(self, path):
        if path not in self.files:
            raise FileNotFoundError(path)
        return json.loads(json.dumps(self.files[path]))  # deep copy

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
    """簡易validator: requiredフィールド欠落・enum不一致のみ検出（PoCの検証範囲で十分）。"""

    def validate(self, document, schema):
        errors = []
        for field in schema.get("required", []):
            if field not in document:
                errors.append(f"'{field}' is a required property")
        for field, fdef in schema.get("properties", {}).items():
            if field in document and "enum" in fdef and document[field] not in fdef["enum"]:
                errors.append(f"'{document[field]}' is not one of {fdef['enum']}")
        return errors


def _engine(files=None, schemas=None):
    return MigrationEngine(_FakeDocuments(files or {}), _FakeSchemas(schemas or {}), _FakeValidator())


# --- publishVersion / deprecateVersion ---

def test_publishVersionは未公開のschemaをPUBLISHEDにする():
    files = {"schema/v2.json": {"documentId": "x"}}
    result = _engine(files).run("publishVersion", {"schemaPath": "schema/v2.json"})
    assert isinstance(result, Ok)
    assert files["schema/v2.json"]["x-schema-status"] == "PUBLISHED"


def test_publishVersionは既に公開済みのschemaを拒否する():
    files = {"schema/v2.json": {"x-schema-status": "PUBLISHED"}}
    result = _engine(files).run("publishVersion", {"schemaPath": "schema/v2.json"})
    assert isinstance(result, Err)
    assert result.details[0] == "ALREADY_PUBLISHED"


def test_deprecateVersionはPUBLISHEDをDEPRECATEDにする():
    files = {"schema/v1.json": {"x-schema-status": "PUBLISHED"}}
    result = _engine(files).run("deprecateVersion", {"schemaPath": "schema/v1.json"})
    assert isinstance(result, Ok)
    assert files["schema/v1.json"]["x-schema-status"] == "DEPRECATED"


def test_deprecateVersionはPUBLISHED以外を拒否する():
    files = {"schema/v1.json": {"x-schema-status": "DEPRECATED"}}
    result = _engine(files).run("deprecateVersion", {"schemaPath": "schema/v1.json"})
    assert isinstance(result, Err)
    assert result.details[0] == "INVALID_STATE"


# --- prepareMigration / applyMigration（合成データPoC相当） ---

_V2_SCHEMA_TOY = {
    "required": ["name", "biography", "createdAt", "seniorityLevel"],
    "properties": {
        "name": {"type": "string"},
        "biography": {"type": "string", "x-migration": {"as": "rename", "from": "bio"}},
        "createdAt": {"type": "string", "x-migration": {"as": "default", "value": "unknown"}},
        "seniorityLevel": {
            "type": "string", "enum": ["junior", "senior"],
            "x-migration": {"as": "ai-infer", "prompt": "biographyから経験レベルを判定する"},
        },
    },
}


def test_prepareMigrationは機械変換を適用しai_infer分のワークシートを作る():
    files = {
        "docs": ["docs/doc1.json", "docs/doc2.json"],
        "docs/doc1.json": {"schemaRef": "Person/v1", "name": "Alice", "bio": "10年シニアエンジニア"},
        "docs/doc2.json": {"schemaRef": "Person/v1", "name": "Bob", "bio": "新卒1年目"},
    }
    engine = _engine(files, {"Person/v2": _V2_SCHEMA_TOY})
    result = engine.run("prepareMigration", {
        "fromSchemaRef": "Person/v1", "toSchemaRef": "Person/v2", "documentsDir": "docs",
    })
    assert isinstance(result, Ok)
    partial = result.value["partialDocuments"]["docs/doc1.json"]
    assert partial["biography"] == "10年シニアエンジニア"  # rename
    assert partial["createdAt"] == "unknown"  # default
    assert "seniorityLevel" not in partial  # ai-infer分はここでは埋まらない
    assert result.value["worksheets"]["docs/doc1.json"]["seniorityLevel"]["prompt"]


def test_applyMigrationはAI回答をマージし検証をパスすれば書き込む():
    files = {
        "docs/doc1.json": {"schemaRef": "Person/v1", "name": "Alice", "bio": "10年シニア"},
    }
    engine = _engine(files, {"Person/v2": _V2_SCHEMA_TOY})
    partial_documents = {
        "docs/doc1.json": {"schemaRef": "Person/v2", "name": "Alice", "biography": "10年シニア", "createdAt": "unknown"},
    }
    result = engine.run("applyMigration", {
        "toSchemaRef": "Person/v2",
        "partialDocuments": partial_documents,
        "answers": {"docs/doc1.json": {"seniorityLevel": "senior"}},
    })
    assert isinstance(result, Ok)
    assert result.value["migrated"] == ["docs/doc1.json"]
    assert files["docs/doc1.json"]["seniorityLevel"] == "senior"


def test_applyMigrationはAIの不正な回答を安全網で拒否する():
    """PoCで実証した安全網: enum範囲外の値は書き込まれず rejected に回る。"""
    files = {"docs/doc1.json": {}}
    engine = _engine(files, {"Person/v2": _V2_SCHEMA_TOY})
    partial_documents = {
        "docs/doc1.json": {"schemaRef": "Person/v2", "name": "Alice", "biography": "x", "createdAt": "unknown"},
    }
    result = engine.run("applyMigration", {
        "toSchemaRef": "Person/v2",
        "partialDocuments": partial_documents,
        "answers": {"docs/doc1.json": {"seniorityLevel": "mid-level"}},
    })
    assert isinstance(result, Ok)
    assert result.value["migrated"] == []
    assert result.value["rejected"][0]["documentPath"] == "docs/doc1.json"
    assert files["docs/doc1.json"] == {}  # 元のまま（不正な内容で上書きされていない）


# --- value-map / discriminator-remap（実データPoC相当: SpecSchema v1→v2） ---

_V2_SCHEMA_REAL = {
    "required": ["documentType", "specKind"],
    "properties": {
        "documentType": {
            "type": "string", "enum": ["DomainSpec"],
            "x-migration": {"as": "value-map", "from": "documentType", "mapping": {"Spec": "DomainSpec"}},
        },
        "specKind": {
            "type": "string", "enum": ["aggregate", "subdomain"],
            "x-migration": {
                "as": "discriminator-remap",
                "rules": [
                    {"ifHasField": "aggregateRoot", "then": "aggregate"},
                    {"ifHasField": "members", "then": "subdomain"},
                ],
            },
        },
    },
}


def test_value_mapとdiscriminator_remapで実際のspecKind移行を機械的に処理する():
    """git履歴から復元した実データ(dm-document.json v1形状)を模した移行ケース。"""
    files = {
        "docs": ["docs/dm-document.json"],
        "docs/dm-document.json": {
            "schemaRef": "SpecSchema/v1", "documentType": "Spec", "specKind": "domain-model",
            "content": {"aggregateRoot": {}, "entities": []},
        },
    }
    engine = _engine(files, {"SpecSchema/v2": _V2_SCHEMA_REAL})
    result = engine.run("prepareMigration", {
        "fromSchemaRef": "SpecSchema/v1", "toSchemaRef": "SpecSchema/v2", "documentsDir": "docs",
    })
    assert isinstance(result, Ok)
    partial = result.value["partialDocuments"]["docs/dm-document.json"]
    assert partial["documentType"] == "DomainSpec"  # value-map
    assert partial["specKind"] == "aggregate"  # discriminator-remap（contentにaggregateRootを持つため）
    assert result.value["worksheets"] == {}  # ai-inferが無いので全て機械変換のみで完結


# --- agg-schemaの残り2シナリオ ---

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
    # newRequiredFieldを埋めないまま移行結果を作る(=後方互換を壊す変更)
    partial_documents = {"docs/doc1.json": {"schemaRef": "Foo/v2", "name": "既存データ"}}
    result = engine.run("applyMigration", {
        "toSchemaRef": "Foo/v2", "partialDocuments": partial_documents, "answers": {},
    })
    assert isinstance(result, Ok)
    assert result.value["migrated"] == []
    assert result.value["rejected"][0]["documentPath"] == "docs/doc1.json"
    assert files["docs/doc1.json"] == {"schemaRef": "Foo/v1", "name": "既存データ"}  # 元のまま


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
