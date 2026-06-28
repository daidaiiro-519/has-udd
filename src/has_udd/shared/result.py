"""Result — 全 engine の戻り値の共通エンベロープ。

@stack:error-handling
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Generic, TypeVar, Union

T = TypeVar("T")


@dataclass(frozen=True)
class Ok(Generic[T]):
    """成功。value に成果物を持つ。"""

    value: T


@dataclass(frozen=True)
class Err:
    """失敗。message と details(任意) を持つ。"""

    message: str
    details: list[str] = field(default_factory=list)


Result = Union[Ok[T], Err]
