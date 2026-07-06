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
        {"blockKey": "acceptanceScenarios", "arrayField": "scenarios", "key": "category", "value": "異常系"},
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
        {"blockKey": "acceptanceScenarios", "arrayField": "scenarios", "key": "category", "value": "存在しないカテゴリ"},
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
        {"blockKey": "acceptanceScenarios", "arrayField": "scenarios", "field": "name", "pattern": "("},
    )
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATTERN"


def test_scanは生テキストを返す():
    """
    Given query engine と対象 Document
    When operation scan を実行する
    Then value は生テキストであり、prompt は null である
    """
    result = _engine().run("scan", _TARGET)
    assert isinstance(result, Ok), result
    assert result.value["prompt"] is None
    assert "documentId" in result.value["value"]


def test_get_metaはメタ情報を返す():
    """
    Given query engine と対象 Document
    When operation get_meta を実行する
    Then value にはdocumentId等のメタフィールドのみが含まれる
    """
    result = _engine().run("get_meta", _TARGET)
    assert isinstance(result, Ok), result
    assert result.value["value"]["documentId"] == "uc-query-document"


def test_index_scanはblockTypeとpromptをschemaから動的算出する():
    """
    Given query engine と対象 Document
    When operation index_scan を実行する
    Then 各blockのblockTypeとx-prompt-query由来のpromptが返る
    """
    result = _engine().run("index_scan", _TARGET)
    assert isinstance(result, Ok), result
    assert result.value["value"]["mainFlow"]["blockType"] == "MainFlow"
    assert result.value["value"]["mainFlow"]["prompt"]


def test_index_scan_dirはディレクトリ横断でindexを集約する():
    """
    Given query engine と対象ディレクトリ
    When operation index_scan_dir を実行する
    Then ディレクトリ配下の各Documentのindexがまとめて返る
    """
    result = _engine().run(
        "index_scan_dir", ".waffle/documents/specs/bc-waffle-engines/subdomain/sd-document-engine/usecase",
    )
    assert isinstance(result, Ok), result
    assert any("uc-query-document.json" in k for k in result.value["value"])


def test_get_fieldはblockの1フィールドを返す():
    """
    Given query engine と対象 Document
    When operation get_field を blockKey, field で実行する
    Then value は指定フィールドの値である
    """
    result = _engine().run("get_field", _TARGET, {"blockKey": "title", "field": "title"})
    assert isinstance(result, Ok), result
    assert result.value["value"] == "uc-query-document"


def test_get_by_idは単一オブジェクトを返す():
    """
    Given query engine と対象 Document
    When operation get_by_id を idField, idValue で実行する
    Then 一致した単一の要素がvalueとして返る（配列ではない）
    """
    result = _engine().run(
        "get_by_id", _TARGET,
        {"blockKey": "acceptanceScenarios", "arrayField": "scenarios", "idField": "name", "idValue": "ブロックを丸ごと取得する"},
    )
    assert isinstance(result, Ok), result
    assert result.value["value"]["name"] == "ブロックを丸ごと取得する"


def test_find_allは全階層を再帰収集する():
    """
    Given query engine と対象 Document
    When operation find_all を fieldName で実行する
    Then 全階層に出現するfieldNameの値がvalueとして返る
    """
    result = _engine().run("find_all", _TARGET, {"fieldName": "category"})
    assert isinstance(result, Ok), result
    assert "異常系" in result.value["value"]


def test_schemaRefを持たないファイルはrawで返す():
    """
    Given schemaRefを持たない対象ファイル
    When 任意のoperationを実行する
    Then valueはtype=rawとして生テキストを返す
    """
    import json
    import tempfile

    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump({"hello": "world"}, f)
        path = f.name

    result = _engine().run("get_meta", path)
    assert isinstance(result, Ok), result
    assert result.value["type"] == "raw"
