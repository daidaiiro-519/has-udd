"""agg-schema（Schema集約）のunitTestScenariosのうち、port を介さず（実schemaロード
のみで）検証できるものに対応するネイティブテスト。

test-standardの命名規約(test_{documentId}.py)に従い、以前
test_schema_invariants.py/test_migration_meta_schema.py/test_schema_status.py/
test_part_renderer.py(x-render/lint分)に散らばっていたagg-schema由来のテストを
ここに集約した。port をテストダブルにする2シナリオ（後方互換・移行方向）は
tests/unit/application/test_agg_schema.py。
"""
import pytest

from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository

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
    """
    Given as=renameでfromを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される
    """
    assert _lint_migration({"as": "rename", "from": "bio"}) == []
    assert _lint_migration({"as": "rename"}) != []


def test_default宣言は_value_を要求する():
    """
    Given as=defaultでvalueを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される
    """
    assert _lint_migration({"as": "default", "value": "unknown"}) == []
    assert _lint_migration({"as": "default"}) != []


def test_value_map宣言は_from_と_mapping_を要求する():
    """
    Given as=value-mapでmappingを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される
    """
    assert _lint_migration({"as": "value-map", "from": "documentType", "mapping": {"Spec": "DomainSpec"}}) == []
    assert _lint_migration({"as": "value-map", "from": "documentType"}) != []


def test_discriminator_remap宣言は_rules_を要求する():
    """
    Given as=discriminator-remapでrulesを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される
    """
    assert _lint_migration({"as": "discriminator-remap", "rules": [{"ifHasField": "aggregateRoot", "then": "aggregate"}]}) == []
    assert _lint_migration({"as": "discriminator-remap"}) != []


def test_ai_infer宣言は_prompt_を要求する():
    """
    Given as=ai-inferでpromptを欠くx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される
    """
    assert _lint_migration({"as": "ai-infer", "prompt": "経験レベルを判定する"}) == []
    assert _lint_migration({"as": "ai-infer"}) != []


def test_未知の種別は拒否される():
    """
    Given asが未知の種別であるx-migration宣言
    When MigrationMetaSchemaで適合を検証する
    Then 不適合として拒否される
    """
    assert _lint_migration({"as": "foobar"}) != []


@pytest.mark.parametrize("schema_ref", ["MigrationMetaSchema/v1"])
def test_schema自体がロードできる(schema_ref):
    """
    Given MigrationMetaSchema/v1
    When PackageSchemaRepositoryでロードする
    Then $defs.MigrationDeclarationが取得できる
    """
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
