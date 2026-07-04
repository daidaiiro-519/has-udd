"""SampleAggregate 用 repository の driven interface を定義する。"""

from __future__ import annotations

from abc import ABC, abstractmethod

from example_hex.domain.model.sample_aggregate import SampleAggregate


class SampleRepositoryPort(ABC):
    """SampleAggregate の load/save を要求する driven interface。

    集約1つにつき1つの repository を用意する規約に従い、
    この port は SampleAggregate 専用とする。実装（外部ライブラリ等）は
    adapters/outbound に閉じ込める。
    """

    @abstractmethod
    def load(self, aggregate_id: str) -> SampleAggregate | None:
        """id で SampleAggregate を取得する。

        Args:
            aggregate_id: 取得したい集約の id。

        Returns:
            見つかった場合は SampleAggregate、無ければ None。
        """
        raise NotImplementedError

    @abstractmethod
    def save(self, aggregate: SampleAggregate) -> None:
        """SampleAggregate を永続化する。

        Args:
            aggregate: 保存する集約。
        """
        raise NotImplementedError
