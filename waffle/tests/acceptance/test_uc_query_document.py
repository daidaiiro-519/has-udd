"""uc-query-document の受け入れテスト（ネイティブpytest）。

.waffle/specs/.../uc-query-document.feature は参照専用の仕様書であり、実行対象ではない。
このファイルはその .feature の内容を読んで直接実装したもの（手編集前提・render で上書きされない）。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.query_engine import QueryEngine
from waffle.shared.result import Err, Ok

_TARGET = ".waffle/documents/specs/bc-waffle-engines/subdomain/sd-document-engine/usecase/uc-query-document.json"


def _engine() -> QueryEngine:
    return QueryEngine(FsDocumentRepository(), PackageSchemaRepository())


def test_ブロックを丸ごと取得する():
    """
    Given query engine と対象 Document
    When operation get_block を blockKey interface で実行する
    Then value は対象ブロックであり、prompt に読み方の指針が付く
    """
    result = _engine().run("get_block", _TARGET, {"blockKey": "title"})
    assert isinstance(result, Ok), result
    assert result.value["prompt"] is not None


def test_条件に一致する配列要素だけを絞り込む():
    """
    Given query engine と対象 Document
    When operation filter_items で required=true を指定する
    Then value には required な要素だけが含まれる
    """
    result = _engine().run(
        "filter_items", _TARGET,
        {"blockKey": "testScenarios", "arrayField": "scenarios", "key": "category", "value": "異常系"},
    )
    assert isinstance(result, Ok), result
    assert all(item["category"] == "異常系" for item in result.value["value"])


def test_一致が無くても正常系で空配列を返す():
    """
    When 一致しないフィルタ条件で filter_items を実行する
    Then value は空配列で、エラーにはならない
    """
    result = _engine().run(
        "filter_items", _TARGET,
        {"blockKey": "testScenarios", "arrayField": "scenarios", "key": "category", "value": "存在しないカテゴリ"},
    )
    assert isinstance(result, Ok), result
    assert result.value["value"] == []


def test_未知の_operation_はエラーを返す():
    """
    When 未知の operation を実行する
    Then INVALID_OPERATION エラーが返る
    """
    result = _engine().run("unknown_op", _TARGET, {})
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_OPERATION"


def test_存在しないパスはエラーを返す():
    """
    When 存在しないパスを対象に query する
    Then INVALID_PATH エラーが返る
    """
    result = _engine().run("get_block", "does/not/exist.json", {"blockKey": "title"})
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"


def test_必須パラメータの欠落はエラーを返す():
    """
    When blockKey を指定せずに get_block を実行する
    Then MISSING_PARAM エラーが返る
    """
    result = _engine().run("get_block", _TARGET, {})
    assert isinstance(result, Err), result
    assert result.details[0] == "MISSING_PARAM"


def test_存在しないblockKeyはエラーを返す():
    """
    When 存在しない blockKey を指定して get_block を実行する
    Then NOT_FOUND エラーが返る
    """
    result = _engine().run("get_block", _TARGET, {"blockKey": "no_such_block"})
    assert isinstance(result, Err), result
    assert result.details[0] == "NOT_FOUND"


def test_不正な正規表現はエラーを返す():
    """
    When 不正な正規表現で filter_pattern を実行する
    Then INVALID_PATTERN エラーが返る
    """
    result = _engine().run(
        "filter_pattern", _TARGET,
        {"blockKey": "testScenarios", "arrayField": "scenarios", "field": "name", "pattern": "("},
    )
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATTERN"
