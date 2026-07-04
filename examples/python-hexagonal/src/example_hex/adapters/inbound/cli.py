"""CLI（driving/inbound adapter）と合成ルートを定義する。

外部入力（コマンドライン引数）を application usecase の呼び出しへ
変換するだけで、判断ロジックは持たない。依存の生成（DI）は
この起動点にのみ置く（合成ルート）。
"""

from __future__ import annotations

import typer

from example_hex.adapters.outbound.in_memory_sample_repository import (
    InMemorySampleRepository,
)
from example_hex.adapters.outbound.system_clock import SystemClock
from example_hex.application.usecases.do_something import DoSomethingUsecase

app = typer.Typer(help="python-hexagonal サンプル CLI")


@app.command("do-something")
def do_something(
    name: str = typer.Option(..., "--name", help="追加するアイテムの名前"),
    item_id: str = typer.Option(
        "item-1", "--item-id", help="追加するアイテムの id"
    ),
) -> None:
    """DoSomething usecase を実行し、結果を標準出力へ表示する。

    合成ルートとして repository / clock の実装を生成し、
    DoSomethingUsecase へコンストラクタ注入してから実行する。

    Args:
        name: 追加するアイテムの名前（CLI オプション --name）。
        item_id: 追加するアイテムの id（CLI オプション --item-id）。
    """
    repository = InMemorySampleRepository()
    clock = SystemClock()
    usecase = DoSomethingUsecase(repository=repository, clock=clock)

    result = usecase.execute(item_id=item_id, item_name=name)

    if result.is_ok:
        typer.echo(result.value)
        return
    typer.echo(f"エラー[{result.code}]: {result.message}")
    raise typer.Exit(code=1)


@app.command("ping")
def ping() -> None:
    """CLI の疎通確認用コマンド（結線確認のためのユーティリティ）。

    2つ目のサブコマンドを用意し、`do-something` がサブコマンド名として
    呼び出せる形（typer の単一コマンド集約を避ける）を保つ。
    """
    typer.echo("pong")


def main() -> None:
    """CLI アプリケーションのエントリポイント。"""
    app()


if __name__ == "__main__":
    main()
