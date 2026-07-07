"""check schema version drift engine — Document集約の実インスタンス群が参照する
schemaRefの実在性・最新版との一致を検証する application use case。

実行/意味理解はしない（schemaRef文字列と実在するバージョン集合の機械的な突き合わせのみ）。
"""
from __future__ import annotations

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.schema_repository import SchemaRepository
from waffle.domain.services.fill_template import build_fill_template, content_def
from waffle.domain.services.schema_discriminator import discriminator_key
from waffle.domain.services.schema_versioning import version_number
from waffle.shared.path_confinement import is_confined
from waffle.shared.result import Err, Ok, Result


def _err(code: str, message: str) -> Err:
    return Err(message, [code])


def _path_exists(doc: dict, path: str) -> bool:
    parts = path.split(".")
    cur = doc
    for p in parts:
        if not isinstance(cur, dict) or p not in cur:
            return False
        cur = cur[p]
    return True


class CheckSchemaVersionDriftEngine:
    def __init__(self, documents: DocumentRepository, schemas: SchemaRepository) -> None:
        self._documents = documents
        self._schemas = schemas

    def run(self, documents_root: str) -> Result[dict]:
        if not is_confined(documents_root):
            return _err("INVALID_PATH", f"パストラバーサルは許可されません: {documents_root}")
        try:
            doc_paths = self._documents.list_files(documents_root, "**/*.json")
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ディレクトリが見つかりません: {documents_root}")

        broken_references: list[dict] = []
        newer_version_available: list[dict] = []
        missing_declared_fields: list[dict] = []

        for doc_path in doc_paths:
            doc = self._documents.load(doc_path)
            schema_ref = doc.get("schemaRef")
            if not schema_ref or "/" not in schema_ref:
                continue
            name, version = schema_ref.split("/", 1)
            versions = [v for v in self._schemas.list_versions(name) if version_number(v) is not None]
            if version not in versions:
                broken_references.append({"document": doc_path, "schemaRef": schema_ref})
                continue
            latest = max(versions, key=version_number)
            if version_number(version) != version_number(latest):
                newer_version_available.append({
                    "document": doc_path,
                    "schemaRef": schema_ref,
                    "latest": f"{name}/{latest}",
                })

            schema = self._schemas.load(schema_ref)
            disc_key = discriminator_key(schema)
            discriminator = {disc_key: doc.get(disc_key)} if disc_key else {}
            content = content_def(schema, discriminator)
            for entry in build_fill_template(schema, content):
                if entry["required"] and not _path_exists(doc, entry["path"]):
                    missing_declared_fields.append({"document": doc_path, "path": entry["path"]})

        return Ok({
            "broken_references": broken_references,
            "newer_version_available": newer_version_available,
            "missing_declared_fields": missing_declared_fields,
        })
