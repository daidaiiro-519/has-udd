"""render.feature のステップバインディング（受け入れレベル・engine を直接叩く）。

共通ステップ（対象は / 成功する / エラーコード… / schemaRef なし一時ファイル）は
query_steps.py 側を behave が共有ロードする。ここは render 固有のみ。
"""
import tempfile

from behave import given, then, when

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.render_engine import RenderEngine


@given("render engine")
def step_render_engine(context):
    context.engine = RenderEngine(FsDocumentRepository(), PackageSchemaRepository())


@given("不正な JSON の一時ファイルを対象にする")
def step_bad_json(context):
    f = tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8")
    f.write("{ this is not valid json ")
    f.close()
    context.path = f.name


@when("deploy なしでレンダリングする")
def step_render_nodeploy(context):
    context.result = context.engine.run(context.path, deploy=False)


@then('出力フォーマットは "{fmt}"')
def step_output_format(context, fmt):
    assert context.result.value["format"] == fmt, context.result


@then('出力に "{s}" を含む')
def step_output_contains(context, s):
    assert s in context.result.value["content"], f"出力に {s!r} が無い"


@then('feature出力に "{s}" を含む')
def step_feature_contains(context, s):
    feature = context.result.value.get("feature") or ""
    assert s in feature, f"feature 出力に {s!r} が無い"
