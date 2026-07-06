"""lifecycle_guard（Re-2: schema の x-lifecycle を読む薄い guard）の単体テスト。

agg-document(Document集約)の unitTestScenarios に対応するネイティブテスト。
"""
from waffle.domain.services.lifecycle_guard import next_status

_SCHEMA = {
    "x-lifecycle": {
        "transitions": [
            {"from": None, "to": "CREATED", "command": "create"},
            {"from": "CREATED", "to": "VALIDATED", "command": "validate"},
            {"from": "VALIDATED", "to": "RENDERED", "command": "render"},
            {"from": "VALIDATED", "to": "SUPERSEDED", "command": "supersede"},
        ]
    }
}

_CODING_SKILL_SCHEMA = {
    "x-lifecycle": {
        "transitions": [
            {"from": None, "to": "DRAFT", "command": "create"},
            {"from": "DRAFT", "to": "ACTIVE", "command": "activate"},
            {"from": "ACTIVE", "to": "DEPRECATED", "command": "deprecate"},
        ]
    }
}


def test_legal_transition_returns_target():
    assert next_status(_SCHEMA, "CREATED", "validate") == "VALIDATED"


def test_status_は逆行できない():
    """
    Given RENDERED 状態の Document
    When validate へ戻そうとする
    Then 状態遷移は拒否され、状態は RENDERED のままである
    """
    assert next_status(_SCHEMA, "RENDERED", "validate") is None


def test_未検証では_render_できない():
    """
    Given CREATED 状態の Document
    When render する
    Then 拒否され、状態は CREATED のままである
    """
    assert next_status(_SCHEMA, "CREATED", "render") is None


def test_SUPERSEDED_は終端():
    """
    Given SUPERSEDED 状態の Document
    When 任意のコマンドを実行する
    Then 拒否される
    """
    assert next_status(_SCHEMA, "SUPERSEDED", "validate") is None
    assert next_status(_SCHEMA, "SUPERSEDED", "render") is None


def test_Coding_Skill_の_status_も逆行できない():
    """
    Given ACTIVE 状態の Coding document
    When DRAFT へ戻そうとする
    Then 状態遷移は拒否され、状態は ACTIVE のままである
    """
    assert next_status(_CODING_SKILL_SCHEMA, "ACTIVE", "create") is None


def test_DEPRECATED_は終端():
    """
    Given DEPRECATED 状態の Skill document
    When 任意のコマンドを実行する
    Then 拒否される
    """
    assert next_status(_CODING_SKILL_SCHEMA, "DEPRECATED", "activate") is None
    assert next_status(_CODING_SKILL_SCHEMA, "DEPRECATED", "deprecate") is None


def test_schema_without_lifecycle_returns_none():
    assert next_status({}, "ACTIVE", "validate") is None
