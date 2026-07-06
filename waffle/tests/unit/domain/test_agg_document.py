"""agg-document（Document集約）の invariantScenarios に対応するネイティブテスト。

lifecycle_guard（schema の x-lifecycle を読む薄い guard）・path_confinement
（パス解決の不変条件を実装する、業務ロジックを含まない汎用ユーティリティ）・
require_schema_ref（既にロード済みのDocumentのschemaRef有無を見るだけの純粋な判定）
経由で実証する。
"""
from waffle.domain.services.lifecycle_guard import next_status
from waffle.domain.services.schema_ref_guard import require_schema_ref
from waffle.shared.path_confinement import is_confined
from waffle.shared.result import Err

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


# --- パス解決はプロジェクトルート内に閉じ込められる（G6/G7） ---

def test_パストラバーサルを含むパスは拒否される():
    """
    Given '..' を含む対象パス
    When 任意の operation・command を実行する
    Then INVALID_PATH エラーが返り、プロジェクトルート外へはアクセスしない
    """
    assert is_confined("docs/../../etc/passwd") is False
    assert is_confined("docs/valid.json") is True


def test_ディレクトリ横断はプロジェクトルート外を拒否する():
    """
    Given プロジェクトルート外を指すディレクトリパス
    When index_scan_dir を実行する
    Then INVALID_PATH エラーが返る
    """
    assert is_confined("../outside") is False


def test_schemaRefを持たないDocumentはMISSING_SCHEMA_REFとして拒否される():
    """
    Given schemaRef を持たない Document
    When schema 解決を要する operation・command を実行する
    Then MISSING_SCHEMA_REF エラーが返る
    """
    result = require_schema_ref({"name": "x"})
    assert isinstance(result, Err)
    assert result.details[0] == "MISSING_SCHEMA_REF"
