"""scaffold.feature のステップバインディング（受け入れレベル）。

共通ステップ（成功する / エラーコード…）は query_steps.py を behave が共有ロードする。
"""
from pathlib import Path

from behave import given, then, when

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.scaffold_engine import ScaffoldEngine


def _disc(s: str) -> dict:
    k, _, v = s.partition("=")
    return {k.strip(): v.strip()}


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


@then('ファイル "{path}" が存在する')
def step_file_exists(context, path):
    assert Path(path).exists(), f"{path} が無い"


@then('fillTemplate に path "{p}" のエントリがあり prompt が付く')
def step_filltemplate_has(context, p):
    entries = context.result.value["fillTemplate"]
    assert any(e["path"] == p and e.get("prompt") for e in entries), [e["path"] for e in entries]
