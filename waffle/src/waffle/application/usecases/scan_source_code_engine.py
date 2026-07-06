"""scan source code engine — 対象コードベース(ディレクトリ)から公開要素の docstring を
kind 別に構造化抽出する application use case。

SourceScanner port（kind ごとの言語パーサ技術を隠蔽する Secondary Port）を編成する。
"""
from __future__ import annotations

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.source_scanner import SourceScanner, UnsupportedKind
from waffle.shared.path_confinement import is_confined
from waffle.shared.result import Err, Ok, Result


def _err(code: str, message: str) -> Err:
    return Err(message, [code])


class ScanSourceCodeEngine:
    def __init__(self, documents: DocumentRepository, scanner: SourceScanner) -> None:
        self._documents = documents
        self._scanner = scanner

    def run(self, target_path: str, kind: str) -> Result[list[dict]]:
        if not is_confined(target_path):
            return _err("INVALID_PATH", f"パストラバーサルは許可されません: {target_path}")
        try:
            files = self._documents.list_files(target_path, "*.py")
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ディレクトリが見つかりません: {target_path}")

        elements: list[dict] = []
        for path in files:
            source = self._documents.read_text(path)
            try:
                elements.extend(self._scanner.scan(source, path, kind))
            except UnsupportedKind:
                return _err("UNSUPPORTED_KIND", f"対応していないkindです: {kind}")
            except SyntaxError:
                return _err("INVALID_SOURCE", f"構文解析できません: {path}")

        return Ok(elements)
