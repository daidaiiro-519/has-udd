"""document_loading — agg-document/agg-schema の不変条件のうち、DocumentRepository/
SchemaRepositoryを介する（port必須の）読込・エラーマッピングを一箇所に集約する
application層の共通ヘルパー。5つのengine(query/render/validate/scaffold/migrate)に
同一ロジックが重複していたものを、ここに集約した。

パス解決の核心（G6/G7）自体は port 不要の純粋ロジックであり
domain/services/path_confinement.py が担う。ここはその判定結果を使い、port経由の
読込を実行してResultへマッピングする「編成」だけを担当する。
"""
from __future__ import annotations

import json

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.schema_repository import SchemaRepository
from waffle.domain.services.path_confinement import is_confined
from waffle.shared.result import Err, Ok, Result


def _err(code: str, message: str) -> Err:
    return Err(message, [code])


def load_document(documents: DocumentRepository, path: str) -> Result[dict]:
    """G6パス確認＋document読込＋エラーマッピング(INVALID_PATH/INVALID_JSON)。"""
    if not is_confined(path):
        return _err("INVALID_PATH", f"パストラバーサルは許可されません: {path}")
    try:
        return Ok(documents.load(path))
    except FileNotFoundError:
        return _err("INVALID_PATH", f"ファイルが見つかりません: {path}")
    except json.JSONDecodeError:
        return _err("INVALID_JSON", f"JSON として解釈できません: {path}")


def require_schema_ref(document: dict) -> Result[str]:
    """document が schemaRef を持つことを確認する(MISSING_SCHEMA_REF)。"""
    schema_ref = document.get("schemaRef")
    if not schema_ref:
        return _err("MISSING_SCHEMA_REF", "document に schemaRef がありません")
    return Ok(schema_ref)


def load_schema(schemas: SchemaRepository, schema_ref: str) -> Result[dict]:
    """schema解決＋エラーマッピング(INVALID_SCHEMA_REF)。"""
    try:
        return Ok(schemas.load(schema_ref))
    except (FileNotFoundError, ModuleNotFoundError):
        return _err("INVALID_SCHEMA_REF", f"schema を解決できません: {schema_ref}")
