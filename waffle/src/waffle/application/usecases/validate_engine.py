"""validate engine — document を schema 適合検証する application use case。

driven port（DocumentRepository / SchemaRepository / Validator）を編成する。
inbound（driving）側のエントリは run()。

@spec uc-validate-document
"""
from __future__ import annotations

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.schema_repository import SchemaRepository
from waffle.application.ports.validator import Validator
from waffle.application.services.document_loading import load_document, load_schema, require_schema_ref
from waffle.domain.services.lifecycle_guard import next_status
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
        loaded = load_document(self._documents, document_path)
        if isinstance(loaded, Err):
            return loaded
        document = loaded.value

        schema_ref_result = require_schema_ref(document)
        if isinstance(schema_ref_result, Err):
            return schema_ref_result
        schema_ref = schema_ref_result.value

        schema_result = load_schema(self._schemas, schema_ref)
        if isinstance(schema_result, Err):
            return schema_result
        schema = schema_result.value

        # 検証は JSON Schema 適合のみ（x-render の RenderMetaSchema lint は別責務）。
        errors = self._validator.validate(document, schema)
        if errors:
            # 不適合時は details に違反詳細（list[str]）を載せる（コードではなく違反内容が成果物）
            return Err(f"{document_path} は {schema_ref} に不適合", errors)

        # status 遷移は schema の x-lifecycle（宣言的）を読むだけの薄い guard で守る。
        # schema がこの document 種別で "validate" を状態遷移コマンドと定義していない場合
        # （例: CodingSchema/SkillSchema の maturityLifecycle には無い）、status は変更しない。
        lifecycle = schema.get("x-lifecycle")
        defines_validate = lifecycle is not None and any(
            t["command"] == "validate" for t in lifecycle["transitions"]
        )
        if not defines_validate:
            return Ok({"path": document_path, "schemaRef": schema_ref, "status": document.get("status")})

        # status 自体は書き換えない（判定のみ・冪等）。next_status は「進んでよい状態」の判定値。
        current_status = document.get("status")
        target_status = next_status(schema, current_status, "validate")
        if target_status is None:
            return _err(
                "INVALID_TRANSITION",
                f"status '{current_status}' から validate では遷移できません",
            )
        return Ok({"path": document_path, "schemaRef": schema_ref, "status": target_status})
