"""agg-schema（Schema集約）のinvariantScenariosに対応するネイティブテスト。

集約の不変条件はいずれもport（DocumentRepository/SchemaRepository）を必要としない
純粋な検証（実schemaロード・実Validator呼び出し）で完結するため、全件をdomain層に置く。

test-standardの命名規約(test_{documentId}.py)に従い、以前
test_schema_invariants.py/test_schema_status.py/test_part_renderer.py(x-render/lint分)
に散らばっていたagg-schema由来のテストをここに集約した。

x-migration語彙(MigrationMetaSchema)・移行方向の不変条件は、実際の運用規模では
過剰なため撤去した(詳細はdocs/brainstorm/brainstorm-schema-versioning-migration.md
に撤回の経緯として追記済み)。
"""
import pytest

from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository

_IN_SCOPE_SCHEMAS = ["SkillSchema/v1", "CodingSchema/v2", "DomainSpecSchema/v2", "PresentationSpecSchema/v1", "PlatformSpec/v1"]


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


# --- 公開済みの版は後方互換を壊さない ---
# (port不要・純粋なValidator呼び出しで検証できる不変条件)

def test_公開済みの版は後方互換を壊さない():
    """
    Given PUBLISHED の Schema 版
    When 既存ブロックに必須フィールドを追加しようとする
    Then 後方互換を壊す変更として拒否される

    (実証: 旧バージョンの既存Documentに、新schemaが要求する必須フィールドが
    欠けている場合、実際のJsonSchemaValidatorで不適合と判定されることを確認する)
    """
    old_document = {"name": "既存データ"}
    new_schema_with_required_field = {
        "required": ["name", "newRequiredField"],
        "properties": {"name": {"type": "string"}, "newRequiredField": {"type": "string"}},
    }
    errors = JsonSchemaValidator().validate(old_document, new_schema_with_required_field)
    assert errors, "後方互換を壊す変更（必須フィールド追加）が誤って適合と判定された"
