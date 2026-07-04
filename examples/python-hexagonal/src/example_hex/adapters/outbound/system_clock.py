"""ClockPort の実装（outbound adapter）を定義する。"""

from __future__ import annotations

from datetime import datetime, timezone

from example_hex.application.ports.clock_port import ClockPort


class SystemClock(ClockPort):
    """OS の現在時刻を返す ClockPort 実装。

    外部ライブラリ（標準の datetime）への依存をこの adapter に
    閉じ込め、application / domain からは直接参照させない。
    """

    def now_iso(self) -> str:
        """現在の UTC 時刻を ISO 8601 形式の文字列で返す。

        Returns:
            現在時刻を表す ISO 8601 形式の文字列。
        """
        return datetime.now(timezone.utc).isoformat()
