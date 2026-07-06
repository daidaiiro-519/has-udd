"""scaffold engine — document.json の骨格生成（create）と値書き込み（fill）。

Harness 原則: AI は「値」だけを生成し、document.json の構造は engine が組む。
- create: schema を機械走査して skeleton（自分の schema で valid・status=enum 先頭）と
  fillTemplate（値フィールドの path × x-prompt-write）を生成し、x-source-target に書く。
- fill: AI が生成した values を、宣言済み値フィールドにのみ機械的に書き込む（構造保護）。
  値の型/enum 適合検証は uc-validate-document の責務（疎結合）。
"""
from __future__ import annotations

import json

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.schema_repository import SchemaRepository
from waffle.application.services.document_loading import load_document, load_schema, require_schema_ref
from waffle.domain.services import path_template
from waffle.domain.services.schema_discriminator import discriminator_key as _discriminator_key
from waffle.shared.result import Err, Ok, Result

def _err(code: str, message: str) -> Err:
    return Err(message, [code])

class ScaffoldEngine:
    def __init__(self, documents: DocumentRepository, schemas: SchemaRepository) -> None:
        self._documents = documents
        self._schemas = schemas

    def run(self, operation: str, params: dict | None = None) -> Result[dict]:
        params = params or {}
        if operation == "create":
            return self._create(params)
        if operation == "fill":
            return self._fill(params)
        return _err("INVALID_OPERATION", f"未知の operation: {operation}")

    def _create(self, params: dict) -> Result[dict]:
        schema_ref = params.get("schemaRef")
        document_id = params.get("documentId")
        if not schema_ref or not document_id:
            return _err("MISSING_PARAM", "create には schemaRef, documentId が必要です")
        schema_result = load_schema(self._schemas, schema_ref)
        if isinstance(schema_result, Err):
            return schema_result
        schema = schema_result.value

        disc_key = _discriminator_key(schema)
        discriminator = params.get("discriminator") or {}
        if disc_key and disc_key not in discriminator:
            cands = ", ".join(_discriminator_candidates(schema, disc_key))
            return _err("MISSING_DISCRIMINATOR", f"{disc_key} の指定が必要です（候補: {cands}）")

        content_def = _content_def(schema, discriminator)
        skeleton = _build_skeleton(schema, document_id, disc_key, discriminator, content_def)
        fill_template = _build_fill_template(schema, content_def)

        x_source = schema.get("x-source-target") or ""
        template = x_source.get(discriminator.get(disc_key)) if isinstance(x_source, dict) else x_source
        path_vars = {"documentId": document_id, **{k: v for k, v in params.items() if isinstance(v, str)}}
        path = path_template.resolve(template, **path_vars) if template else ""
        if path:
            existing = self._existing_document(path)
            if existing is not None:
                # べき等性: 既にdocument.jsonがある場合は、fillで埋めた既存のvaluesを保護し
                # 骨格で上書きしない（構造は保存時に既に保証済み）。
                skeleton = existing
            else:
                self._documents.save(path, skeleton)
        return Ok({"skeleton": skeleton, "fillTemplate": fill_template, "path": path})

    def _existing_document(self, path: str) -> dict | None:
        try:
            return self._documents.load(path)
        except (FileNotFoundError, json.JSONDecodeError):
            return None

    def _fill(self, params: dict) -> Result[dict]:
        document_path = params.get("documentPath")
        values = params.get("values")
        if not document_path or values is None:
            return _err("MISSING_PARAM", "fill には documentPath, values が必要です")
        loaded = load_document(self._documents, document_path)
        if isinstance(loaded, Err):
            return loaded
        doc = loaded.value

        schema_ref_result = require_schema_ref(doc)
        if isinstance(schema_ref_result, Err):
            return schema_ref_result
        schema_ref = schema_ref_result.value

        schema_result = load_schema(self._schemas, schema_ref)
        if isinstance(schema_result, Err):
            return schema_result
        schema = schema_result.value

        disc_key = _discriminator_key(schema)
        discriminator = {disc_key: doc.get(disc_key)} if disc_key else {}
        content_def = _content_def(schema, discriminator)
        allowed = {e["path"] for e in _build_fill_template(schema, content_def)}

        written: list[str] = []
        skipped: list[str] = []
        for path, value in values.items():
            if path in allowed and _set_path(doc, path, value):
                written.append(path)
            else:
                skipped.append(path)  # 未知 / const / discriminator / 構造改変は拒否
        self._documents.save(document_path, doc)
        return Ok({"documentPath": document_path, "written": written, "skipped": skipped})

# --- schema 走査ヘルパ（純ロジック・機械的） ---

def _resolve(schema: dict, ref: str):
    node = schema
    for part in ref.lstrip("#/").split("/"):
        node = node[part]
    return node

def _discriminator_candidates(schema: dict, key: str) -> list:
    return schema.get("properties", {}).get(key, {}).get("enum", [])

def _matches(if_clause: dict, discriminator: dict) -> bool:
    for key, cond in if_clause.get("properties", {}).items():
        if "const" in cond and discriminator.get(key) != cond["const"]:
            return False
    return True

def _content_def(schema: dict, discriminator: dict) -> dict:
    if "if" in schema:
        branch = schema.get("then", {}) if _matches(schema["if"], discriminator) else schema.get("else", {})
        ref = branch.get("properties", {}).get("content", {}).get("$ref")
        if ref:
            return _resolve(schema, ref)
    for entry in schema.get("allOf", []):
        if "if" in entry and _matches(entry["if"], discriminator):
            ref = entry.get("then", {}).get("properties", {}).get("content", {}).get("$ref")
            if ref:
                return _resolve(schema, ref)
    return schema.get("properties", {}).get("content", {})

def _skeleton_from_def(schema: dict, d: dict):
    if "$ref" in d:
        return _skeleton_from_def(schema, _resolve(schema, d["$ref"]))
    if "const" in d:
        return d["const"]
    if "enum" in d:
        return d["enum"][0]
    t = d.get("type")
    if t == "object":
        return {k: _skeleton_from_def(schema, v) for k, v in d.get("properties", {}).items()}
    if t == "array":
        return []
    if t == "string":
        return ""
    if t in ("number", "integer"):
        return 0
    if t == "boolean":
        return False
    return None

def _build_skeleton(schema, document_id, disc_key, discriminator, content_def) -> dict:
    props = schema.get("properties", {})
    out: dict = {}
    for name in schema.get("required", []):
        if name == "documentId":
            out[name] = document_id
        elif name == disc_key:
            out[name] = discriminator[disc_key]
        elif name == "content":
            out[name] = _skeleton_from_def(schema, content_def)
        else:
            out[name] = _skeleton_from_def(schema, props.get(name, {}))
    if "tags" in props and "tags" not in out:
        out["tags"] = []
    return out

def _merge_allof(schema: dict, d: dict) -> dict:
    if "allOf" not in d:
        return d
    merged: dict = {"properties": {}}
    for part in d["allOf"]:
        if "$ref" in part:
            part = _resolve(schema, part["$ref"])
        merged["properties"].update(part.get("properties", {}))
    merged["properties"].update(d.get("properties", {}))
    return merged

def _build_fill_template(schema: dict, content_def: dict) -> list:
    entries: list = []
    _walk_fill(schema, content_def, "content", entries)
    return entries

def _walk_fill(schema, d, path, entries):
    if "$ref" in d:
        return _walk_fill(schema, _resolve(schema, d["$ref"]), path, entries)
    t = d.get("type")
    if t == "object":
        for k, v in d.get("properties", {}).items():
            _walk_fill(schema, v, f"{path}.{k}", entries)
        return
    if "const" in d or "x-prompt-write" not in d:
        return
    entry = {"path": path, "type": t or "any", "prompt": d["x-prompt-write"], "required": True}
    if "enum" in d:
        entry["enum"] = d["enum"]
    if t == "array":
        item = d.get("items", {})
        if "$ref" in item:
            item = _resolve(schema, item["$ref"])
        item = _merge_allof(schema, item)
        element = {
            ik: iv["x-prompt-write"]
            for ik, iv in item.get("properties", {}).items()
            if "x-prompt-write" in iv and "const" not in iv
        }
        if element:
            entry["element"] = element
    entries.append(entry)

def _set_path(doc: dict, path: str, value) -> bool:
    parts = path.split(".")
    cur = doc
    for p in parts[:-1]:
        if not isinstance(cur, dict) or p not in cur:
            return False
        cur = cur[p]
    if isinstance(cur, dict) and parts[-1] in cur:
        cur[parts[-1]] = value
        return True
    return False
