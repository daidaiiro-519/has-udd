"""agg-document の不変条件（G6/G7: パス解決はプロジェクトルート内に閉じ込められる）を検証する
unitレベルのネイティブテスト。

.waffle/specs/.../agg-document.feature（unitTestScenarios）は参照専用の仕様書であり、実行対象ではない。
この不変条件は Document 集約が扱う全ての operation・command に横断的に適用されるため、
特定usecaseの受け入れテストではなく、集約レベルのunitテストとしてここに置く。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.query_engine import QueryEngine
from waffle.application.usecases.render_engine import RenderEngine
from waffle.application.usecases.validate_engine import ValidateEngine
from waffle.shared.result import Err


def test_パストラバーサルを含むパスは拒否される():
    """
    Given '..' を含む対象パス
    When 任意の operation・command を実行する
    Then INVALID_PATH エラーが返り、プロジェクトルート外へはアクセスしない
    """
    malicious_path = "../etc/passwd.json"

    q = QueryEngine(FsDocumentRepository(), PackageSchemaRepository()).run("scan", malicious_path)
    assert isinstance(q, Err), q
    assert q.details[0] == "INVALID_PATH"

    r = RenderEngine(FsDocumentRepository(), PackageSchemaRepository()).run(malicious_path, deploy=False)
    assert isinstance(r, Err), r
    assert r.details[0] == "INVALID_PATH"

    v = ValidateEngine(
        FsDocumentRepository(), PackageSchemaRepository(), JsonSchemaValidator()
    ).run(malicious_path)
    assert isinstance(v, Err), v
    assert v.details[0] == "INVALID_PATH"


def test_ディレクトリ横断はプロジェクトルート外を拒否する():
    """
    Given プロジェクトルート外を指すディレクトリパス
    When index_scan_dir を実行する
    Then INVALID_PATH エラーが返る
    """
    result = QueryEngine(FsDocumentRepository(), PackageSchemaRepository()).run(
        "index_scan_dir", "/etc"
    )
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"
