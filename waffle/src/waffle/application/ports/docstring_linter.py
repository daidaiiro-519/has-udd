"""DocstringLinter port — kind ごとに確立された既存 lint ツールを呼び出し、
docstringの構造欠落（Args/Returns/Raisesセクションの有無・引数名の不一致）を
検出する Secondary Port。

docstringの有無（MISSING_DOC_COMMENT）はuc-scan-source-codeの結果（hasDocstring）
から導出するため、このportの責務には含まれない。
"""
from __future__ import annotations

from typing import Protocol


class UnsupportedKind(Exception):
    """アダプタが対応していない kind が指定されたときに送出する。"""


class ToolNotAvailable(Exception):
    """kind に対応する既存lintツールが実行環境に存在しないときに送出する。"""


class DocstringLinter(Protocol):
    def lint(self, target_path: str, kind: str) -> list[dict]:
        """target_path配下をkindの規約に従って検証し、構造違反
        ({path, elementKind, name, code, detail})の配列を返す。
        codeは ARGS_MISMATCH / MISSING_ARGS_SECTION / MISSING_RETURNS_SECTION /
        MISSING_RAISES_SECTION のいずれか。非公開要素は対象外。
        """
        ...
