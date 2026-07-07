"""lint docstring engine — 対象コードベースの docstring が規約どおりの構造か
（必須セクションの有無・引数名と実シグネチャの整合）を、既存 lint ツールを
呼び出して判定する application use case。自前の照合ロジックは持たない。

MISSING_DOC_COMMENT は uc-scan-source-code の結果（hasDocstring）から導出し、
ARGS_MISMATCH は DocstringLinter port（kind ごとの既存lintツール）に委ねる。

@spec uc-lint-docstring
"""
from __future__ import annotations

from waffle.application.ports.docstring_linter import DocstringLinter, ToolNotAvailable
from waffle.application.ports.docstring_linter import UnsupportedKind as LinterUnsupportedKind
from waffle.application.usecases.scan_source_code_engine import ScanSourceCodeEngine
from waffle.shared.result import Err, Ok, Result


def _err(code: str, message: str) -> Err:
    return Err(message, [code])


class LintDocstringEngine:
    def __init__(self, scan_engine: ScanSourceCodeEngine, linter: DocstringLinter) -> None:
        self._scan_engine = scan_engine
        self._linter = linter

    def run(self, target_path: str, kind: str) -> Result[list[dict]]:
        scan_result = self._scan_engine.run(target_path, kind)
        if isinstance(scan_result, Err):
            return scan_result

        violations = [
            {
                "path": e["path"], "elementKind": e["elementKind"], "name": e["name"],
                "code": "MISSING_DOC_COMMENT", "detail": "docstringがありません",
            }
            for e in scan_result.value
            if not e["hasDocstring"] and e["elementKind"] != "module"
        ]

        try:
            violations.extend(self._linter.lint(target_path, kind))
        except LinterUnsupportedKind:
            return _err("UNSUPPORTED_KIND", f"対応していないkindです: {kind}")
        except ToolNotAvailable as e:
            return _err("TOOL_NOT_AVAILABLE", f"lintツールが実行環境にありません: {e}")

        return Ok(violations)
