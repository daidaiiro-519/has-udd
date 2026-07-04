"""query.feature のステップバインディング（受け入れレベル・engine を直接叩く）。

.feature が What の SSOT。ここは「シナリオの語彙 → engine 呼び出し / 検証」を繋ぐだけ。
将来 P6 で UsecaseSpec から .feature を render しても、この束縛はそのまま使える。
"""
import json
import tempfile

from behave import given, then, when

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.query_engine import QueryEngine
from waffle.shared.result import Err, Ok


def _parse_params(kv: str) -> dict:
    out: dict = {}
    for pair in kv.split(";"):
        if pair.strip():
            k, _, v = pair.partition("=")
            out[k.strip()] = v.strip()
    return out


def _resolve(value, vpath: str):
    cur = value
    for part in vpath.split("."):
        cur = cur[part]
    return cur


@given("query engine")
def step_engine(context):
    context.engine = QueryEngine(FsDocumentRepository(), PackageSchemaRepository())


@given('対象は "{path}"')
def step_target(context, path):
    context.path = path


@given("schemaRef なしの一時ファイルを対象にする")
def step_tmp(context):
    f = tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8")
    f.write(json.dumps({"hello": "world"}))
    f.close()
    context.path = f.name


@when('operation "{op}" を実行する')
def step_run(context, op):
    context.result = context.engine.run(op, context.path)


@when('operation "{op}" を params "{kv}" で実行する')
def step_run_params(context, op, kv):
    context.result = context.engine.run(op, context.path, _parse_params(kv))


@then("成功する")
def step_ok(context):
    assert isinstance(context.result, Ok), f"Ok を期待: {context.result}"


@then('エラーコード "{code}" で失敗する')
def step_err(context, code):
    r = context.result
    assert isinstance(r, Err), f"Err を期待: {r}"
    assert r.details and r.details[0] == code, f"コード {code} を期待: {r.details}"


@then("prompt は null")
def step_prompt_null(context):
    assert context.result.value["prompt"] is None


@then("prompt は非空")
def step_prompt_truthy(context):
    assert context.result.value["prompt"]


@then("value は空配列")
def step_value_empty(context):
    assert context.result.value["value"] == []


@then('value は "{s}" を含む')
def step_value_contains(context, s):
    v = context.result.value["value"]
    assert s in v if isinstance(v, (list, str)) else s in str(v)


@then('value の "{vpath}" は "{expected}"')
def step_value_path_eq(context, vpath, expected):
    got = _resolve(context.result.value["value"], vpath)
    assert str(got) == expected, f'{vpath}: {got!r} != {expected!r}'


@then('value の "{vpath}" は非空')
def step_value_path_truthy(context, vpath):
    assert _resolve(context.result.value["value"], vpath)


@then('value の "{field}" 集合は "{csv}"')
def step_value_set(context, field, csv):
    got = {x[field] for x in context.result.value["value"]}
    assert got == set(csv.split(",")), f'{got} != {csv}'


@then('value のキーに "{s}" を含むものがある')
def step_value_key_contains(context, s):
    assert any(s in k for k in context.result.value["value"])


@then('結果の "{vpath}" は "{expected}"')
def step_payload_path_eq(context, vpath, expected):
    got = _resolve(context.result.value, vpath)
    assert str(got) == expected, f'{vpath}: {got!r} != {expected!r}'
