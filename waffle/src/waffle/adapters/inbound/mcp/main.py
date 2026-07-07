"""waffle MCP サーバ — inbound(driving) adapter（fastmcp）。

CLI と並ぶもう1つの front-door。各 engine(application use case) に outbound adapter を結線し、
MCP ツールとして公開する。返り値は dict（Ok→value / Err→{error, message}）。
engine Skill の InvocationSpec が指す MCP ツール（query_document 等）の実体。
"""
from __future__ import annotations

from fastmcp import FastMCP

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.pydoclint_linter import PydoclintLinter
from waffle.adapters.outbound.python_ast_source_scanner import PythonAstSourceScanner
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.check_scenario_drift_engine import CheckScenarioDriftEngine
from waffle.application.usecases.check_agent_skill_drift_engine import CheckAgentSkillDriftEngine
from waffle.application.usecases.check_error_code_drift_engine import CheckErrorCodeDriftEngine
from waffle.application.usecases.check_schema_version_drift_engine import CheckSchemaVersionDriftEngine
from waffle.application.usecases.check_spec_integrity_engine import CheckSpecIntegrityEngine
from waffle.application.usecases.lint_docstring_engine import LintDocstringEngine
from waffle.application.usecases.query_engine import QueryEngine
from waffle.application.usecases.render_engine import RenderEngine
from waffle.application.usecases.scaffold_engine import ScaffoldEngine
from waffle.application.usecases.scan_source_code_engine import ScanSourceCodeEngine
from waffle.application.usecases.validate_engine import ValidateEngine
from waffle.shared.result import Ok, Result

mcp = FastMCP("waffle")

def _dict(result: Result) -> dict:
    if isinstance(result, Ok):
        return result.value
    return {"error": result.details[0] if result.details else "ERROR", "message": result.message}

def _docs() -> FsDocumentRepository:
    return FsDocumentRepository()

def _schemas() -> PackageSchemaRepository:
    return PackageSchemaRepository()

@mcp.tool
def query_document(
    operation: str,
    path: str,
    blockKey: str | None = None,
    arrayField: str | None = None,
    field: str | None = None,
    idField: str | None = None,
    idValue: str | None = None,
    key: str | None = None,
    value: str | None = None,
    pattern: str | None = None,
    start: int | None = None,
    end: int | None = None,
    fieldName: str | None = None,
    nestedField: str | None = None,
) -> dict:
    """document.json へのセマンティック・クエリ（uc-query-document）。"""
    raw = {
        "blockKey": blockKey, "arrayField": arrayField, "field": field,
        "idField": idField, "idValue": idValue, "key": key, "value": value,
        "pattern": pattern, "start": start, "end": end,
        "fieldName": fieldName, "nestedField": nestedField,
    }
    params = {k: v for k, v in raw.items() if v is not None}
    return _dict(QueryEngine(_docs(), _schemas()).run(operation, path, params))

@mcp.tool
def render_document(path: str, deploy: bool = True) -> dict:
    """document.json を成果物にレンダリングして deploy（uc-render-document）。"""
    return _dict(RenderEngine(_docs(), _schemas()).run(path, deploy=deploy))

@mcp.tool
def validate_document(path: str) -> dict:
    """document を schema 適合検証（uc-validate-document）。"""
    return _dict(ValidateEngine(_docs(), _schemas(), JsonSchemaValidator()).run(path))

@mcp.tool
def scaffold_document(
    operation: str,
    schemaRef: str | None = None,
    documentId: str | None = None,
    discriminator: dict | None = None,
    contextRef: str | None = None,
    subdomainRef: str | None = None,
    documentPath: str | None = None,
    values: dict | None = None,
) -> dict:
    """document.json の骨格生成 / 値書き込み（uc-scaffold-document）。"""
    if operation == "create":
        params: dict = {"schemaRef": schemaRef, "documentId": documentId}
        if discriminator:
            params["discriminator"] = discriminator
        if contextRef:
            params["contextRef"] = contextRef
        if subdomainRef:
            params["subdomainRef"] = subdomainRef
    elif operation == "fill":
        params = {"documentPath": documentPath, "values": values or {}}
    else:
        params = {}
    return _dict(ScaffoldEngine(_docs(), _schemas()).run(operation, params))

@mcp.tool
def check_spec_integrity(path: str, documentsRoot: str = ".waffle/documents") -> dict:
    """bc.jsonのmembers宣言とディスク上の実ファイルの参照整合性を検証（uc-check-spec-integrity）。"""
    return _dict(CheckSpecIntegrityEngine(_docs()).run(path, documentsRoot))

@mcp.tool
def check_scenario_drift(specPath: str, testPath: str) -> dict:
    """specのシナリオとテストコードの対応関係を検証（uc-check-scenario-drift）。"""
    return _dict(CheckScenarioDriftEngine(_docs()).run(specPath, testPath))

@mcp.tool
def check_schema_version_drift(documentsRoot: str = ".waffle/documents") -> dict:
    """DocumentのschemaRefが実在し最新であるかを検証（uc-check-schema-version-drift）。"""
    return _dict(CheckSchemaVersionDriftEngine(_docs(), _schemas()).run(documentsRoot))

@mcp.tool
def check_agent_skill_drift(documentsRoot: str = ".waffle/documents") -> dict:
    """subagentのskillPreloadsが参照するSkillの実在性・プリロード可能性を検証（uc-check-agent-skill-drift）。"""
    return _dict(CheckAgentSkillDriftEngine(_docs()).run(documentsRoot))

@mcp.tool
def check_error_code_drift(specsRoot: str, codeRoot: str) -> dict:
    """specのErrorsが宣言するコードが@specタグでリンクされた実装に実在するかを検証（uc-check-error-code-drift）。"""
    return _dict(CheckErrorCodeDriftEngine(_docs()).run(specsRoot, codeRoot))

@mcp.tool
def scan_source_code(path: str, kind: str) -> dict | list:
    """対象コードベースの公開要素のdocstringを構造化抽出（uc-scan-source-code）。"""
    return _dict(ScanSourceCodeEngine(_docs(), PythonAstSourceScanner()).run(path, kind))

@mcp.tool
def lint_docstring(path: str, kind: str) -> dict | list:
    """対象コードベースのdocstringが規約どおりか既存lintツールで検証（uc-lint-docstring）。"""
    scan_engine = ScanSourceCodeEngine(_docs(), PythonAstSourceScanner())
    return _dict(LintDocstringEngine(scan_engine, PydoclintLinter()).run(path, kind))
