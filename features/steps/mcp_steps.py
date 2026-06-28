"""mcp.feature のステップバインディング（fastmcp の in-memory Client でツールを呼ぶ）。"""
import asyncio

from behave import given, then, when
from fastmcp import Client

from has_udd.adapters.inbound.mcp.main import mcp


def _coerce(v: str):
    if v == "true":
        return True
    if v == "false":
        return False
    if v.lstrip("-").isdigit():
        return int(v)
    return v


def _parse(kv: str) -> dict:
    out: dict = {}
    for pair in kv.split(";"):
        if pair.strip():
            k, _, v = pair.partition("=")
            out[k.strip()] = _coerce(v.strip())
    return out


def _resolve(value, vpath: str):
    cur = value
    for part in vpath.split("."):
        cur = cur[part]
    return cur


async def _call(tool: str, args: dict):
    async with Client(mcp) as client:
        result = await client.call_tool(tool, args)
        return result.data


@given("has-udd MCP サーバ")
def step_server(context):
    context.server = mcp


@when('MCP ツール "{tool}" を引数 "{kv}" で呼ぶ')
def step_call(context, tool, kv):
    context.mcp_out = asyncio.run(_call(tool, _parse(kv)))


@then('MCP出力の "{vpath}" は "{val}"')
def step_out(context, vpath, val):
    got = _resolve(context.mcp_out, vpath)
    assert str(got) == val, f"{vpath}: {got!r} != {val!r}"
