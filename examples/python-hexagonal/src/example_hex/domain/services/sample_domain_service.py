"""複数の集約を跨る計算を行うドメインサービスを定義する。"""

from __future__ import annotations

from example_hex.domain.model.sample_aggregate import SampleAggregate


class SampleDomainService:
    """複数の SampleAggregate を跨る計算を行うステートレスなサービス。

    単一の集約の内部に置くには適さない、集約横断の判断ロジックを
    ここに置く。インスタンス状態を持たない。
    """

    def more_active_items(
        self, left: SampleAggregate, right: SampleAggregate
    ) -> SampleAggregate:
        """有効なアイテム数がより多い方の集約を返す。

        Args:
            left: 比較対象の集約その1。
            right: 比較対象の集約その2。

        Returns:
            有効アイテム数が多い方の集約（同数の場合は left）。
        """
        left_count = sum(1 for item in left.items if item.active)
        right_count = sum(1 for item in right.items if item.active)
        return left if left_count >= right_count else right
