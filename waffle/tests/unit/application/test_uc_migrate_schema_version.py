"""uc-migrate-schema-version の TestScenarios に対応するネイティブテスト。

brainstorm-schema-versioning-migration.mdの論点1〜3で合意した設計を実証する
（合成データのPoC・git履歴から復元した実データPoCの両方を、pytestとして固定する）。
agg-schema由来の2シナリオ(後方互換・移行方向)は test_agg_schema.py に集約した。
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
    """
    Given x-schema-statusが未設定のSchemaファイル
    When publishVersionを実行する
    Then x-schema-statusがPUBLISHEDになる
    """
    files = {"schema/v2.json": {"documentId": "x"}}
    result = _engine(files).run("publishVersion", {"schemaPath": "schema/v2.json"})
    assert isinstance(result, Ok)
    assert files["schema/v2.json"]["x-schema-status"] == "PUBLISHED"


def test_publishVersionは既に公開済みのschemaを拒否する():
    """
    Given x-schema-statusが既に設定されたSchemaファイル
    When publishVersionを実行する
    Then ALREADY_PUBLISHEDエラーが返る
    """
    files = {"schema/v2.json": {"x-schema-status": "PUBLISHED"}}
    result = _engine(files).run("publishVersion", {"schemaPath": "schema/v2.json"})
    assert isinstance(result, Err)
    assert result.details[0] == "ALREADY_PUBLISHED"


def test_deprecateVersionはPUBLISHEDをDEPRECATEDにする():
    """
    Given PUBLISHEDのSchemaファイル
    When deprecateVersionを実行する
    Then x-schema-statusがDEPRECATEDになる
    """
    files = {"schema/v1.json": {"x-schema-status": "PUBLISHED"}}
    result = _engine(files).run("deprecateVersion", {"schemaPath": "schema/v1.json"})
    assert isinstance(result, Ok)
    assert files["schema/v1.json"]["x-schema-status"] == "DEPRECATED"


def test_deprecateVersionはPUBLISHED以外を拒否する():
    """
    Given PUBLISHED以外の状態のSchemaファイル
    When deprecateVersionを実行する
    Then INVALID_STATEエラーが返る
    """
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
    """
    Given rename/default/ai-infer宣言を持つ新schemaと旧schema形状のDocument
    When prepareMigrationを実行する
    Then 機械変換フィールドは適用され、ai-infer分だけがワークシートとして返る
    """
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
    """
    Given 機械変換済みの部分DocumentとAIが埋めたai-infer分の回答
    When applyMigrationを実行する
    Then マージ結果が新schemaで検証に通り書き込まれる
    """
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
    """
    Given 新schemaのenum範囲外の値を含むAI回答
    When applyMigrationを実行する
    Then 書き込まれずrejectedとして報告される

    (PoCで実証した安全網: enum範囲外の値は書き込まれず rejected に回る)
    """
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
    """
    Given value-map/discriminator-remap宣言を持つ新schemaと、旧documentType/specKindを持つDocument
    When prepareMigrationを実行する
    Then 値の対応表と旧content構造の照合により、AIの推論を介さず機械的に新しい値へ変換される

    (git履歴から復元した実データ(dm-document.json v1形状)を模した移行ケース)
    """
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
