"""DocstringLinter port — kind ごとに確立された既存 lint ツールを呼び出し、
docstringの引数名とシグネチャの不一致（ARGS_MISMATCH）を検出する Secondary Port。

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
        """target_path配下をkindの規約に従って検証し、引数名不一致の違反
        ({path, elementKind, name, code:"ARGS_MISMATCH", detail})の配列を返す。
        """
        ...
