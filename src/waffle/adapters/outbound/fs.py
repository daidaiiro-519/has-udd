"""ファイルシステム adapter（DocumentRepository 実装）。

@stack:filesystem
"""
from __future__ import annotations

import json
from pathlib import Path

from waffle.application.ports.document_repository import DocumentRepository


class FsDocumentRepository(DocumentRepository):
    def load(self, path: str) -> dict:
        # has-udd:impl-start
        return json.loads(Path(path).read_text(encoding="utf-8"))
        # has-udd:impl-end

    def save(self, path: str, document: dict) -> None:
        # has-udd:impl-start
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(
            json.dumps(document, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        # has-udd:impl-end

    def write_text(self, path: str, text: str) -> None:
        # has-udd:impl-start
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text, encoding="utf-8")
        # has-udd:impl-end

    def read_text(self, path: str) -> str:
        # has-udd:impl-start
        return Path(path).read_text(encoding="utf-8")
        # has-udd:impl-end

    def list_json(self, directory: str) -> list[str]:
        # has-udd:impl-start
        d = Path(directory)
        if not d.is_dir():
            raise FileNotFoundError(directory)
        return sorted(str(p) for p in d.glob("*.json"))
        # has-udd:impl-end
