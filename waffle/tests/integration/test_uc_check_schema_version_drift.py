"""uc-check-schema-version-drift のguaranteeScenarios(operationGuaranteesと対)のうち、
リポジトリ解決契約(対象のdocuments_root)に対応する統合テスト。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.check_schema_version_drift_engine import CheckSchemaVersionDriftEngine
from waffle.shared.result import Err


def _engine() -> CheckSchemaVersionDriftEngine:
    return CheckSchemaVersionDriftEngine(FsDocumentRepository(), PackageSchemaRepository())


def test_存在しないdocuments_rootはINVALID_PATH():
    """
    When 存在しないdocuments_rootでschema版ドリフト検査を実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("does/not/exist")
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"
