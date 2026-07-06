"""uc-render-document のguaranteeScenarios(operationGuaranteesと対)に対応する統合テスト。

part_rendererの15件の整形保証はsd-document-engineのdomainServiceScenarios(domain層)へ
再分類済み。ここはrender_engine自体が呼び出し元に約束する保証(決定性・配線・リポジトリ解決契約)
のみを実engine+実adapterで検証する。
"""
from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.render_engine import RenderEngine
from waffle.shared.result import Err, Ok


def _engine() -> RenderEngine:
    return RenderEngine(FsDocumentRepository(), PackageSchemaRepository())


def test_存在しないパスはINVALID_PATH():
    """
    Given 実在しない対象パス
    When 本usecaseを実行する
    Then INVALID_PATHエラーが返る
    """
    result = _engine().run("does/not/exist.json", deploy=False)
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"


def test_解決できないschemaRefはINVALID_SCHEMA_REF():
    """
    Given 解決できないschemaRef
    When 本usecaseを実行する
    Then INVALID_SCHEMA_REFエラーが返る
    """
    import json
    import tempfile

    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump({"documentId": "x", "schemaRef": "NoSuchSchema/v1"}, f)
        path = f.name

    result = _engine().run(path, deploy=False)
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_SCHEMA_REF"


def test_render_engineはschemaのx_render宣言をpart_rendererへ正しく配線する():
    """
    Given interfaceブロック(x-render宣言=table)を持つDocument
    When render engine経由でrenderする(part_rendererを直接呼ばず)
    Then schemaのx-render宣言どおりに整形されたMarkdownテーブルが出力に含まれる

    (domainテストはpart_renderer.render_partsを直接呼ぶため、render_engineが実際に
    schemaからx-render宣言を読み取りpart_rendererへ渡す配線そのものはここでしか検証されない)
    """
    result = _engine().run(".waffle/documents/skills/harness-query-engine.json", deploy=False)
    assert isinstance(result, Ok), result
    assert "| name | type | 必須 | 説明 | 例 |" in result.value["content"]
    assert "| operation | string | ✓" in result.value["content"]


def test_同じDocumentを2回renderしても同一の成果物になる():
    """
    Given 変更されていないDocument
    When 同じDocumentを2回renderする
    Then 1回目と2回目の成果物は同一である
    """
    first = _engine().run(".waffle/documents/skills/harness-query-engine.json", deploy=False)
    second = _engine().run(".waffle/documents/skills/harness-query-engine.json", deploy=False)
    assert isinstance(first, Ok), first
    assert isinstance(second, Ok), second
    assert first.value["content"] == second.value["content"]
