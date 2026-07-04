"""waffle MCP サーバ — inbound(driving) adapter（fastmcp）。

CLI と並ぶもう1つの front-door。各 engine(application use case) に outbound adapter を結線し、
MCP ツールとして公開する。返り値は dict（Ok→value / Err→{error, message}）。
engine Skill の InvocationSpec が指す MCP ツール（query_document 等）の実体。

@stack:mcp
"""
from __future__ import annotations

from fastmcp import FastMCP

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.query_engine import QueryEngine
from waffle.application.usecases.render_engine import RenderEngine
from waffle.application.usecases.scaffold_engine import ScaffoldEngine
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
    # waffle:impl-start
    raw = {
        "blockKey": blockKey, "arrayField": arrayField, "field": field,
        "idField": idField, "idValue": idValue, "key": key, "value": value,
        "pattern": pattern, "start": start, "end": end,
        "fieldName": fieldName, "nestedField": nestedField,
    }
    params = {k: v for k, v in raw.items() if v is not None}
    return _dict(QueryEngine(_docs(), _schemas()).run(operation, path, params))
    # waffle:impl-end


@mcp.tool
def render_document(path: str, deploy: bool = True) -> dict:
    """document.json を成果物にレンダリングして deploy（uc-render-document）。"""
    # waffle:impl-start
    return _dict(RenderEngine(_docs(), _schemas()).run(path, deploy=deploy))
    # waffle:impl-end


@mcp.tool
def validate_document(path: str) -> dict:
    """document を schema 適合検証（uc-validate-document）。"""
    # waffle:impl-start
    return _dict(ValidateEngine(_docs(), _schemas(), JsonSchemaValidator()).run(path))
    # waffle:impl-end


@mcp.tool
def scaffold_document(
    operation: str,
    schemaRef: str | None = None,
    documentId: str | None = None,
    discriminator: dict | None = None,
    documentPath: str | None = None,
    values: dict | None = None,
) -> dict:
    """document.json の骨格生成 / 値書き込み（uc-scaffold-document）。"""
    # waffle:impl-start
    if operation == "create":
        params: dict = {"schemaRef": schemaRef, "documentId": documentId}
        if discriminator:
            params["discriminator"] = discriminator
    elif operation == "fill":
        params = {"documentPath": documentPath, "values": values or {}}
    else:
        params = {}
    return _dict(ScaffoldEngine(_docs(), _schemas()).run(operation, params))
    # waffle:impl-end
