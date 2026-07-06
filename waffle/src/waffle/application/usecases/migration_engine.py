"""migration engine — Schema集約のバージョニング/移行(publishVersion/deprecateVersion/migrateDocuments)。

Harness原則をSchema自身の進化にも適用する: AIは「x-migrationのai-infer宣言が要求する値」だけを
生成し、機械的な変換(rename/default/value-map/discriminator-remap)・検証はengineが担う。
migrateDocumentsはscaffoldのcreate/fillと同じ2段階に分ける:
- prepareMigration: 機械変換を適用し、ai-infer宣言のあるフィールドだけをワークシートとして返す
- applyMigration: AIが埋めたワークシートの値をマージし、論点2の実証的検証(新schemaでvalidate)を
  経て書き込む
"""
from __future__ import annotations

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.schema_repository import SchemaRepository
from waffle.application.ports.validator import Validator
from waffle.application.services.document_loading import load_document, load_schema
from waffle.domain.services.schema_versioning import is_forward_migration
from waffle.shared.result import Err, Ok, Result

def _err(code: str, message: str) -> Err:
    return Err(message, [code])

class MigrationEngine:
    def __init__(
        self,
        documents: DocumentRepository,
        schemas: SchemaRepository,
        validator: Validator,
    ) -> None:
        self._documents = documents
        self._schemas = schemas
        self._validator = validator

    def run(self, operation: str, params: dict | None = None) -> Result[dict]:
        params = params or {}
        if operation == "publishVersion":
            return self._publish_version(params)
        if operation == "deprecateVersion":
            return self._deprecate_version(params)
        if operation == "prepareMigration":
            return self._prepare_migration(params)
        if operation == "applyMigration":
            return self._apply_migration(params)
        return _err("INVALID_OPERATION", f"未知の operation: {operation}")

    def _load_schema_file(self, schema_path: str) -> Result[dict]:
        return load_document(self._documents, schema_path)

    def _publish_version(self, params: dict) -> Result[dict]:
        schema_path = params.get("schemaPath")
        if not schema_path:
            return _err("MISSING_PARAM", "publishVersion には schemaPath が必要です")
        loaded = self._load_schema_file(schema_path)
        if isinstance(loaded, Err):
            return loaded
        schema = loaded.value
        current = schema.get("x-schema-status")
        if current is not None:
            return _err("ALREADY_PUBLISHED", f"既に x-schema-status={current} です（requiresState=null は未公開のみ）")
        schema["x-schema-status"] = "PUBLISHED"
        self._documents.save(schema_path, schema)
        return Ok({"schemaPath": schema_path, "status": "PUBLISHED"})

    def _deprecate_version(self, params: dict) -> Result[dict]:
        schema_path = params.get("schemaPath")
        if not schema_path:
            return _err("MISSING_PARAM", "deprecateVersion には schemaPath が必要です")
        loaded = self._load_schema_file(schema_path)
        if isinstance(loaded, Err):
            return loaded
        schema = loaded.value
        current = schema.get("x-schema-status")
        if current != "PUBLISHED":
            return _err("INVALID_STATE", f"deprecateVersion は PUBLISHED からのみ実行可能です(現在: {current})")
        schema["x-schema-status"] = "DEPRECATED"
        self._documents.save(schema_path, schema)
        return Ok({"schemaPath": schema_path, "status": "DEPRECATED"})

    def _prepare_migration(self, params: dict) -> Result[dict]:
        from_ref = params.get("fromSchemaRef")
        to_ref = params.get("toSchemaRef")
        documents_dir = params.get("documentsDir")
        if not from_ref or not to_ref or not documents_dir:
            return _err("MISSING_PARAM", "prepareMigration には fromSchemaRef, toSchemaRef, documentsDir が必要です")
        if not is_forward_migration(from_ref, to_ref):
            return _err("INVALID_MIGRATION_DIRECTION", f"移行は版を上げる方向にのみ行えます: {from_ref} -> {to_ref}")
        schema_result = load_schema(self._schemas, to_ref)
        if isinstance(schema_result, Err):
            return schema_result
        to_schema = schema_result.value

        migrations = _collect_migrations(to_schema)
        mechanical = {f: m for f, m in migrations.items() if m["as"] != "ai-infer"}
        ai_infer = {f: m for f, m in migrations.items() if m["as"] == "ai-infer"}

        try:
            doc_paths = self._documents.list_json(documents_dir)
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ディレクトリが見つかりません: {documents_dir}")

        partial_documents: dict = {}
        worksheets: dict = {}
        for doc_path in doc_paths:
            doc = self._documents.load(doc_path)
            if doc.get("schemaRef") != from_ref:
                continue
            partial = dict(doc)
            for field, mig in mechanical.items():
                partial[field] = _apply_mechanical(mig, doc)
            partial["schemaRef"] = to_ref
            partial_documents[doc_path] = partial
            if ai_infer:
                worksheets[doc_path] = {
                    field: {"prompt": mig["prompt"], "context": partial} for field, mig in ai_infer.items()
                }

        return Ok({"partialDocuments": partial_documents, "worksheets": worksheets})

    def _apply_migration(self, params: dict) -> Result[dict]:
        to_ref = params.get("toSchemaRef")
        partial_documents = params.get("partialDocuments")
        answers = params.get("answers") or {}
        if not to_ref or partial_documents is None:
            return _err("MISSING_PARAM", "applyMigration には toSchemaRef, partialDocuments が必要です")
        schema_result = load_schema(self._schemas, to_ref)
        if isinstance(schema_result, Err):
            return schema_result
        to_schema = schema_result.value

        migrated: list[str] = []
        rejected: list[dict] = []
        for doc_path, partial in partial_documents.items():
            merged = {**partial, **answers.get(doc_path, {})}
            errors = self._validator.validate(merged, to_schema)
            if errors:
                rejected.append({"documentPath": doc_path, "errors": errors})
                continue
            self._documents.save(doc_path, merged)
            migrated.append(doc_path)
        return Ok({"migrated": migrated, "rejected": rejected})

# --- x-migration 走査ヘルパ（純ロジック・機械的） ---

def _collect_migrations(schema: dict) -> dict:
    return {
        name: fdef["x-migration"]
        for name, fdef in schema.get("properties", {}).items()
        if "x-migration" in fdef
    }

def _apply_mechanical(mig: dict, doc: dict):
    as_ = mig["as"]
    if as_ == "rename":
        return doc.get(mig["from"])
    if as_ == "default":
        return mig["value"]
    if as_ == "value-map":
        old_value = doc.get(mig["from"])
        return mig["mapping"].get(old_value, old_value)
    if as_ == "discriminator-remap":
        content = doc.get("content", {})
        for rule in mig["rules"]:
            if rule["ifHasField"] in content:
                return rule["then"]
        return None
    return None

