"""DocumentRepository port — document.json の読み書きの Secondary Port。"""
from __future__ import annotations

from typing import Protocol


class DocumentRepository(Protocol):
    def load(self, path: str) -> dict:
        """path の document.json を dict で返す。"""
        ...

    def save(self, path: str, document: dict) -> None:
        """document を path に書き込む。"""
        ...

    def write_text(self, path: str, text: str) -> None:
        """レンダリング成果物などのテキストを path に書き込む。"""
        ...

    def read_text(self, path: str) -> str:
        """path のファイルを生テキストで返す（scan / raw フォールバック用）。無ければ FileNotFoundError。"""
        ...

    def list_json(self, directory: str) -> list[str]:
        """directory 直下の *.json パス一覧（昇順）。ディレクトリが無ければ FileNotFoundError。"""
        ...

    def list_dirs(self, directory: str) -> list[str]:
        """directory 直下のサブディレクトリ名一覧（昇順）。ディレクトリが無ければ FileNotFoundError。"""
        ...
