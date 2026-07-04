"""repository 以外の一般的な driven interface の例として時刻取得を定義する。"""

from __future__ import annotations

from abc import ABC, abstractmethod


class ClockPort(ABC):
    """application が要求する「現在時刻を得る」ための driven interface。

    repository 以外にも application は外部依存を port 経由で要求する、
    という一般的な port の形の最小例として置く。
    """

    @abstractmethod
    def now_iso(self) -> str:
        """現在時刻を ISO 8601 形式の文字列で返す。

        Returns:
            現在時刻を表す ISO 8601 形式の文字列。
        """
        raise NotImplementedError
