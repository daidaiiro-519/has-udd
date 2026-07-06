"""uc-check-scenario-drift のguaranteeScenarios(operationGuaranteesと対)のうち、
リポジトリ解決契約(対象のspec.json・テストファイル)に対応する統合テスト。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_scenario_drift_engine import CheckScenarioDriftEngine
from waffle.shared.result import Err


def _engine() -> CheckScenarioDriftEngine:
    return CheckScenarioDriftEngine(FsDocumentRepository())


def test_存在しないspec_jsonはINVALID_PATH():
    """
    When 存在しないspec.jsonのパスでドリフト検査を実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("does/not/exist.json", "tests/integration/test_uc_check_scenario_drift.py")
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"


def test_存在しないテストファイルはINVALID_PATH():
    """
    When 存在しないテストファイルのパスでドリフト検査を実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run(
        ".waffle/documents/specs/bc-waffle-engines/subdomain/sd-reconciliation/usecase/uc-check-scenario-drift.json",
        "does/not/exist.py",
    )
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"
