"""agg-schema(Schema集約)の不変条件のうち、構造的に機械検証できるものの単体テスト。"""
import pytest

from waffle.adapters.outbound.schema_repo import PackageSchemaRepository

_IN_SCOPE_SCHEMAS = ["SkillSchema/v1", "CodingSchema/v2", "DomainSpecSchema/v2", "PresentationSpecSchema/v1", "PlatformSpec/v1"]


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

    (実証: 全 in-scope schema の $defs に oneOf/anyOf が一切含まれない)
    """
    schema = PackageSchemaRepository().load(schema_ref)
    assert not _contains_oneof_or_anyof(schema["$defs"]), f"{schema_ref} の $defs に oneOf/anyOf が含まれている"
