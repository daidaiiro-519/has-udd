"""uc-check-error-code-driftのguaranteeScenarios(operationGuaranteesと対)のうち、
リポジトリ解決契約(対象のspecs_root/code_root)に対応する統合テスト。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_error_code_drift_engine import CheckErrorCodeDriftEngine
from waffle.shared.result import Err


def _engine() -> CheckErrorCodeDriftEngine:
    return CheckErrorCodeDriftEngine(FsDocumentRepository())


def test_存在しないspecs_rootはINVALID_PATH():
    """
    When 存在しないspecs_rootでエラーコード整合検査を実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("does/not/exist", "src")
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"
