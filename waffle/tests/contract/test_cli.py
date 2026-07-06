"""waffle CLI（inbound adapter）の契約テスト（ネイティブpytest・CliRunner経由）。

engineの振る舞いはtests/acceptance・tests/integrationが担保する。ここは「引数成型・
出力JSON整形・終了コード」というCLI自体の公開インターフェース契約だけを固定する
（旧features/cli.featureから移行。test-standardのtestTypes.contractはtool=pytestを
宣言しており、behaveとの不一致を解消した）。
"""
import json
from pathlib import Path

from typer.testing import CliRunner

from waffle.adapters.inbound.cli.main import app

_runner = CliRunner()
_SCAFFOLD_DEMO_PATH = ".waffle/documents/skills/scaffold-demo.json"


def teardown_function():
    Path(_SCAFFOLD_DEMO_PATH).unlink(missing_ok=True)


def test_queryはブロックを取得しvalueをJSONで返す():
    """
    Given waffle CLI
    When query --operation get_block --path ... --blockKey interface を実行する
    Then 終了コードは0で、出力JSONのvalue.blockTypeはInterface
    """
    result = _runner.invoke(app, [
        "query", "--operation", "get_block",
        "--path", ".waffle/documents/skills/harness-query-engine.json",
        "--blockKey", "interface",
    ])
    assert result.exit_code == 0, result.output
    data = json.loads(result.output)
    assert data["value"]["blockType"] == "Interface"


def test_queryのエラーはerror_messageと非ゼロ終了で返す():
    """
    Given waffle CLI
    When 未知のoperationを実行する
    Then 終了コードは1で、出力JSONのerrorはINVALID_OPERATION
    """
    result = _runner.invoke(app, [
        "query", "--operation", "bogus",
        "--path", ".waffle/documents/skills/harness-query-engine.json",
    ])
    assert result.exit_code == 1, result.output
    data = json.loads(result.output)
    assert data["error"] == "INVALID_OPERATION"


def test_render_no_deployはmdフォーマットを返す():
    """
    Given waffle CLI
    When render --no-deploy を実行する
    Then 終了コードは0で、出力JSONのformatはmd
    """
    result = _runner.invoke(app, [
        "render", "--path", ".waffle/documents/skills/harness-query-engine.json", "--no-deploy",
    ])
    assert result.exit_code == 0, result.output
    data = json.loads(result.output)
    assert data["format"] == "md"


def test_validateは適合でstatus判定を返す():
    """
    Given waffle CLI
    When validateを実行する
    Then 終了コードは0で、出力JSONのstatusはDRAFT
    """
    result = _runner.invoke(app, [
        "validate", "--path", ".waffle/documents/skills/harness-query-engine.json",
    ])
    assert result.exit_code == 0, result.output
    data = json.loads(result.output)
    assert data["status"] == "DRAFT"


def test_scaffold_createは骨格を返す():
    """
    Given waffle CLI
    When scaffold --operation create を実行する
    Then 終了コードは0で、出力JSONのskeleton.documentTypeはSkill
    """
    result = _runner.invoke(app, [
        "scaffold", "--operation", "create",
        "--schemaRef", "SkillSchema/v1", "--documentId", "scaffold-demo",
        "--discriminator", "skillKind=engine",
    ])
    assert result.exit_code == 0, result.output
    data = json.loads(result.output)
    assert data["skeleton"]["documentType"] == "Skill"


def test_check_spec_integrityは6フィールドの差分結果を返す():
    """
    Given waffle CLI
    When check-spec-integrity --path bc-waffle-engines.json を実行する
    Then 終了コードは0で、出力JSONは6フィールド全て空配列（自己整合済み）
    """
    result = _runner.invoke(app, [
        "check-spec-integrity",
        "--path", ".waffle/documents/specs/bc-waffle-engines/bc-waffle-engines.json",
    ])
    assert result.exit_code == 0, result.output
    data = json.loads(result.output)
    assert data == {
        "declared_subdomains_missing_on_disk": [],
        "subdomains_on_disk_not_declared_in_bc": [],
        "usecases_orphaned_no_subdomain": [],
        "usecases_in_subdomain_not_declared_in_bc": [],
        "usecase_files_missing_on_disk": [],
        "usecase_files_orphaned_on_disk": [],
    }
