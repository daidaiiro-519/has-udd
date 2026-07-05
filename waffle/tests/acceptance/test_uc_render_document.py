"""uc-render-document の受け入れテスト（ネイティブpytest）。

.waffle/specs/.../uc-render-document.feature は参照専用の仕様書であり、実行対象ではない。
"""
import json
import tempfile

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.render_engine import RenderEngine
from waffle.shared.result import Err, Ok


def _engine() -> RenderEngine:
    return RenderEngine(FsDocumentRepository(), PackageSchemaRepository())


def test_検証済み_Document_を成果物に描画する():
    """
    Given 描画対象の Document
    When render する
    Then 成果物が生成され、生成パス一覧が返る
    """
    result = _engine().run(".waffle/documents/skills/harness-query-engine.json", deploy=False)
    assert isinstance(result, Ok), result
    assert result.value["format"] == "md"
    assert "# harness-query-engine" in result.value["content"]


def test_schemaRef_を持たない_Document_は描画しない():
    """
    Given schemaRef の無い Document
    When render する
    Then MISSING_SCHEMA_REF エラーが返る
    """
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump({"documentId": "no-schema-ref"}, f)
        path = f.name

    result = _engine().run(path, deploy=False)
    assert isinstance(result, Err), result
    assert result.details[0] == "MISSING_SCHEMA_REF"


def test_存在しないパスは描画しない():
    """
    When 存在しないパスを対象に render する
    Then INVALID_PATH エラーが返る
    """
    result = _engine().run("does/not/exist.json", deploy=False)
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_PATH"


def test_deploy_すると_canonical_と_deploy_先の両方に書く():
    """
    Given deploy 先を持つ Document
    When deploy を有効にして render する
    Then canonical と deploy 先の両方に成果物が書かれる
    """
    result = _engine().run(".waffle/documents/skills/harness-query-engine.json", deploy=True)
    assert isinstance(result, Ok), result
    assert result.value["path"] == ".waffle/skills/harness-query-engine/SKILL.md"
    assert ".claude/skills/harness-query-engine/SKILL.md" in result.value["deployed"]


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
