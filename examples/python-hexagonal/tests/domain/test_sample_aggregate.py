"""SampleAggregate の不変条件を検証する単体テスト。"""

from __future__ import annotations

import pytest

from example_hex.domain.model.sample_aggregate import (
    SampleAggregate,
    SampleInvariantError,
)
from example_hex.domain.model.sample_name import SampleName
from example_hex.domain.services.sample_domain_service import (
    SampleDomainService,
)


def _new_aggregate() -> SampleAggregate:
    return SampleAggregate(aggregate_id="agg-1", name=SampleName("root"))


def test_add_item_succeeds_with_unique_name() -> None:
    """一意な名前のアイテムは追加できる。"""
    aggregate = _new_aggregate()

    item = aggregate.add_item("item-1", SampleName("apple"))

    assert item.item_id == "item-1"
    assert len(aggregate.items) == 1


def test_add_item_rejects_duplicate_name() -> None:
    """同名アイテムの追加は不変条件違反として拒否される。"""
    aggregate = _new_aggregate()
    aggregate.add_item("item-1", SampleName("apple"))

    with pytest.raises(SampleInvariantError):
        aggregate.add_item("item-2", SampleName("apple"))


def test_add_item_rejects_over_max_items() -> None:
    """最大件数を超えるアイテム追加は不変条件違反として拒否される。"""
    aggregate = _new_aggregate()
    for i in range(5):
        aggregate.add_item(f"item-{i}", SampleName(f"name-{i}"))

    with pytest.raises(SampleInvariantError):
        aggregate.add_item("item-overflow", SampleName("overflow"))


def test_deactivate_item_marks_entity_inactive() -> None:
    """エンティティは集約経由で無効化でき、同一性は id で保たれる。"""
    aggregate = _new_aggregate()
    aggregate.add_item("item-1", SampleName("apple"))

    aggregate.deactivate_item("item-1")

    assert aggregate.items[0].active is False
    assert aggregate.items[0].item_id == "item-1"


def test_deactivate_item_missing_id_raises() -> None:
    """存在しない id の無効化は不変条件違反として拒否される。"""
    aggregate = _new_aggregate()

    with pytest.raises(SampleInvariantError):
        aggregate.deactivate_item("missing")


def test_sample_name_value_equality() -> None:
    """value-object は値そのもので等価性が判定される。"""
    assert SampleName("apple") == SampleName("apple")
    assert SampleName("apple") != SampleName("banana")


def test_sample_name_rejects_blank_value() -> None:
    """value-object は生成時に不変条件（空文字禁止）を強制する。"""
    with pytest.raises(ValueError):
        SampleName("   ")


def test_domain_service_picks_aggregate_with_more_active_items() -> None:
    """domain-service は複数集約を跨いで判断を行う。"""
    left = _new_aggregate()
    left.add_item("l-1", SampleName("l-apple"))
    left.add_item("l-2", SampleName("l-banana"))

    right = SampleAggregate(aggregate_id="agg-2", name=SampleName("root2"))
    right.add_item("r-1", SampleName("r-apple"))

    service = SampleDomainService()
    winner = service.more_active_items(left, right)

    assert winner is left
