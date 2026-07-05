"""Schema集約（agg-schema）の status フィールドが、Documentのschemaが指しうる型
（DomainSpecSchema/PresentationSpecSchema/CodingSchema/SkillSchema）すべてに
宣言されていることを検証する。RenderMetaSchema/DocstringSchemaはagg-schemaの
対象外なので含めない。
"""
import importlib.resources
import json

import pytest

_IN_SCOPE_SCHEMAS = [
    "DomainSpecSchema/v2",
    "PresentationSpecSchema/v1",
    "CodingSchema/v2",
    "SkillSchema/v1",
]
_VALID_STATUSES = {"PUBLISHED", "DEPRECATED"}


def _load_schema(schema_ref: str) -> dict:
    name, version = schema_ref.split("/")
    resource = importlib.resources.files("waffle.domain.model").joinpath(name, f"{version}.json")
    return json.loads(resource.read_text())


@pytest.mark.parametrize("schema_ref", _IN_SCOPE_SCHEMAS)
def test_schema_declares_status(schema_ref):
    schema = _load_schema(schema_ref)
    assert "x-schema-status" in schema, f"{schema_ref} は x-schema-status を宣言していない"
    assert schema["x-schema-status"] in _VALID_STATUSES
