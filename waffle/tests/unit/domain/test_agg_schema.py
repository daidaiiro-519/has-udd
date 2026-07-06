"""agg-schema（Schema集約）のunitTestScenariosに対応するネイティブテスト。

test-standardの命名規約(test_{documentId}.py)に従い、以前
test_schema_invariants.py/test_migration_meta_schema.py/test_schema_status.py/
test_part_renderer.py(x-render/lint分)/test_migration_engine.py(後方互換/移行方向分)
に散らばっていたagg-schema由来のテストをここに集約した。
"""
import json

import pytest

from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.migration_engine import MigrationEngine
from waffle.shared.result import Err, Ok

_IN_SCOPE_SCHEMAS = ["SkillSchema/v1", "CodingSchema/v2", "DomainSpecSchema/v2", "PresentationSpecSchema/v1", "PlatformSpec/v1"]
_VALID_STATUSES = {"PUBLISHED", "DEPRECATED"}


# --- 値フィールドに oneOf を持てない ---

def _contains_oneof_or_anyof(node) -> bool:
    if isinstance(node, dict):
        if "oneOf" in node or "anyOf" in node:
            return True
        return any(_contains_oneof_or_anyof(v) for v in node.values())
    if isinstance(node, list):
        return any(_contains_oneof_or_anyof(v) for v in node)
    return False


@pytest.mark.parametrize("schema_ref", _IN_SCOPE_SCHEMAS)
def test_値フィールドに_oneOf_を持てない(schema_ref):
    """
    Given 値フィールドに oneOf を含む Schema
    When scaffoldability を検証する
    Then scaffold 不能として拒否される
    """
    schema = PackageSchemaRepository().load(schema_ref)
    assert not _contains_oneof_or_anyof(schema["$defs"]), f"{schema_ref} の $defs に oneOf/anyOf が含まれている"


# --- x-render は閉じた語彙にのみ従う ---

def _lint_render(parts):
    meta = PackageSchemaRepository().load("RenderMetaSchema/v1")
    schema = {"$defs": meta["$defs"], "type": "array", "items": {"$ref": "#/$defs/RenderPart"}}
    return JsonSchemaValidator().validate(parts, schema)


def test_lint_accepts_valid():
    assert _lint_render([{"as": "paragraph", "from": "text"},
                          {"as": "table", "from": "rows", "columns": [{"field": "name"}]}]) == []


def test_lint_rejects_unknown_part():
    assert _lint_render([{"as": "foobar", "from": "x"}])  # enum 違反で非空


def test_lint_rejects_missing_required_attr():
    assert _lint_render([{"as": "table", "from": "rows"}])  # columns 漏れで非空


@pytest.mark.parametrize("schema_ref", _IN_SCOPE_SCHEMAS)
def test_x_render_は閉じた語彙にのみ従う(schema_ref):
    """
    Given 未知の部品種別、または必須属性が欠けた x-render 宣言を持つ Schema
    When x-render の適合を検証する
    Then 不適合として拒否される

    (実証: 全 in-scope schema の全 block の x-render が RenderMetaSchema に適合する)
    """
    schema = PackageSchemaRepository().load(schema_ref)
    for name, bdef in schema["$defs"].items():
        if "x-render" in bdef:
            assert _lint_render(bdef["x-render"]) == [], f"{schema_ref}:{name} の x-render が不適合"


# --- x-migration は MigrationMetaSchema の閉じた語彙にのみ従う ---

def _lint_migration(declaration: dict):
    meta = PackageSchemaRepository().load("MigrationMetaSchema/v1")
    schema = {"$defs": meta["$defs"], "$ref": "#/$defs/MigrationDeclaration"}
    return JsonSchemaValidator().validate(declaration, schema)


def test_rename宣言は_from_を要求する():
    assert _lint_migration({"as": "rename", "from": "bio"}) == []
    assert _lint_migration({"as": "rename"}) != []


def test_default宣言は_value_を要求する():
    assert _lint_migration({"as": "default", "value": "unknown"}) == []
    assert _lint_migration({"as": "default"}) != []


def test_value_map宣言は_from_と_mapping_を要求する():
    assert _lint_migration({"as": "value-map", "from": "documentType", "mapping": {"Spec": "DomainSpec"}}) == []
    assert _lint_migration({"as": "value-map", "from": "documentType"}) != []


def test_discriminator_remap宣言は_rules_を要求する():
    assert _lint_migration({"as": "discriminator-remap", "rules": [{"ifHasField": "aggregateRoot", "then": "aggregate"}]}) == []
    assert _lint_migration({"as": "discriminator-remap"}) != []


def test_ai_infer宣言は_prompt_を要求する():
    assert _lint_migration({"as": "ai-infer", "prompt": "経験レベルを判定する"}) == []
    assert _lint_migration({"as": "ai-infer"}) != []


def test_未知の種別は拒否される():
    assert _lint_migration({"as": "foobar"}) != []


@pytest.mark.parametrize("schema_ref", ["MigrationMetaSchema/v1"])
def test_schema自体がロードできる(schema_ref):
    schema = PackageSchemaRepository().load(schema_ref)
    assert "$defs" in schema
    assert "MigrationDeclaration" in schema["$defs"]


# --- Documentのschemaが指しうる型は常にx-schema-statusを宣言する ---

@pytest.mark.parametrize("schema_ref", _IN_SCOPE_SCHEMAS)
def test_全schemaがx_schema_statusを宣言している(schema_ref):
    """
    Given Documentのschemaが指しうる型(DomainSpecSchema/PresentationSpecSchema/CodingSchema/SkillSchema)
    When 各schemaファイルのx-schema-statusを確認する
    Then 全てPUBLISHED/DEPRECATEDのいずれかを宣言している
    """
    schema = PackageSchemaRepository().load(schema_ref)
    assert "x-schema-status" in schema, f"{schema_ref} は x-schema-status を宣言していない"
    assert schema["x-schema-status"] in _VALID_STATUSES


# --- 公開済みの版は後方互換を壊さない / 移行は版を上げる方向にのみ行う ---
# (MigrationEngine経由でのみ実証できる集約レベルの不変条件)

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
