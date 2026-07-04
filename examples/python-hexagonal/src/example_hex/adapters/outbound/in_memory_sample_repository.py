"""SampleRepositoryPort のインメモリ実装（outbound adapter）を定義する。"""

from __future__ import annotations

from example_hex.application.ports.sample_repository_port import (
    SampleRepositoryPort,
)
from example_hex.domain.model.sample_aggregate import SampleAggregate


class InMemorySampleRepository(SampleRepositoryPort):
    """プロセス内メモリに SampleAggregate を保持する repository 実装。

    本来は DB 等の外部ライブラリをここに閉じ込める想定だが、この
    サンプルでは最小構成としてインメモリ辞書に保存する。
    """

    def __init__(self) -> None:
        self._store: dict[str, SampleAggregate] = {}

    def load(self, aggregate_id: str) -> SampleAggregate | None:
        """id で SampleAggregate を取得する。

        Args:
            aggregate_id: 取得したい集約の id。

        Returns:
            見つかった場合は SampleAggregate、無ければ None。
        """
        return self._store.get(aggregate_id)

    def save(self, aggregate: SampleAggregate) -> None:
        """SampleAggregate をインメモリ辞書へ保存する。

        Args:
            aggregate: 保存する集約。
        """
        self._store[aggregate.aggregate_id] = aggregate
