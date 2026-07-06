"""waffle MCPサーバ（inbound adapter）の契約テスト（ネイティブpytest・fastmcp in-memory Client経由）。

CLIと並ぶ第2のfront-door。engineの振る舞いはtests/acceptance・tests/integrationが担保する。
ここは「MCPツール経由でengineが正しく呼ばれdictを返す」というMCP自体の公開インターフェース契約
だけを固定する（旧features/mcp.featureから移行）。
"""
import asyncio

from fastmcp import Client

from waffle.adapters.inbound.mcp.main import mcp


async def _call(tool: str, args: dict):
    async with Client(mcp) as client:
        result = await client.call_tool(tool, args)
        return result.data


def test_query_documentはブロックを取得する():
    """
    Given waffle MCPサーバ
    When query_documentツールをoperation=get_blockで呼ぶ
    Then MCP出力のvalue.blockTypeはInterface
    """
    out = asyncio.run(_call("query_document", {
        "operation": "get_block",
        "path": ".waffle/documents/skills/harness-query-engine.json",
        "blockKey": "interface",
    }))
    assert out["value"]["blockType"] == "Interface"


def test_query_documentのエラーはerror_messageを返す():
    """
    Given waffle MCPサーバ
    When query_documentツールを未知のoperationで呼ぶ
    Then MCP出力のerrorはINVALID_OPERATION
    """
    out = asyncio.run(_call("query_document", {
        "operation": "bogus",
        "path": ".waffle/documents/skills/harness-query-engine.json",
    }))
    assert out["error"] == "INVALID_OPERATION"


def test_validate_documentは適合でstatus判定を返す():
    """
    Given waffle MCPサーバ
    When validate_documentツールを呼ぶ
    Then MCP出力のstatusはDRAFT
    """
    out = asyncio.run(_call("validate_document", {
        "path": ".waffle/documents/skills/harness-query-engine.json",
    }))
    assert out["status"] == "DRAFT"


def test_render_documentはmdフォーマットを返す():
    """
    Given waffle MCPサーバ
    When render_documentツールをdeploy=falseで呼ぶ
    Then MCP出力のformatはmd
    """
    out = asyncio.run(_call("render_document", {
        "path": ".waffle/documents/skills/harness-query-engine.json",
        "deploy": False,
    }))
    assert out["format"] == "md"
