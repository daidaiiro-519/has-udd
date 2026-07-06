"""SourceScanner port — ソースコードの公開要素(module/class/function)を構造化抽出する
Secondary Port。kind(DocstringSchemaが定義する言語別docstring規約)ごとに異なる
言語パーサ技術を使うアダプタがこれを実装する。
"""
from __future__ import annotations

from typing import Protocol


class UnsupportedKind(Exception):
    """アダプタが対応していない kind が指定されたときに送出する。"""


class SourceScanner(Protocol):
    def scan(self, source: str, path: str, kind: str) -> list[dict]:
        """source(ソーステキスト)を kind の規約に従って走査し、公開要素ごとの
        構造化抽出結果({path, kind, elementKind, name, hasDocstring, signatureParams,
        summary, body, args, returns, raises, attributes})の配列を返す。
        構文解析できなければ SyntaxError、kind に対応していなければ UnsupportedKind を送出する。
        """
        ...
