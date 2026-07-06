"""uc-check-spec-integrity のguaranteeScenarios(operationGuaranteesと対)のうち、
リポジトリ解決契約(対象のbc.json)に対応する統合テスト。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_spec_integrity_engine import CheckSpecIntegrityEngine
from waffle.shared.result import Err


def _engine() -> CheckSpecIntegrityEngine:
    return CheckSpecIntegrityEngine(FsDocumentRepository())


def test_存在しないbc_jsonはINVALID_PATH():
    """
    When 存在しないbc.jsonのパスで参照整合性検査を実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("does/not/exist.json", ".waffle/documents")
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"
