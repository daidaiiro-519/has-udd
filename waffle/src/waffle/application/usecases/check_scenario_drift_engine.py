"""check scenario drift engine — spec の TestScenarios と、対応するネイティブテスト
ファイルのテスト関数名・docstring内容を突き合わせる application use case。

実行/意味理解はしない（AST解析のみ）。検出した差分の中身の妥当性評価はAIが担う。
"""
from __future__ import annotations

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.services.document_loading import load_document
from waffle.domain.services.scenario_drift import (
    contains_subsequence,
    docstring_lines,
    scenario_gherkins,
    test_function_docstrings,
)
from waffle.shared.path_confinement import is_confined
from waffle.shared.result import Err, Ok, Result


def _err(code: str, message: str) -> Err:
    return Err(message, [code])


class CheckScenarioDriftEngine:
    def __init__(self, documents: DocumentRepository) -> None:
        self._documents = documents

    def run(self, spec_path: str, test_file_path: str) -> Result[dict]:
        spec_loaded = load_document(self._documents, spec_path)
        if isinstance(spec_loaded, Err):
            return spec_loaded
        spec_doc = spec_loaded.value

        if not is_confined(test_file_path):
            return _err("INVALID_PATH", f"パストラバーサルは許可されません: {test_file_path}")
        try:
            source = self._documents.read_text(test_file_path)
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ファイルが見つかりません: {test_file_path}")

        gherkins = scenario_gherkins(spec_doc)
        try:
            test_docstrings = test_function_docstrings(source)
        except SyntaxError:
            return _err("INVALID_SOURCE", f"構文解析できません: {test_file_path}")

        spec_names = set(gherkins.keys())
        test_names = set(test_docstrings.keys())
        matched = spec_names & test_names
        mismatches = [
            name for name in matched
            if not contains_subsequence(docstring_lines(test_docstrings[name]), gherkins[name])
        ]

        return Ok({
            "missing_in_tests": sorted(spec_names - test_names),
            "orphaned_in_tests": sorted(test_names - spec_names),
            "matched": sorted(matched),
            "gherkin_mismatches": sorted(mismatches),
        })
