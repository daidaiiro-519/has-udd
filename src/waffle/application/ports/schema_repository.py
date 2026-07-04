"""SchemaRepository port — schema 解決の Secondary Port。"""
from __future__ import annotations

from typing import Protocol


class SchemaRepository(Protocol):
    def load(self, schema_ref: str) -> dict:
        """schemaRef（例: 'CodingSchema/v2'）から schema(dict) を返す。"""
        ...
