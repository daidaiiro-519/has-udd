"""DoSomethingUsecase の単体テスト（port はテストダブルで代替する）。"""

from __future__ import annotations

from example_hex.application.ports.clock_port import ClockPort
from example_hex.application.ports.sample_repository_port import (
    SampleRepositoryPort,
)
from example_hex.application.usecases.do_something import DoSomethingUsecase
from example_hex.domain.model.sample_aggregate import SampleAggregate


class _FakeRepository(SampleRepositoryPort):
    """テスト用のインメモリ repository ダブル（実物の DB に依存しない）。"""

    def __init__(self) -> None:
        self.saved: SampleAggregate | None = None

    def load(self, aggregate_id: str) -> SampleAggregate | None:
        return self.saved

    def save(self, aggregate: SampleAggregate) -> None:
        self.saved = aggregate


class _FixedClock(ClockPort):
    """常に固定値を返すテスト用 clock ダブル。"""

    def now_iso(self) -> str:
        return "2026-01-01T00:00:00+00:00"


def test_execute_adds_item_and_returns_ok() -> None:
    """正常系: アイテム追加が成功すると Ok が返る。"""
    usecase = DoSomethingUsecase(
        repository=_FakeRepository(), clock=_FixedClock()
    )

    result = usecase.execute(item_id="item-1", item_name="apple")

    assert result.is_ok
    assert "apple" in result.value
    assert "2026-01-01T00:00:00+00:00" in result.value


def test_execute_persists_via_repository_port() -> None:
    """成功時は repository port 経由で集約が保存される。"""
    repository = _FakeRepository()
    usecase = DoSomethingUsecase(repository=repository, clock=_FixedClock())

    usecase.execute(item_id="item-1", item_name="apple")

    assert repository.saved is not None
    assert len(repository.saved.items) == 1


def test_execute_returns_err_on_duplicate_name() -> None:
    """異常系: ドメイン例外は握り潰されず、code 付き Err へ写像される。"""
    repository = _FakeRepository()
    usecase = DoSomethingUsecase(repository=repository, clock=_FixedClock())
    usecase.execute(item_id="item-1", item_name="apple")

    result = usecase.execute(item_id="item-2", item_name="apple")

    assert not result.is_ok
    assert result.code == "DUPLICATE_ITEM_NAME"
    assert "apple" in result.message


def test_execute_returns_err_on_blank_name() -> None:
    """異常系: value-object の不変条件違反も、識別可能なコードを伴う
    Err へ写像される。
    """
    usecase = DoSomethingUsecase(
        repository=_FakeRepository(), clock=_FixedClock()
    )

    result = usecase.execute(item_id="item-1", item_name="   ")

    assert not result.is_ok
    assert result.code == "INVALID_NAME"


def test_execute_returns_err_on_item_limit_exceeded() -> None:
    """異常系: 最大件数超過も ITEM_LIMIT_EXCEEDED を伴う Err へ写像される。"""
    repository = _FakeRepository()
    usecase = DoSomethingUsecase(repository=repository, clock=_FixedClock())
    for i in range(5):
        usecase.execute(item_id=f"item-{i}", item_name=f"name-{i}")

    result = usecase.execute(item_id="item-6", item_name="overflow")

    assert not result.is_ok
    assert result.code == "ITEM_LIMIT_EXCEEDED"
