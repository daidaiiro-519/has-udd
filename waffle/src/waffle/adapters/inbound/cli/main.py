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
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.query_engine import QueryEngine
from waffle.application.usecases.render_engine import RenderEngine
from waffle.application.usecases.scaffold_engine import ScaffoldEngine
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
    path: str = typer.Option(None, "--path", help="fill 対象の documentPath"),
    values: str = typer.Option(None, "--values", help="fill する値の JSON オブジェクト"),
) -> None:
    """document.json の骨格生成 / 値書き込み（uc-scaffold-document）。"""
    if operation == "create":
        params: dict = {"schemaRef": schema_ref, "documentId": document_id}
        if discriminator:
            k, _, v = discriminator.partition("=")
            params["discriminator"] = {k: v}
    elif operation == "fill":
        params = {"documentPath": path, "values": json.loads(values) if values else {}}
    else:
        params = {}
    _emit(ScaffoldEngine(_docs(), _schemas()).run(operation, params))

@app.command()
def serve() -> None:
    """MCP サーバを起動（query_document / render_document / … を MCP ツールとして公開）。"""
    from waffle.adapters.inbound.mcp.main import mcp

    mcp.run()

if __name__ == "__main__":
    app()
