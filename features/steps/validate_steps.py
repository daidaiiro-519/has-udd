"""validate.feature のステップバインディング（受け入れレベル）。

共通ステップ（対象は / 成功する / エラーコード… / schemaRef なし一時ファイル /
不正な JSON の一時ファイル）は query_steps.py・render_steps.py 側を behave が共有ロードする。
"""
import json
import tempfile

from behave import given, then, when

from has_udd.adapters.outbound.fs import FsDocumentRepository
from has_udd.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from has_udd.adapters.outbound.schema_repo import PackageSchemaRepository
from has_udd.application.usecases.validate_engine import ValidateEngine
from has_udd.shared.result import Err


@given("validate engine")
def step_validate_engine(context):
    context.engine = ValidateEngine(
        FsDocumentRepository(), PackageSchemaRepository(), JsonSchemaValidator()
    )


@given("不適合な document の一時ファイルを対象にする")
def step_nonconformant(context):
    # SkillSchema を名乗るが必須フィールド欠落 → 違反多数
    f = tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8")
    f.write(json.dumps({"schemaRef": "SkillSchema/v1", "documentId": "bad"}))
    f.close()
    context.path = f.name


@when("検証する")
def step_validate(context):
    context.result = context.engine.run(context.path)


@then('status は "{s}"')
def step_status(context, s):
    assert context.result.value["status"] == s, context.result


@then("違反詳細つきで失敗する")
def step_violations(context):
    r = context.result
    assert isinstance(r, Err), f"Err を期待: {r}"
    assert r.details, "違反詳細（details）が空"
