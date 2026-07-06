"""MigrationMetaSchema（x-migration宣言の5種）の単体テスト。"""
import pytest

from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository


def _lint(declaration: dict):
    meta = PackageSchemaRepository().load("MigrationMetaSchema/v1")
    schema = {"$defs": meta["$defs"], "$ref": "#/$defs/MigrationDeclaration"}
    return JsonSchemaValidator().validate(declaration, schema)


def test_rename宣言は_from_を要求する():
    assert _lint({"as": "rename", "from": "bio"}) == []
    assert _lint({"as": "rename"}) != []


def test_default宣言は_value_を要求する():
    assert _lint({"as": "default", "value": "unknown"}) == []
    assert _lint({"as": "default"}) != []


def test_value_map宣言は_from_と_mapping_を要求する():
    assert _lint({"as": "value-map", "from": "documentType", "mapping": {"Spec": "DomainSpec"}}) == []
    assert _lint({"as": "value-map", "from": "documentType"}) != []


def test_discriminator_remap宣言は_rules_を要求する():
    assert _lint({"as": "discriminator-remap", "rules": [{"ifHasField": "aggregateRoot", "then": "aggregate"}]}) == []
    assert _lint({"as": "discriminator-remap"}) != []


def test_ai_infer宣言は_prompt_を要求する():
    assert _lint({"as": "ai-infer", "prompt": "経験レベルを判定する"}) == []
    assert _lint({"as": "ai-infer"}) != []


def test_未知の種別は拒否される():
    assert _lint({"as": "foobar"}) != []


@pytest.mark.parametrize("schema_ref", ["MigrationMetaSchema/v1"])
def test_schema自体がロードできる(schema_ref):
    schema = PackageSchemaRepository().load(schema_ref)
    assert "$defs" in schema
    assert "MigrationDeclaration" in schema["$defs"]
