"""SampleAggregate（整合性境界を持つ集約）を定義する。"""

from __future__ import annotations

from example_hex.domain.model.sample_item import SampleItem
from example_hex.domain.model.sample_name import SampleName

_MAX_ITEMS = 5

ITEM_LIMIT_EXCEEDED = "ITEM_LIMIT_EXCEEDED"
DUPLICATE_ITEM_NAME = "DUPLICATE_ITEM_NAME"
ITEM_NOT_FOUND = "ITEM_NOT_FOUND"


class SampleInvariantError(Exception):
    """SampleAggregate の不変条件違反を表すドメイン例外。

    application 境界でこの例外は握り潰さず、Result 型（code 付き）へ
    写像すること。

    Attributes:
        code: 違反の種類を識別する定数文字列。
        message: 人が読むための説明。
    """

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code
        self.message = message


class SampleAggregate:
    """アイテムの集合に対する整合性境界を持つ集約ルート。

    不変条件（アイテム名の重複禁止・最大件数）はコマンドメソッド
    （`add_item` 等）の内部で強制する。永続化は repository 経由で行い、
    このクラス自身は I/O を持たない。

    Attributes:
        aggregate_id: 集約を一意に識別する id。
        name: 集約の名前（値オブジェクト）。
    """

    def __init__(self, aggregate_id: str, name: SampleName) -> None:
        self.aggregate_id = aggregate_id
        self.name = name
        self._items: list[SampleItem] = []

    @property
    def items(self) -> tuple[SampleItem, ...]:
        """集約が保持するアイテムを読み取り専用で返す。"""
        return tuple(self._items)

    def add_item(self, item_id: str, item_name: SampleName) -> SampleItem:
        """アイテムを1件追加するコマンド。

        Args:
            item_id: 追加するアイテムの id。
            item_name: 追加するアイテムの名前。

        Returns:
            追加された SampleItem。

        Raises:
            SampleInvariantError: 名前が重複する（code=DUPLICATE_ITEM_NAME）、
                または最大件数を超える場合（code=ITEM_LIMIT_EXCEEDED）。
        """
        if len(self._items) >= _MAX_ITEMS:
            raise SampleInvariantError(
                ITEM_LIMIT_EXCEEDED,
                f"アイテムは最大 {_MAX_ITEMS} 件までしか保持できない",
            )
        if any(existing.name == item_name for existing in self._items):
            raise SampleInvariantError(
                DUPLICATE_ITEM_NAME,
                f"アイテム名 '{item_name.value}' は既に使用されている",
            )
        item = SampleItem(item_id=item_id, name=item_name)
        self._items.append(item)
        return item

    def deactivate_item(self, item_id: str) -> None:
        """指定した id のアイテムを無効化するコマンド。

        Args:
            item_id: 無効化するアイテムの id。

        Raises:
            SampleInvariantError: 指定した id のアイテムが存在しない場合
                （code=ITEM_NOT_FOUND）。
        """
        for existing in self._items:
            if existing.item_id == item_id:
                existing.deactivate()
                return
        raise SampleInvariantError(
            ITEM_NOT_FOUND, f"id '{item_id}' のアイテムは存在しない"
        )
