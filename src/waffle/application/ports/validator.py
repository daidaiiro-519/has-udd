"""Validator port — document の schema 適合検証の Secondary Port。"""
from __future__ import annotations

from typing import Protocol


class Validator(Protocol):
    def validate(self, document: dict, schema: dict) -> list[str]:
        """schema 適合を検証し、違反メッセージのリストを返す（空リスト=PASS）。"""
        ...
