"""pydoclint adapter（DocstringLinter実装）。

kind="google"専用。pydoclintを起動し、Args/Returns/Raisesセクションの構造欠落・
引数名不一致を診断コードごとに正規化する。要約行のみの短いdocstringもチェック
対象に含め（--skip-checking-short-docstrings=False）、非公開要素は対象外にする
（--skip-checking-private-functions=True）。他の診断コード（型ヒント関連等）は
本usecaseの対象外のため無視する。
"""
from __future__ import annotations

import re
import subprocess

from waffle.application.ports.docstring_linter import DocstringLinter, ToolNotAvailable, UnsupportedKind

_LINE_RE = re.compile(r"^\s+\d+:\s+DOC(?P<code>\d+):\s+(?:Function|Method|Class)\s+`(?P<name>\w+)`")
_CODE_MAP = {
    "101": "MISSING_ARGS_SECTION",
    "103": "ARGS_MISMATCH",
    "201": "MISSING_RETURNS_SECTION",
    "501": "MISSING_RAISES_SECTION",
}


class PydoclintLinter(DocstringLinter):
    def __init__(self, executable: str = "pydoclint") -> None:
        self._executable = executable

    def lint(self, target_path: str, kind: str) -> list[dict]:
        if kind != "google":
            raise UnsupportedKind(kind)

        args = [
            self._executable,
            "--style", "google",
            "--arg-type-hints-in-signature=False",
            "--arg-type-hints-in-docstring=False",
            "--check-return-types=False",
            "--check-yield-types=False",
            "--skip-checking-short-docstrings=False",
            "--skip-checking-private-functions=True",
            target_path,
        ]
        try:
            proc = subprocess.run(args, capture_output=True, text=True)
        except FileNotFoundError as e:
            raise ToolNotAvailable(self._executable) from e

        # pydoclint はTTYが無い実行(subprocess経由)ではstderrに出力する
        return _parse_output(proc.stderr or proc.stdout)


def _parse_output(stdout: str) -> list[dict]:
    violations: list[dict] = []
    current_file: str | None = None
    for line in stdout.splitlines():
        if line and not line[0].isspace():
            if line.startswith("Skipping") or "No violations" in line:
                continue
            current_file = line.strip()
            continue
        m = _LINE_RE.match(line)
        code = _CODE_MAP.get(m.group("code")) if m else None
        if code:
            violations.append({
                "path": current_file,
                "elementKind": "function",
                "name": m.group("name"),
                "code": code,
                "detail": line.strip(),
            })
    return violations
