"""ファイルシステム adapter（DocumentRepository 実装）。

document.json の読み書きおよび任意テキスト/ディレクトリ走査をローカル
ファイルシステム上で行う。
"""
from __future__ import annotations

import json
from pathlib import Path

from waffle.application.ports.document_repository import DocumentRepository

class FsDocumentRepository(DocumentRepository):
    def load(self, path: str) -> dict:
        return json.loads(Path(path).read_text(encoding="utf-8"))

    def save(self, path: str, document: dict) -> None:
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(
            json.dumps(document, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )

    def write_text(self, path: str, text: str) -> None:
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text, encoding="utf-8")

    def read_text(self, path: str) -> str:
        return Path(path).read_text(encoding="utf-8")

    def list_json(self, directory: str) -> list[str]:
        d = Path(directory)
        if not d.is_dir():
            raise FileNotFoundError(directory)
        return sorted(str(p) for p in d.glob("*.json"))
