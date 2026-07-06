"""SchemaRepository port — schema 解決の Secondary Port。"""
from __future__ import annotations

from typing import Protocol


class SchemaRepository(Protocol):
    def load(self, schema_ref: str) -> dict:
        """schemaRef（例: 'CodingSchema/v2'）から schema(dict) を返す。"""
        ...

    def list_versions(self, name: str) -> list[str]:
        """name（例: 'CodingSchema'）配下に実在する版識別子（例: ['v1', 'v2']）一覧を返す。無ければ空配列。"""
        ...
