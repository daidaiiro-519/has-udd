"""uc-check-agent-skill-drift の受け入れテスト（ネイティブpytest）。"""
import json
from pathlib import Path

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_agent_skill_drift_engine import CheckAgentSkillDriftEngine
from waffle.shared.result import Ok


def _engine() -> CheckAgentSkillDriftEngine:
    return CheckAgentSkillDriftEngine(FsDocumentRepository())


def _write(path: Path, doc: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(doc, ensure_ascii=False), encoding="utf-8")


def _subagent(skill_ids: list[str]) -> dict:
    return {
        "documentId": "subagent-x",
        "agentKind": "subagent",
        "content": {"skillPreloads": {"blockType": "SkillPreloads", "title": "プリロードSkill", "items": skill_ids, "guidance": "使う"}},
    }


def _skill(manual_only: bool) -> dict:
    return {
        "documentId": "skill-x",
        "skillKind": "custom",
        "content": {"invocationMode": {"blockType": "InvocationMode", "title": "呼び出しモード", "manualOnly": manual_only}},
    }


def test_agentディレクトリがまだ無いとき空配列を返す(tmp_path):
    """
    Given documents_rootは実在するがagentサブディレクトリがまだ無い（Agent documentが1件も無い）
    When Agent-Skill整合検査を実行する
    Then missing_skills・unpreloadable_skills両方が空配列で返る（エラーにしない）
    """
    result = _engine().run(str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value == {"missing_skills": [], "unpreloadable_skills": []}


def test_全subagentの参照が実在しプリロード可能なとき差分なしと判定する(tmp_path):
    """
    Given 全subagentのskillPreloads.itemsが、実在しmanualOnlyでないSkillを参照しているspecツリー
    When Agent-Skill整合検査を実行する
    Then missing_skills・unpreloadable_skills両方が空配列で返る
    """
    _write(tmp_path / "agent" / "subagent-x.json", _subagent(["skill-x"]))
    _write(tmp_path / "skills" / "skill-x.json", _skill(manual_only=False))

    result = _engine().run(str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value == {"missing_skills": [], "unpreloadable_skills": []}


def test_実在しないskillIdを参照するsubagentを検出する(tmp_path):
    """
    Given 実在しないskillIdをskillPreloads.itemsに持つsubagent
    When Agent-Skill整合検査を実行する
    Then missing_skillsにその組が含まれる
    """
    _write(tmp_path / "agent" / "subagent-x.json", _subagent(["no-such-skill"]))

    result = _engine().run(str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["missing_skills"] == [
        {"agent": str(tmp_path / "agent" / "subagent-x.json"), "skillId": "no-such-skill"}
    ]


def test_manualOnlyのSkillをプリロード指定しているsubagentを検出する(tmp_path):
    """
    Given invocationMode.manualOnly=trueのSkillをskillPreloads.itemsに持つsubagent
    When Agent-Skill整合検査を実行する
    Then unpreloadable_skillsにその組が含まれる
    """
    _write(tmp_path / "agent" / "subagent-x.json", _subagent(["skill-x"]))
    skill_path = tmp_path / "skills" / "skill-x.json"
    _write(skill_path, _skill(manual_only=True))

    result = _engine().run(str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["unpreloadable_skills"] == [
        {"agent": str(tmp_path / "agent" / "subagent-x.json"), "skillId": "skill-x", "skillPath": str(skill_path)}
    ]
