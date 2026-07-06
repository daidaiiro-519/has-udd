"""spec の TestScenarios と、対応するネイティブテストファイルのテスト関数名・
docstring内容を突き合わせ、ドリフト（未実装のシナリオ・孤立したテスト・
Given/When/Then本文の不一致）を検出する。

実行/意味理解はしない（AST解析のみ）。検出した差分の中身の妥当性評価はAIが担う。

名前一致だけでは、シナリオ名を変えずにgherkin本文（Given/When/Then）だけを
書き換えるドリフトを検知できない。これを検知するため、テスト関数のdocstringが
対応するシナリオのgherkin本文（"Scenario: ..."見出しを除いた行）をそのまま
含んでいるかも確認する（test-standardの「Given/When/Thenをdocstringに転記する」
規約に基づく機械検証）。

使い方:
    uv run python scripts/check_scenario_drift.py <spec.json> <test_file.py>
"""
import ast
import json
import re
import sys
from pathlib import Path


def sanitize(name: str) -> str:
    return f"test_{re.sub(r'[^\w]', '_', name)}"


def _gherkin_body(gherkin: str) -> list[str]:
    """gherkin文字列から "Scenario: ..." / "Scenario Outline: ..." 見出し行を除いた
    本文行を、前後の空白を落として返す。"""
    lines = gherkin.strip().splitlines()
    if lines and lines[0].strip().startswith(("Scenario:", "Scenario Outline:")):
        lines = lines[1:]
    return [ln.strip() for ln in lines if ln.strip()]


def _docstring_lines(node: ast.FunctionDef) -> list[str]:
    doc = ast.get_docstring(node) or ""
    return [ln.strip() for ln in doc.splitlines() if ln.strip()]


def _contains_subsequence(haystack: list[str], needle: list[str]) -> bool:
    """needle が haystack の中に連続した部分列として（順序通り）出現するか。"""
    if not needle:
        return True
    n = len(needle)
    for i in range(len(haystack) - n + 1):
        if haystack[i:i + n] == needle:
            return True
    return False


def scenario_gherkins(spec_path: str) -> dict[str, list[str]]:
    """sanitize済みシナリオ名 -> gherkin本文行 のマップ。"""
    doc = json.loads(Path(spec_path).read_text())
    content = doc["content"]
    result: dict[str, list[str]] = {}
    for block_key in ("acceptanceScenarios", "guaranteeScenarios", "invariantScenarios", "domainServiceScenarios"):
        block = content.get(block_key)
        if block:
            for s in block["scenarios"]:
                result[sanitize(s["name"])] = _gherkin_body(s["gherkin"])
    return result


def scenario_names(spec_path: str) -> set[str]:
    return set(scenario_gherkins(spec_path).keys())


def test_function_names(test_file_path: str) -> set[str]:
    tree = ast.parse(Path(test_file_path).read_text())
    return {
        node.name
        for node in ast.walk(tree)
        if isinstance(node, ast.FunctionDef) and node.name.startswith("test_")
    }


def gherkin_mismatches(spec_path: str, test_file_path: str) -> list[str]:
    """名前は一致するが、docstringがspec側のgherkin本文を含んでいないシナリオ名の一覧。"""
    gherkins = scenario_gherkins(spec_path)
    tree = ast.parse(Path(test_file_path).read_text())
    mismatches: list[str] = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.FunctionDef) or node.name not in gherkins:
            continue
        expected = gherkins[node.name]
        actual = _docstring_lines(node)
        if not _contains_subsequence(actual, expected):
            mismatches.append(node.name)
    return sorted(mismatches)


def check(spec_path: str, test_file_path: str) -> dict:
    spec_names = scenario_names(spec_path)
    test_names = test_function_names(test_file_path)
    return {
        "missing_in_tests": sorted(spec_names - test_names),
        "orphaned_in_tests": sorted(test_names - spec_names),
        "matched": sorted(spec_names & test_names),
        "gherkin_mismatches": gherkin_mismatches(spec_path, test_file_path),
    }


if __name__ == "__main__":
    spec_arg, test_arg = sys.argv[1], sys.argv[2]
    result = check(spec_arg, test_arg)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    if result["missing_in_tests"] or result["orphaned_in_tests"] or result["gherkin_mismatches"]:
        sys.exit(1)
