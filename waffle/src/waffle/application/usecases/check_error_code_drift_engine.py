"""check error code drift engine — usecase specのErrorsブロックが宣言するエラーコードが、
@specタグでリンクされた実装コードに文字列リテラルとして実在するかを検証する
application use case。

docs/brainstorm/sim-code-spec-link-projection.mdで設計した「@specアンカー＋投影」方式の
最初の具体化。spec proseと実装コードの意味的な乖離を、人手の監査に頼らず継続的に検知する。

@spec uc-check-error-code-drift
"""
from __future__ import annotations

import re

from waffle.application.ports.document_repository import DocumentRepository
from waffle.shared.path_confinement import is_confined
from waffle.shared.result import Err, Ok, Result

_SPEC_TAG = re.compile(r"@spec\s+(\S+)")


def _err(code: str, message: str) -> Err:
    return Err(message, [code])


class CheckErrorCodeDriftEngine:
    def __init__(self, documents: DocumentRepository) -> None:
        self._documents = documents

    def run(self, specs_root: str, code_root: str) -> Result[dict]:
        if not is_confined(specs_root) or not is_confined(code_root):
            return _err("INVALID_PATH", f"パストラバーサルは許可されません: {specs_root}, {code_root}")
        try:
            spec_paths = self._documents.list_files(specs_root, "**/*.json")
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ディレクトリが見つかりません: {specs_root}")
        try:
            code_paths = self._documents.list_files(code_root, "**/*.py")
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ディレクトリが見つかりません: {code_root}")

        links: dict[str, list[str]] = {}
        code_texts: dict[str, str] = {}
        for path in code_paths:
            text = self._documents.read_text(path)
            code_texts[path] = text
            for spec_id in _SPEC_TAG.findall(text):
                links.setdefault(spec_id, []).append(path)

        unlinked_specs: list[str] = []
        missing_error_codes: list[dict] = []
        for spec_path in spec_paths:
            doc = self._documents.load(spec_path)
            codes = [e["code"] for e in doc.get("content", {}).get("errors", {}).get("items", [])]
            if not codes:
                continue
            document_id = doc.get("documentId")
            linked_files = links.get(document_id, [])
            if not linked_files:
                unlinked_specs.append(document_id)
                continue
            for code in codes:
                if not any(code in code_texts[f] for f in linked_files):
                    missing_error_codes.append({"usecase": document_id, "code": code, "files": linked_files})

        return Ok({
            "unlinked_specs": sorted(unlinked_specs),
            "missing_error_codes": missing_error_codes,
        })
