"""waffle CLI — inbound(driving) adapter。

各 engine(application use case) に outbound adapter を結線し、CLI 引数を params に成型して呼ぶ。
Result[dict] を JSON で標準出力に出す（Ok→value をそのまま / Err→{error, message}・exit!=0）。
これが engine Skill の InvocationSpec が指す実体（`waffle <op> ...`）。
"""
from __future__ import annotations

import json

import typer

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.pydoclint_linter import PydoclintLinter
from waffle.adapters.outbound.python_ast_source_scanner import PythonAstSourceScanner
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.check_agent_skill_drift_engine import CheckAgentSkillDriftEngine
from waffle.application.usecases.check_error_code_drift_engine import CheckErrorCodeDriftEngine
from waffle.application.usecases.check_scenario_drift_engine import CheckScenarioDriftEngine
from waffle.application.usecases.check_schema_version_drift_engine import CheckSchemaVersionDriftEngine
from waffle.application.usecases.check_spec_integrity_engine import CheckSpecIntegrityEngine
from waffle.application.usecases.lint_docstring_engine import LintDocstringEngine
from waffle.application.usecases.query_engine import QueryEngine
from waffle.application.usecases.render_engine import RenderEngine
from waffle.application.usecases.scaffold_engine import ScaffoldEngine
from waffle.application.usecases.scan_source_code_engine import ScanSourceCodeEngine
from waffle.application.usecases.validate_engine import ValidateEngine
from waffle.shared.result import Ok, Result

app = typer.Typer(add_completion=False, help="waffle CLI — query / render / validate / scaffold")

def _emit(result: Result) -> None:
    if isinstance(result, Ok):
        typer.echo(json.dumps(result.value, ensure_ascii=False))
        return
    code = result.details[0] if result.details else "ERROR"
    typer.echo(json.dumps({"error": code, "message": result.message}, ensure_ascii=False))
    raise typer.Exit(1)

def _docs() -> FsDocumentRepository:
    return FsDocumentRepository()

def _schemas() -> PackageSchemaRepository:
    return PackageSchemaRepository()

@app.command()
def query(
    operation: str = typer.Option(..., "--operation"),
    path: str = typer.Option(..., "--path"),
    block_key: str = typer.Option(None, "--blockKey", "--block-key"),
    array_field: str = typer.Option(None, "--arrayField", "--array-field"),
    field: str = typer.Option(None, "--field"),
    id_field: str = typer.Option(None, "--idField", "--id-field"),
    id_value: str = typer.Option(None, "--idValue", "--id-value"),
    key: str = typer.Option(None, "--key"),
    value: str = typer.Option(None, "--value"),
    pattern: str = typer.Option(None, "--pattern"),
    start: int = typer.Option(None, "--start"),
    end: int = typer.Option(None, "--end"),
    field_name: str = typer.Option(None, "--fieldName", "--field-name"),
    nested_field: str = typer.Option(None, "--nestedField", "--nested-field"),
) -> None:
    """document.json へのセマンティック・クエリ（uc-query-document）。"""
    raw = {
        "blockKey": block_key, "arrayField": array_field, "field": field,
        "idField": id_field, "idValue": id_value, "key": key, "value": value,
        "pattern": pattern, "start": start, "end": end,
        "fieldName": field_name, "nestedField": nested_field,
    }
    params = {k: v for k, v in raw.items() if v is not None}
    _emit(QueryEngine(_docs(), _schemas()).run(operation, path, params))

@app.command()
def render(
    path: str = typer.Option(..., "--path"),
    no_deploy: bool = typer.Option(False, "--no-deploy"),
) -> None:
    """document.json を成果物にレンダリングして deploy（uc-render-document）。"""
    _emit(RenderEngine(_docs(), _schemas()).run(path, deploy=not no_deploy))

@app.command()
def validate(path: str = typer.Option(..., "--path")) -> None:
    """document を schema 適合検証（uc-validate-document）。"""
    _emit(ValidateEngine(_docs(), _schemas(), JsonSchemaValidator()).run(path))

@app.command()
def scaffold(
    operation: str = typer.Option(..., "--operation"),
    schema_ref: str = typer.Option(None, "--schemaRef", "--schema-ref"),
    document_id: str = typer.Option(None, "--documentId", "--document-id"),
    discriminator: str = typer.Option(None, "--discriminator", help="key=value 形式（例: skillKind=engine）"),
    context_ref: str = typer.Option(None, "--contextRef", "--context-ref", help="所属する bounded-context の documentId（ネストしたx-source-targetが要求する場合）"),
    subdomain_ref: str = typer.Option(None, "--subdomainRef", "--subdomain-ref", help="usecase が属する subdomain の documentId"),
    path: str = typer.Option(None, "--path", help="fill 対象の documentPath"),
    values: str = typer.Option(None, "--values", help="fill する値の JSON オブジェクト"),
) -> None:
    """document.json の骨格生成 / 値書き込み（uc-scaffold-document）。"""
    if operation == "create":
        params: dict = {"schemaRef": schema_ref, "documentId": document_id}
        if discriminator:
            k, _, v = discriminator.partition("=")
            params["discriminator"] = {k: v}
        if context_ref:
            params["contextRef"] = context_ref
        if subdomain_ref:
            params["subdomainRef"] = subdomain_ref
    elif operation == "fill":
        params = {"documentPath": path, "values": json.loads(values) if values else {}}
    else:
        params = {}
    _emit(ScaffoldEngine(_docs(), _schemas()).run(operation, params))

@app.command("check-spec-integrity")
def check_spec_integrity(
    path: str = typer.Option(..., "--path", help="bounded-context の bc.json のパス"),
    documents_root: str = typer.Option(".waffle/documents", "--documentsRoot", "--documents-root", help="Document集約の実インスタンス群を走査する対象ディレクトリ"),
) -> None:
    """bc.jsonのmembers宣言とディスク上の実ファイルの参照整合性を検証（uc-check-spec-integrity）。"""
    _emit(CheckSpecIntegrityEngine(_docs()).run(path, documents_root))

@app.command("check-scenario-drift")
def check_scenario_drift(
    spec_path: str = typer.Option(..., "--specPath", "--spec-path", help="spec.json のパス"),
    test_path: str = typer.Option(..., "--testPath", "--test-path", help="対応するテストファイル(.py)のパス"),
) -> None:
    """specのシナリオとテストコードの対応関係を検証（uc-check-scenario-drift）。"""
    _emit(CheckScenarioDriftEngine(_docs()).run(spec_path, test_path))

@app.command("check-schema-version-drift")
def check_schema_version_drift(
    documents_root: str = typer.Option(".waffle/documents", "--documentsRoot", "--documents-root", help="Document集約の実インスタンス群を走査する対象ディレクトリ"),
) -> None:
    """DocumentのschemaRefが実在し最新であるかを検証（uc-check-schema-version-drift）。"""
    _emit(CheckSchemaVersionDriftEngine(_docs(), _schemas()).run(documents_root))

@app.command("check-agent-skill-drift")
def check_agent_skill_drift(
    documents_root: str = typer.Option(".waffle/documents", "--documentsRoot", "--documents-root", help="Document集約の実インスタンス群を走査する対象ディレクトリ"),
) -> None:
    """subagentのskillPreloadsが参照するSkillの実在性・プリロード可能性を検証（uc-check-agent-skill-drift）。"""
    _emit(CheckAgentSkillDriftEngine(_docs()).run(documents_root))

@app.command("check-error-code-drift")
def check_error_code_drift(
    specs_root: str = typer.Option(..., "--specsRoot", "--specs-root", help="usecase spec群を走査する対象ディレクトリ"),
    code_root: str = typer.Option(..., "--codeRoot", "--code-root", help="実装コード群を走査する対象ディレクトリ"),
) -> None:
    """specのErrorsが宣言するコードが@specタグでリンクされた実装に実在するかを検証（uc-check-error-code-drift）。"""
    _emit(CheckErrorCodeDriftEngine(_docs()).run(specs_root, code_root))

@app.command("scan-source-code")
def scan_source_code(
    path: str = typer.Option(..., "--path", help="対象コードベース(ディレクトリ)のパス"),
    kind: str = typer.Option(..., "--kind", help="DocstringSchemaのkind（現状はgoogleのみ対応）"),
) -> None:
    """対象コードベースの公開要素のdocstringを構造化抽出（uc-scan-source-code）。"""
    _emit(ScanSourceCodeEngine(_docs(), PythonAstSourceScanner()).run(path, kind))

@app.command("lint-docstring")
def lint_docstring(
    path: str = typer.Option(..., "--path", help="対象コードベース(ディレクトリ)のパス"),
    kind: str = typer.Option(..., "--kind", help="DocstringSchemaのkind（現状はgoogleのみ対応）"),
) -> None:
    """対象コードベースのdocstringが規約どおりか既存lintツールで検証（uc-lint-docstring）。"""
    scan_engine = ScanSourceCodeEngine(_docs(), PythonAstSourceScanner())
    _emit(LintDocstringEngine(scan_engine, PydoclintLinter()).run(path, kind))

@app.command()
def serve() -> None:
    """MCP サーバを起動（query_document / render_document / … を MCP ツールとして公開）。"""
    from waffle.adapters.inbound.mcp.main import mcp

    mcp.run()

if __name__ == "__main__":
    app()
