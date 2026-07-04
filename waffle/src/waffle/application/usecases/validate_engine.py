"""validate engine — document を schema 適合検証する application use case。

driven port（DocumentRepository / SchemaRepository / Validator）を編成する。
inbound（driving）側のエントリは run()。
"""
from __future__ import annotations

import json
from pathlib import Path

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.schema_repository import SchemaRepository
from waffle.application.ports.validator import Validator
from waffle.shared.result import Err, Ok, Result

def _err(code: str, message: str) -> Err:
    return Err(message, [code])

class ValidateEngine:
    def __init__(
        self,
        documents: DocumentRepository,
        schemas: SchemaRepository,
        validator: Validator,
    ) -> None:
        self._documents = documents
        self._schemas = schemas
        self._validator = validator

    def run(self, document_path: str) -> Result[dict]:
        # G6: パストラバーサル拒否
        if ".." in Path(document_path).parts:
            return _err("INVALID_PATH", f"パストラバーサルは許可されません: {document_path}")
        try:
            document = self._documents.load(document_path)
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ファイルが見つかりません: {document_path}")
        except json.JSONDecodeError:
            return _err("INVALID_JSON", f"JSON として解釈できません: {document_path}")

        schema_ref = document.get("schemaRef")
        if not schema_ref:
            return _err("MISSING_SCHEMA_REF", "document に schemaRef がありません")
        try:
            schema = self._schemas.load(schema_ref)
        except (FileNotFoundError, ModuleNotFoundError):
            return _err("INVALID_SCHEMA_REF", f"schema を解決できません: {schema_ref}")

        # 検証は JSON Schema 適合のみ（x-render の RenderMetaSchema lint は別責務）。
        errors = self._validator.validate(document, schema)
        if errors:
            # 不適合時は details に違反詳細（list[str]）を載せる（コードではなく違反内容が成果物）
            return Err(f"{document_path} は {schema_ref} に不適合", errors)
        return Ok({"path": document_path, "schemaRef": schema_ref, "status": "VALIDATED"})
