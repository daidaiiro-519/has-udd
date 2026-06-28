"""scaffold.feature のステップバインディング（受け入れレベル）。

共通ステップ（成功する / エラーコード…）は query_steps.py を behave が共有ロードする。
"""
from pathlib import Path

from behave import given, then, when

from has_udd.adapters.outbound.fs import FsDocumentRepository
from has_udd.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from has_udd.adapters.outbound.schema_repo import PackageSchemaRepository
from has_udd.application.usecases.scaffold_engine import ScaffoldEngine
from has_udd.application.usecases.validate_engine import ValidateEngine
from has_udd.shared.result import Ok

_SK = "SkillSchema/v1"


def _disc(s: str) -> dict:
    k, _, v = s.partition("=")
    return {k.strip(): v.strip()}


def _resolve(value, vpath: str):
    cur = value
    for part in vpath.split("."):
        cur = cur[part]
    return cur


@given("scaffold engine")
def step_scaffold_engine(context):
    context.engine = ScaffoldEngine(FsDocumentRepository(), PackageSchemaRepository())


@when('create を schemaRef "{ref}" documentId "{did}" discriminator "{disc}" で実行する')
def step_create_disc(context, ref, did, disc):
    context.result = context.engine.run(
        "create", {"schemaRef": ref, "documentId": did, "discriminator": _disc(disc)}
    )


@when('create を schemaRef "{ref}" documentId "{did}" で実行する')
def step_create_nodisc(context, ref, did):
    context.result = context.engine.run("create", {"schemaRef": ref, "documentId": did})


@then('skeleton の "{key}" は "{val}"')
def step_skeleton_field(context, key, val):
    assert str(context.result.value["skeleton"][key]) == val, context.result.value["skeleton"][key]


@then('skeleton の content に "{key}" がある')
def step_skeleton_content_has(context, key):
    assert key in context.result.value["skeleton"]["content"]


@then("生成された骨格は validate を通る")
def step_skeleton_validates(context):
    ve = ValidateEngine(FsDocumentRepository(), PackageSchemaRepository(), JsonSchemaValidator())
    r = ve.run(context.result.value["path"])
    assert isinstance(r, Ok), f"骨格が不適合: {getattr(r, 'details', None)}"


@then('ファイル "{path}" が存在する')
def step_file_exists(context, path):
    assert Path(path).exists(), f"{path} が無い"


@then('fillTemplate に path "{p}" のエントリがあり prompt が付く')
def step_filltemplate_has(context, p):
    entries = context.result.value["fillTemplate"]
    assert any(e["path"] == p and e.get("prompt") for e in entries), [e["path"] for e in entries]


@given('"{did}" の engine 骨格を作成済み')
def step_created(context, did):
    r = context.engine.run("create", {"schemaRef": _SK, "documentId": did, "discriminator": {"skillKind": "engine"}})
    assert isinstance(r, Ok), r
    context.doc_path = r.value["path"]


@when('fill で値 "{kv}" を書き込む')
def step_fill(context, kv):
    path, _, value = kv.partition("=")
    context.result = context.engine.run(
        "fill", {"documentPath": context.doc_path, "values": {path.strip(): value.strip()}}
    )


@then('written に "{p}" を含む')
def step_written_has(context, p):
    assert p in context.result.value["written"], context.result.value


@then("written は空")
def step_written_empty(context):
    assert context.result.value["written"] == [], context.result.value


@then('skipped に "{p}" を含む')
def step_skipped_has(context, p):
    assert p in context.result.value["skipped"], context.result.value


@then('ファイルの "{vpath}" は "{val}"')
def step_file_field(context, vpath, val):
    doc = FsDocumentRepository().load(context.doc_path)
    assert str(_resolve(doc, vpath)) == val, _resolve(doc, vpath)
