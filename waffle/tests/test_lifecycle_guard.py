"""lifecycle_guard（Re-2: schema の x-lifecycle を読む薄い guard）の単体テスト。"""
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


def test_legal_transition_returns_target():
    assert next_status(_SCHEMA, "CREATED", "validate") == "VALIDATED"


def test_illegal_transition_returns_none():
    assert next_status(_SCHEMA, "RENDERED", "validate") is None


def test_terminal_state_has_no_outgoing_transition():
    assert next_status(_SCHEMA, "SUPERSEDED", "validate") is None


def test_schema_without_lifecycle_returns_none():
    assert next_status({}, "ACTIVE", "validate") is None
