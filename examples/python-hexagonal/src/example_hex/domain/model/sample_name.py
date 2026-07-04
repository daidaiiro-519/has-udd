"""SampleAggregate が扱う値オブジェクトを定義する。

value-object は不変（frozen）であり、等価性は値そのもので判定される
（同一性を持たない）。
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class SampleName:
    """アイテム／集約の名前を表す値オブジェクト。

    不変であり、生成時に空文字を許さない不変条件を強制する。
    等価性は `value` の値そのもので判定される（frozen dataclass の値等価）。

    Attributes:
        value: 名前の文字列表現。空文字は許可されない。

    Raises:
        ValueError: value が空文字の場合。
    """

    value: str

    def __post_init__(self) -> None:
        if not self.value.strip():
            raise ValueError("SampleName は空文字であってはならない")
