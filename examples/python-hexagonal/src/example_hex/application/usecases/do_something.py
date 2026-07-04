"""DoSomething usecase（application service）を定義する。"""

from __future__ import annotations

from example_hex.application.ports.clock_port import ClockPort
from example_hex.application.ports.sample_repository_port import (
    SampleRepositoryPort,
)
from example_hex.domain.model.sample_aggregate import (
    SampleAggregate,
    SampleInvariantError,
)
from example_hex.domain.model.sample_name import SampleName
from example_hex.shared.result import Err, Ok, Result

_DEFAULT_AGGREGATE_ID = "sample-aggregate"
INVALID_NAME = "INVALID_NAME"


class DoSomethingUsecase:
    """SampleAggregate にアイテムを追加する一連の処理を調整する usecase。

    エントリメソッドは `execute` の1つのみ。ドメインへは repository
    port 経由でのみアクセスし、外部ライブラリへは直接依存しない。
    """

    def __init__(
        self, repository: SampleRepositoryPort, clock: ClockPort
    ) -> None:
        """依存をコンストラクタ注入で受け取る。

        Args:
            repository: SampleAggregate の load/save を行う port。
            clock: 現在時刻を取得する port。
        """
        self._repository = repository
        self._clock = clock

    def execute(self, item_id: str, item_name: str) -> Result[str]:
        """アグリゲートへアイテムを1件追加し、結果メッセージを返す。

        アグリゲートが存在しない場合は新規作成してから追加する。
        ドメイン例外（不変条件違反）は握り潰さず、識別可能なエラーコード
        を伴う Err へ写像する。

        Args:
            item_id: 追加するアイテムの id。
            item_name: 追加するアイテムの名前。

        Returns:
            成功時は Ok(メッセージ)、失敗時は Err(code, message)。
            code は SampleInvariantError の code、または名前が空文字の場合は
            INVALID_NAME。
        """
        aggregate = self._repository.load(_DEFAULT_AGGREGATE_ID)
        if aggregate is None:
            aggregate = SampleAggregate(
                aggregate_id=_DEFAULT_AGGREGATE_ID,
                name=SampleName("default"),
            )

        try:
            name = SampleName(item_name)
        except ValueError as exc:
            return Err(INVALID_NAME, str(exc))

        try:
            aggregate.add_item(item_id=item_id, item_name=name)
        except SampleInvariantError as exc:
            return Err(exc.code, exc.message)

        self._repository.save(aggregate)
        timestamp = self._clock.now_iso()
        return Ok(
            f"'{item_name}' を追加した（{timestamp}）。"
            f"現在のアイテム数: {len(aggregate.items)}"
        )
