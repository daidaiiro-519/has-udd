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


def test_SkillSchemaをMarkdownにレンダリングする():
    """
    Given SkillSchemaのDocument
    When renderする
    Then 見出し・目的・パラメータ表・オペレーション選択・呼び出し例が全て出力に含まれる
    """
    result = _engine().run(".waffle/documents/skills/harness-query-engine.json", deploy=False)
    assert isinstance(result, Ok), result
    content = result.value["content"]
    assert "# harness-query-engine" in content
    assert "## 目的" in content
    assert "| name | type | 必須 | 説明 | 例 |" in content
    assert "### Step 2: オペレーションを選ぶ" in content
    assert "| operation | 用途 | 必須引数 | 例 |" in content
    assert "waffle query --operation get_block" in content


def test_frontmatterはx_frontmatterのドットパスを解決して生成する():
    """
    Given x-frontmatterを宣言するSchemaのDocument
    When renderする
    Then 出力冒頭にname/description等を含むYAML frontmatterが生成される
    """
    result = _engine().run(".waffle/documents/skills/harness-query-engine.json", deploy=False)
    assert isinstance(result, Ok), result
    content = result.value["content"]
    assert "name:" in content
    assert "description:" in content
    assert "harness-query-engine" in content


def test_CodingSchemaはMarkdownとして描画できる():
    """
    Given CodingSchemaのDocument
    When renderする
    Then Markdown形式で見出しを含む出力が生成される
    """
    result = _engine().run(".waffle/documents/coding/tech-stack-python-hexagonal.json", deploy=False)
    assert isinstance(result, Ok), result
    assert result.value["format"] == "md"
    assert "# " in result.value["content"]


def test_usecase_Specは基本フローをシーケンス図にTestScenariosをMarkdownに出す():
    """
    Given usecase SpecのDocument
    When renderする
    Then 出力にmermaidのsequenceDiagramとテストシナリオ節が含まれ、feature出力にも同じシナリオが含まれる
    """
    result = _engine().run(
        ".waffle/documents/specs/bc-waffle-engines/subdomain/sd-document-engine/usecase/uc-query-document.json",
        deploy=False,
    )
    assert isinstance(result, Ok), result
    content = result.value["content"]
    assert result.value["format"] == "md"
    assert "# uc-query-document" in content
    assert "sequenceDiagram" in content
    assert "mermaid" in content
    assert "## テストシナリオ" in content
    assert "Scenario: 未知の operation はエラーを返す" in content
    assert "Feature: uc-query-document" in result.value["feature"]
    assert "Scenario: 未知の operation はエラーを返す" in result.value["feature"]


def test_aggregate_Specは集約の構造とライフサイクルをMarkdownに出す():
    """
    Given aggregate SpecのDocument
    When renderする
    Then 出力にコマンド節・ドメインイベント名・mermaidのstateDiagram-v2が含まれる
    """
    result = _engine().run(
        ".waffle/documents/specs/bc-waffle-engines/aggregate/agg-document.json", deploy=False,
    )
    assert isinstance(result, Ok), result
    content = result.value["content"]
    assert "## コマンド" in content
    assert "DocumentRendered" in content
    assert "stateDiagram-v2" in content


def test_不正なJSONはINVALID_JSON():
    """
    Given 不正なJSONの対象ファイル
    When renderする
    Then INVALID_JSONエラーが返る
    """
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        f.write("{not valid json")
        path = f.name

    result = _engine().run(path, deploy=False)
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_JSON"
