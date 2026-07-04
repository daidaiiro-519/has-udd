"""Result — 全 engine の戻り値の共通エンベロープ。

成功時は Ok(value)、失敗時は Err(message, details) を返す統一型。
呼び出し側は例外ではなくこの型で成否を判定する。
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
