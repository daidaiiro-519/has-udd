"""application 境界で成否を表す結果型を提供する。

domain の不変条件違反はドメイン例外で表すが、application の
エントリメソッド（usecase）は例外を外へ漏らさず、この Result で
成否を呼び出し側へ返す（境界での写像）。失敗は識別可能なエラーコード
（定数文字列）を必ず伴う（メッセージ文字列のみは不可）。
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Generic, TypeVar

T = TypeVar("T")


@dataclass(frozen=True)
class Ok(Generic[T]):
    """成功した結果を保持する。

    Attributes:
        value: 成功時の値。
    """

    value: T

    @property
    def is_ok(self) -> bool:
        """常に True を返す（成功であることを表す）。"""
        return True


@dataclass(frozen=True)
class Err:
    """失敗した結果を保持する。

    Attributes:
        code: 失敗を識別する定数文字列（呼び出し側が種類を判定するために使う）。
        message: 人が読むための説明。
    """

    code: str
    message: str

    @property
    def is_ok(self) -> bool:
        """常に False を返す（失敗であることを表す）。"""
        return False


Result = Ok[T] | Err
"""成功（Ok）または失敗（Err）のいずれかを表す合併型。"""
