"""query engine — document.json / 通常ファイルへのセマンティック・クエリ。

16 のセマンティック操作で、AI がファイルを直接読まずに必要な意味単位だけを取得する。
構造アクセスは全て Python が担い、`{ prompt, value }` を返す（prompt=value の読み方の指針＝
対象 block の x-prompt-query を schema から動的算出）。schemaRef を持たないファイルは raw フォールバック。
全エラーは Result.Err（details[0]=エラーコード）で構造化し、例外を AI に素通りさせない。

@spec:uc-query-document
"""
from __future__ import annotations

import json
import re
from pathlib import Path

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.schema_repository import SchemaRepository
from waffle.shared.result import Err, Ok, Result

# 各 operation の必須パラメータ（path を除く）
_REQUIRED: dict[str, list[str]] = {
    "scan": [],
    "get_meta": [],
    "index_scan": [],
    "index_scan_dir": [],
    "get_block": ["blockKey"],
    "get_field": ["blockKey", "field"],
    "get_items": ["blockKey", "arrayField"],
    "get_item_field": ["blockKey", "arrayField", "field"],
    "get_items_slice": ["blockKey", "arrayField", "start", "end"],
    "filter_items": ["blockKey", "arrayField", "key", "value"],
    "filter_exists": ["blockKey", "arrayField", "field"],
    "filter_pattern": ["blockKey", "arrayField", "field", "pattern"],
    "get_by_id": ["blockKey", "arrayField", "idField", "idValue"],
    "get_nested_items": ["blockKey", "arrayField", "nestedField"],
    "get_children": ["blockKey", "arrayField", "idField", "idValue"],
    "find_all": ["fieldName"],
}

_META_FIELDS = ("documentId", "documentType", "schemaRef", "skillKind", "codingKind", "status", "tags")


class QueryEngine:
    def __init__(self, documents: DocumentRepository, schemas: SchemaRepository) -> None:
        self._documents = documents
        self._schemas = schemas

    def run(self, operation: str, path: str, params: dict | None = None) -> Result[dict]:
        # has-udd:impl-start
        params = params or {}
        if operation not in _REQUIRED:
            return _err("INVALID_OPERATION", f"未知の operation: {operation}")
        missing = [p for p in _REQUIRED[operation] if params.get(p) in (None, "")]
        if missing:
            return _err("MISSING_PARAM", f"{operation} には {', '.join(missing)} が必要です")

        # G6: パストラバーサル拒否（全 operation・読み取り対象を制限）
        if ".." in Path(path).parts:
            return _err("INVALID_PATH", f"パストラバーサルは許可されません: {path}")

        # Group 1: ファイル/ディレクトリ単位（schema 不要なものを先に処理）
        if operation == "scan":
            try:
                return Ok({"prompt": None, "value": self._documents.read_text(path)})
            except FileNotFoundError:
                return _err("INVALID_PATH", f"ファイルが見つかりません: {path}")
        if operation == "index_scan_dir":
            return self._index_scan_dir(path)

        # それ以外は document を読む
        try:
            doc = self._documents.load(path)
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ファイルが見つかりません: {path}")
        except json.JSONDecodeError:
            return _err("INVALID_JSON", f"JSON として解釈できません: {path}")

        # schemaRef を持たない通常ファイル → raw フォールバック
        if not isinstance(doc, dict) or "schemaRef" not in doc:
            return Ok({"type": "raw", "content": self._documents.read_text(path)})

        schema = self._schemas.load(doc["schemaRef"])
        return self._dispatch(operation, doc, schema, params)
        # has-udd:impl-end

    # --- ディスパッチ ---

    def _dispatch(self, operation: str, doc: dict, schema: dict, params: dict) -> Result[dict]:
        # has-udd:impl-start
        if operation == "get_meta":
            return Ok({"prompt": None, "value": {k: doc[k] for k in _META_FIELDS if k in doc}})
        if operation == "index_scan":
            return Ok({"prompt": None, "value": _index(doc, schema)})
        if operation == "find_all":
            return Ok({"prompt": None, "value": _find_all(doc, params["fieldName"])})

        # Group 2/3: block が必要
        block = doc.get("content", {}).get(params["blockKey"])
        if not isinstance(block, dict):
            return _err("NOT_FOUND", f"block が見つかりません: {params['blockKey']}")
        prompt = _block_prompt(schema, block)

        if operation == "get_block":
            return Ok({"prompt": prompt, "value": block})
        if operation == "get_field":
            field = params["field"]
            if field not in block:
                return _err("NOT_FOUND", f"field が見つかりません: {field}")
            return Ok({"prompt": prompt, "value": block[field]})

        # Group 3: 配列が必要
        arr = block.get(params["arrayField"])
        if not isinstance(arr, list):
            return _err("NOT_FOUND", f"配列フィールドが見つかりません: {params['arrayField']}")
        return self._array_op(operation, arr, prompt, params)

    def _array_op(self, operation: str, arr: list, prompt, params: dict) -> Result[dict]:
        if operation == "get_items":
            value = arr
        elif operation == "get_item_field":
            value = [x[params["field"]] for x in arr if isinstance(x, dict) and params["field"] in x]
        elif operation == "get_items_slice":
            try:
                start, end = int(params["start"]), int(params["end"])
            except (TypeError, ValueError):
                return _err("MISSING_PARAM", "start / end は整数で指定してください")
            value = arr[start:end]
        elif operation == "filter_items":
            key, want = params["key"], params["value"]
            value = [x for x in arr if isinstance(x, dict) and _eq(x.get(key), want)]
        elif operation == "filter_exists":
            field = params["field"]
            value = [x for x in arr if isinstance(x, dict) and field in x]
        elif operation == "filter_pattern":
            field = params["field"]
            try:
                rx = re.compile(params["pattern"])
            except re.error as e:
                return _err("INVALID_PATTERN", f"正規表現が不正です: {e}")
            value = [x for x in arr if isinstance(x, dict) and field in x and rx.search(str(x[field]))]
        elif operation == "get_by_id":
            hit = _find_by_id(arr, params["idField"], params["idValue"])
            if hit is None:
                return _err("NOT_FOUND", f"{params['idField']}={params['idValue']} の要素がありません")
            value = hit
        elif operation == "get_nested_items":
            nf = params["nestedField"]
            value = [sub for x in arr if isinstance(x, dict) for sub in x.get(nf, [])]
        elif operation == "get_children":
            hit = _find_by_id(arr, params["idField"], params["idValue"])
            if hit is None:
                return _err("NOT_FOUND", f"{params['idField']}={params['idValue']} の要素がありません")
            value = hit.get("children", [])
        else:  # pragma: no cover — _REQUIRED で網羅済み
            return _err("INVALID_OPERATION", f"未知の operation: {operation}")
        return Ok({"prompt": prompt, "value": value})

    def _index_scan_dir(self, directory: str) -> Result[dict]:
        # G7: index_scan_dir はプロジェクトルート配下のディレクトリのみ対象
        root = Path.cwd().resolve()
        target = Path(directory).resolve()
        if target != root and root not in target.parents:
            return _err("INVALID_PATH", f"プロジェクトルート外は走査できません: {directory}")
        try:
            paths = self._documents.list_json(directory)
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ディレクトリが見つかりません: {directory}")
        out: dict[str, dict] = {}
        for p in paths:
            doc = self._documents.load(p)
            if isinstance(doc, dict) and "schemaRef" in doc:
                out[p] = _index(doc, self._schemas.load(doc["schemaRef"]))
        return Ok({"prompt": None, "value": out})
        # has-udd:impl-end


# --- 純ヘルパ ---

def _err(code: str, message: str) -> Err:
    return Err(message, [code])


def _eq(a, b) -> bool:
    # has-udd:impl-start
    return a == b or str(a).lower() == str(b).lower()
    # has-udd:impl-end


def _block_prompt(schema: dict, block: dict):
    # has-udd:impl-start
    bdef = schema.get("$defs", {}).get(f"{block.get('blockType')}Block", {})
    return bdef.get("x-prompt-query")
    # has-udd:impl-end


def _index(doc: dict, schema: dict) -> dict:
    """blockType × schema.x-prompt-query から _index を読み取り時に動的算出する（保存はしない）。"""
    # has-udd:impl-start
    defs = schema.get("$defs", {})
    out: dict[str, dict] = {}
    for key, block in doc.get("content", {}).items():
        bt = block.get("blockType") if isinstance(block, dict) else None
        out[key] = {"blockType": bt, "prompt": defs.get(f"{bt}Block", {}).get("x-prompt-query")}
    return out
    # has-udd:impl-end


def _find_by_id(arr: list, id_field: str, id_value):
    # has-udd:impl-start
    for x in arr:
        if isinstance(x, dict) and _eq(x.get(id_field), id_value):
            return x
    return None
    # has-udd:impl-end


def _find_all(node, field: str) -> list:
    """node 配下を再帰走査し field の値を全て集める（全階層検索）。"""
    # has-udd:impl-start
    out: list = []
    if isinstance(node, dict):
        if field in node:
            out.append(node[field])
        for v in node.values():
            out.extend(_find_all(v, field))
    elif isinstance(node, list):
        for v in node:
            out.extend(_find_all(v, field))
    return out
    # has-udd:impl-end
