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
    Then MCP出力のvalue.blockTypeはResponseTypes
    """
    out = asyncio.run(_call("query_document", {
        "operation": "get_block",
        "path": ".waffle/documents/skills/tech-lead-advisor.json",
        "blockKey": "responseTypes",
    }))
    assert out["value"]["blockType"] == "ResponseTypes"


def test_query_documentのエラーはerror_messageを返す():
    """
    Given waffle MCPサーバ
    When query_documentツールを未知のoperationで呼ぶ
    Then MCP出力のerrorはINVALID_OPERATION
    """
    out = asyncio.run(_call("query_document", {
        "operation": "bogus",
        "path": ".waffle/documents/skills/tech-lead-advisor.json",
    }))
    assert out["error"] == "INVALID_OPERATION"


def test_validate_documentは適合でstatus判定を返す():
    """
    Given waffle MCPサーバ
    When validate_documentツールを呼ぶ
    Then MCP出力のstatusはDRAFT
    """
    out = asyncio.run(_call("validate_document", {
        "path": ".waffle/documents/skills/tech-lead-advisor.json",
    }))
    assert out["status"] == "DRAFT"


def test_render_documentはmdフォーマットを返す():
    """
    Given waffle MCPサーバ
    When render_documentツールをdeploy=falseで呼ぶ
    Then MCP出力のformatはmd
    """
    out = asyncio.run(_call("render_document", {
        "path": ".waffle/documents/skills/tech-lead-advisor.json",
        "deploy": False,
    }))
    assert out["format"] == "md"


def test_check_spec_integrityは10フィールドの差分結果を返す():
    """
    Given waffle MCPサーバ
    When check_spec_integrityツールをbc-waffle.jsonで呼ぶ
    Then MCP出力は10フィールド全て空配列（自己整合済み）
    """
    out = asyncio.run(_call("check_spec_integrity", {
        "path": ".waffle/documents/specs/bc-waffle/bc-waffle.json",
    }))
    assert out == {
        "declared_subdomains_missing_on_disk": [],
        "subdomains_on_disk_not_declared_in_bc": [],
        "usecases_orphaned_no_subdomain": [],
        "usecases_in_subdomain_not_declared_in_bc": [],
        "usecase_files_missing_on_disk": [],
        "usecase_files_orphaned_on_disk": [],
        "orphaned_value_objects": [],
        "undeclared_document_fields": [],
        "subdomain_ref_mismatches": [],
        "missing_aggregate_refs": [],
    }


def test_check_schema_version_driftは3フィールドの差分結果を返す():
    """
    Given waffle MCPサーバ
    When check_schema_version_driftツールを呼ぶ
    Then MCP出力は3フィールド全て空配列（自己整合済み）
    """
    out = asyncio.run(_call("check_schema_version_drift", {}))
    assert out == {"broken_references": [], "newer_version_available": [], "missing_declared_fields": []}


def test_check_agent_skill_driftは2フィールドの差分結果を返す():
    """
    Given waffle MCPサーバ
    When check_agent_skill_driftツールを呼ぶ
    Then MCP出力は2フィールド全て空配列（自己整合済み）
    """
    out = asyncio.run(_call("check_agent_skill_drift", {}))
    assert out == {"missing_skills": [], "unpreloadable_skills": []}


def test_check_scenario_driftは4フィールドの差分結果を返す():
    """
    Given waffle MCPサーバ
    When check_scenario_driftツールを呼ぶ
    Then MCP出力は4フィールドを持つ
    """
    out = asyncio.run(_call("check_scenario_drift", {
        "specPath": ".waffle/documents/specs/bc-waffle/subdomain/sd-reconciliation/usecase/uc-check-spec-integrity.json",
        "testPath": "tests/integration/test_uc_check_spec_integrity.py",
    }))
    assert set(out.keys()) == {"missing_in_tests", "orphaned_in_tests", "matched", "gherkin_mismatches"}


def test_scan_source_codeは公開要素の一覧を返す(tmp_path):
    """
    Given waffle MCPサーバ
    When scan_source_codeツールをkind=googleで呼ぶ
    Then MCP出力は要素の配列
    """
    sample = tmp_path / "sample.py"
    sample.write_text('def f():\n    """要約。"""\n    pass\n', encoding="utf-8")

    out = asyncio.run(_call("scan_source_code", {"path": str(tmp_path), "kind": "google"}))
    assert any(e["name"] == "f" for e in out)


def test_lint_docstringは違反の配列を返す(tmp_path):
    """
    Given waffle MCPサーバ
    When lint_docstringツールをkind=googleで呼ぶ
    Then MCP出力は違反の配列（適合すれば空配列）
    """
    sample = tmp_path / "sample.py"
    sample.write_text('def f():\n    """要約。"""\n    pass\n', encoding="utf-8")

    out = asyncio.run(_call("lint_docstring", {"path": str(tmp_path), "kind": "google"}))
    assert isinstance(out, list)
