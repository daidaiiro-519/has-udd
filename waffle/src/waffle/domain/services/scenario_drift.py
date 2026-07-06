"""scenario_drift — spec の TestScenarios が宣言するシナリオ名・gherkin本文と、
テストコードの test_* 関数名・docstring を突き合わせる純粋なドメインサービス。

test_* 関数名の抽出は ast モジュールのみで行う軽量な実装であり、
uc-scan-source-code の kind 別 docstring 構造化抽出（DocstringSchema）とは
独立している（関数名・docstringを見るだけの目的には過剰結合になるため）。
"""
from __future__ import annotations

import ast
import re

_SCENARIO_BLOCK_KEYS = (
    "acceptanceScenarios",
    "guaranteeScenarios",
    "invariantScenarios",
    "domainServiceScenarios",
)


def sanitize(name: str) -> str:
    """シナリオ名の非単語文字を _ に置換し test_ を前置する。"""
    return f"test_{re.sub(r'[^\w]', '_', name)}"


def gherkin_body(gherkin: str) -> list[str]:
    """gherkin文字列から "Scenario: ..." / "Scenario Outline: ..." 見出し行を除いた
    本文行を、前後の空白を落として返す。"""
    lines = gherkin.strip().splitlines()
    if lines and lines[0].strip().startswith(("Scenario:", "Scenario Outline:")):
        lines = lines[1:]
    return [ln.strip() for ln in lines if ln.strip()]


def scenario_gherkins(spec_doc: dict) -> dict[str, list[str]]:
    """spec document(dict) から sanitize済みシナリオ名 -> gherkin本文行 のマップを作る。"""
    content = spec_doc.get("content", {})
    result: dict[str, list[str]] = {}
    for block_key in _SCENARIO_BLOCK_KEYS:
        block = content.get(block_key)
        if block:
            for s in block["scenarios"]:
                result[sanitize(s["name"])] = gherkin_body(s["gherkin"])
    return result


def test_function_docstrings(source: str) -> dict[str, str]:
    """テストファイルのソーステキストから test_* 関数名 -> docstring(無ければ空文字)
    のマップを返す。構文解析できなければ SyntaxError を送出する。"""
    tree = ast.parse(source)
    return {
        node.name: ast.get_docstring(node) or ""
        for node in ast.walk(tree)
        if isinstance(node, ast.FunctionDef) and node.name.startswith("test_")
    }


def docstring_lines(docstring: str) -> list[str]:
    return [ln.strip() for ln in docstring.splitlines() if ln.strip()]


def contains_subsequence(haystack: list[str], needle: list[str]) -> bool:
    """needle が haystack の中に連続した部分列として（順序通り）出現するか。"""
    if not needle:
        return True
    n = len(needle)
    return any(haystack[i:i + n] == needle for i in range(len(haystack) - n + 1))
