"""agg-document の不変条件（G6/G7: パス解決はプロジェクトルート内に閉じ込められる）が、
実際の engine + 実adapterの組み合わせで正しく配線されていることを検証する統合テスト。

不変条件そのもの（純粋ロジック）は domain/services/path_confinement.py として抽出され、
tests/unit/domain/test_agg_document.py で port 不要のまま検証済み。ここでは
「各engineが is_confined() を正しく呼び、INVALID_PATH へ正しくマッピングしているか」という
配線（実FsDocumentRepository/PackageSchemaRepository/JsonSchemaValidator経由）だけを確認する。
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
