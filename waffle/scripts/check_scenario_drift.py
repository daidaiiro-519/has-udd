"""spec の TestScenarios と、対応するネイティブテストファイルのテスト関数名を突き合わせ、
ドリフト（未実装のシナリオ・孤立したテスト）を検出する。

実行/意味理解はしない（AST解析のみ）。検出した差分の中身の妥当性評価はAIが担う。

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


def scenario_names(spec_path: str) -> set[str]:
    doc = json.loads(Path(spec_path).read_text())
    content = doc["content"]
    names: set[str] = set()
    for block_key in ("testScenarios", "guaranteeScenarios"):
        block = content.get(block_key)
        if block:
            names |= {sanitize(s["name"]) for s in block["scenarios"]}
    return names


def test_function_names(test_file_path: str) -> set[str]:
    tree = ast.parse(Path(test_file_path).read_text())
    return {
        node.name
        for node in ast.walk(tree)
        if isinstance(node, ast.FunctionDef) and node.name.startswith("test_")
    }


def check(spec_path: str, test_file_path: str) -> dict:
    spec_names = scenario_names(spec_path)
    test_names = test_function_names(test_file_path)
    return {
        "missing_in_tests": sorted(spec_names - test_names),
        "orphaned_in_tests": sorted(test_names - spec_names),
        "matched": sorted(spec_names & test_names),
    }


if __name__ == "__main__":
    spec_arg, test_arg = sys.argv[1], sys.argv[2]
    result = check(spec_arg, test_arg)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    if result["missing_in_tests"] or result["orphaned_in_tests"]:
        sys.exit(1)
