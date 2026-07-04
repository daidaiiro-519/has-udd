"""cli.feature のステップバインディング（CLI を typer CliRunner で実行）。"""
import json
import shlex

from behave import given, then, when
from typer.testing import CliRunner

from waffle.adapters.inbound.cli.main import app


def _resolve(value, vpath: str):
    cur = value
    for part in vpath.split("."):
        cur = cur[part]
    return cur


@given("waffle CLI")
def step_cli(context):
    context.runner = CliRunner()


@when('CLI "{args}" を実行する')
def step_cli_run(context, args):
    context.cli = context.runner.invoke(app, shlex.split(args))


@then("終了コードは {code:d}")
def step_exit(context, code):
    assert context.cli.exit_code == code, f"exit={context.cli.exit_code} out={context.cli.output}"


@then('出力JSONの "{vpath}" は "{val}"')
def step_cli_json(context, vpath, val):
    data = json.loads(context.cli.output)
    got = _resolve(data, vpath)
    assert str(got) == val, f"{vpath}: {got!r} != {val!r}"
