"""uc-check-agent-skill-driftのguaranteeScenarios(operationGuaranteesと対)のうち、
リポジトリ解決契約(対象のdocuments_root)に対応する統合テスト。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_agent_skill_drift_engine import CheckAgentSkillDriftEngine
from waffle.shared.result import Err


def _engine() -> CheckAgentSkillDriftEngine:
    return CheckAgentSkillDriftEngine(FsDocumentRepository())


def test_存在しないdocuments_rootはINVALID_PATH():
    """
    When 存在しないdocuments_rootでAgent-Skill整合検査を実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("does/not/exist")
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"
