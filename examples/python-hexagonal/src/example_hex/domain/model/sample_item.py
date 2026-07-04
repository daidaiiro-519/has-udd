"""SampleAggregate の内側に存在するエンティティを定義する。"""

from __future__ import annotations

from example_hex.domain.model.sample_name import SampleName


class SampleItem:
    """SampleAggregate に属するアイテムを表すエンティティ。

    同一性は `item_id` で判定される（値が変わっても同一の項目とみなす）。
    可変であってよいのは集約の内側のみであり、状態変更はすべて
    SampleAggregate 経由のコマンドから呼ばれることを前提とする。

    Attributes:
        item_id: このアイテムを一意に識別する id。
        name: アイテムの名前（値オブジェクト）。
    """

    def __init__(self, item_id: str, name: SampleName) -> None:
        self.item_id = item_id
        self.name = name
        self._active = True

    @property
    def active(self) -> bool:
        """アイテムが有効かどうかを返す。"""
        return self._active

    def deactivate(self) -> None:
        """アイテムを無効化する（集約内部からのみ呼ばれる想定）。"""
        self._active = False

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, SampleItem):
            return NotImplemented
        return self.item_id == other.item_id

    def __hash__(self) -> int:
        return hash(self.item_id)
