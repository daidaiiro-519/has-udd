"""uc-render-document の受け入れテスト（ネイティブpytest）。"""
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
    result = _engine().run(".waffle/documents/skills/tech-lead-advisor.json", deploy=False)
    assert isinstance(result, Ok), result
    assert result.value["format"] == "md"
    assert "# tech-lead-advisor" in result.value["content"]


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



def test_deploy_すると_canonical_と_deploy_先の両方に書く():
    """
    Given deploy 先を持つ Document
    When deploy を有効にして render する
    Then canonical と deploy 先の両方に成果物が書かれる
    """
    result = _engine().run(".waffle/documents/skills/tech-lead-advisor.json", deploy=True)
    assert isinstance(result, Ok), result
    assert result.value["path"] == ".waffle/skills/tech-lead-advisor/SKILL.md"
    assert ".claude/skills/tech-lead-advisor/SKILL.md" in result.value["deployed"]


class _FakeSchemaRepository:
    def __init__(self, schema: dict) -> None:
        self._schema = schema

    def load(self, schema_ref: str) -> dict:
        return self._schema

    def list_versions(self, name: str) -> list[str]:
        return []


def test_discriminatorごとに異なるdeploy先へ書き分ける(tmp_path):
    """
    Given deploy先がdiscriminatorの値ごとに異なる配列として宣言されたschemaのDocument
    When deployを有効にしてrenderする
    Then そのDocumentのdiscriminator値に対応する配列のdeploy先だけに書かれる
    """
    schema = {
        "if": {"properties": {"kind": {"const": "a"}}},
        "properties": {"content": {"type": "object", "properties": {}}},
        "x-render-target": {
            "formats": ["md"],
            "path": {"a": str(tmp_path / "canonical-a.md"), "b": str(tmp_path / "canonical-b.md")},
            "deploy": {"a": [str(tmp_path / "deploy-a.md")], "b": [str(tmp_path / "deploy-b.md")]},
        },
    }
    doc_path = tmp_path / "doc.json"
    doc_path.write_text(
        json.dumps({"documentId": "x", "schemaRef": "Fake/v1", "kind": "a", "content": {}}),
        encoding="utf-8",
    )

    engine = RenderEngine(FsDocumentRepository(), _FakeSchemaRepository(schema))
    result = engine.run(str(doc_path), deploy=True)
    assert isinstance(result, Ok), result
    assert result.value["path"] == str(tmp_path / "canonical-a.md")
    assert str(tmp_path / "deploy-a.md") in result.value["deployed"]
    assert str(tmp_path / "deploy-b.md") not in result.value["deployed"]


def test_pathVarsで宣言したcontent値をパステンプレートの変数として使う(tmp_path):
    """
    Given x-render-target.pathVarsでcontentのドットパスを宣言したschemaのDocument
    When renderする
    Then そのcontent値がパステンプレートの変数として解決され、対応するパスに書かれる
    """
    schema = {
        "properties": {"content": {"type": "object", "properties": {}}},
        "x-render-target": {
            "formats": ["md"],
            "pathVars": {"scope": "doc.content.scope.path"},
            "path": str(tmp_path / "{scope}-canonical.md"),
            "deploy": [str(tmp_path / "{scope}-deploy.md")],
        },
    }
    doc_path = tmp_path / "doc.json"
    doc_path.write_text(
        json.dumps({
            "documentId": "x", "schemaRef": "Fake/v1",
            "content": {"scope": {"blockType": "Scope", "title": "scope", "path": "waffle"}},
        }),
        encoding="utf-8",
    )

    engine = RenderEngine(FsDocumentRepository(), _FakeSchemaRepository(schema))
    result = engine.run(str(doc_path), deploy=True)
    assert isinstance(result, Ok), result
    assert result.value["path"] == str(tmp_path / "waffle-canonical.md")
    assert str(tmp_path / "waffle-deploy.md") in result.value["deployed"]


def test_discriminatorごとに異なるpathVarsを解決する(tmp_path):
    """
    Given discriminatorの値ごとに異なるpathVars宣言（kindごとの変数マップ）を持つschemaのDocument
    When renderする
    Then そのDocumentのdiscriminator値に対応する変数マップだけが解決され、パステンプレートに反映される
    """
    schema = {
        "if": {"properties": {"kind": {"const": "orchestrator"}}},
        "properties": {"content": {"type": "object", "properties": {}}},
        "x-render-target": {
            "formats": ["md"],
            "pathVars": {
                "orchestrator": {"scope": "doc.content.scope.path"},
                "subagent": {},
            },
            "path": str(tmp_path / "{documentId}.md"),
            "deploy": {
                "orchestrator": [str(tmp_path / "{scope}-CLAUDE.md")],
                "subagent": [str(tmp_path / "agents" / "{documentId}.md")],
            },
        },
    }
    doc_path = tmp_path / "doc.json"
    doc_path.write_text(
        json.dumps({
            "documentId": "x", "schemaRef": "Fake/v1", "kind": "orchestrator",
            "content": {"scope": {"blockType": "Scope", "title": "scope", "path": "waffle"}},
        }),
        encoding="utf-8",
    )

    engine = RenderEngine(FsDocumentRepository(), _FakeSchemaRepository(schema))
    result = engine.run(str(doc_path), deploy=True)
    assert isinstance(result, Ok), result
    assert str(tmp_path / "waffle-CLAUDE.md") in result.value["deployed"]

    doc_path2 = tmp_path / "doc2.json"
    doc_path2.write_text(
        json.dumps({"documentId": "y", "schemaRef": "Fake/v1", "kind": "subagent", "content": {}}),
        encoding="utf-8",
    )
    result2 = engine.run(str(doc_path2), deploy=True)
    assert isinstance(result2, Ok), result2
    assert str(tmp_path / "agents" / "y.md") in result2.value["deployed"]


def test_discriminatorごとに異なるx_frontmatterを生成する(tmp_path):
    """
    Given discriminatorの値ごとに異なるx-frontmatter宣言（kindごとのフィールドマップ）を持つschemaのDocument
    When renderする
    Then そのDocumentのdiscriminator値に対応するフィールドマップだけからfrontmatterが生成される
    """
    schema = {
        "if": {"properties": {"kind": {"const": "subagent"}}},
        "properties": {"content": {"type": "object", "properties": {}}},
        "x-frontmatter": {
            "subagent": {"name": "doc.documentId", "description": "doc.content.description.text"},
            "orchestrator": {},
        },
        "x-render-target": {"formats": ["md"], "path": str(tmp_path / "{documentId}.md")},
    }
    doc_path = tmp_path / "doc.json"
    doc_path.write_text(
        json.dumps({
            "documentId": "x", "schemaRef": "Fake/v1", "kind": "subagent",
            "content": {"description": {"blockType": "Description", "title": "d", "text": "when to use"}},
        }),
        encoding="utf-8",
    )

    engine = RenderEngine(FsDocumentRepository(), _FakeSchemaRepository(schema))
    result = engine.run(str(doc_path), deploy=False)
    assert isinstance(result, Ok), result
    assert 'name: "x"' in result.value["content"]
    assert 'description: "when to use"' in result.value["content"]

    doc_path2 = tmp_path / "doc2.json"
    doc_path2.write_text(
        json.dumps({"documentId": "y", "schemaRef": "Fake/v1", "kind": "orchestrator", "content": {}}),
        encoding="utf-8",
    )
    result2 = engine.run(str(doc_path2), deploy=False)
    assert isinstance(result2, Ok), result2
    assert "---" not in result2.value["content"]


def test_存在しないx_frontmatterのドットパスは省略する(tmp_path):
    """
    Given x-frontmatterが宣言するドットパスに対応するcontentブロックを持たない、または値が空であるDocument
    When renderする
    Then そのフィールドはfrontmatterから省略される
    """
    schema = {
        "properties": {"content": {"type": "object", "properties": {}}},
        "x-frontmatter": {
            "name": "doc.documentId",
            "model": "doc.content.runtimeConfig.model",
            "permissionMode": "doc.content.runtimeConfig.permissionMode",
        },
        "x-render-target": {"formats": ["md"], "path": str(tmp_path / "{documentId}.md")},
    }
    doc_path = tmp_path / "doc.json"
    doc_path.write_text(
        json.dumps({
            "documentId": "x", "schemaRef": "Fake/v1",
            "content": {"runtimeConfig": {"blockType": "RuntimeConfig", "title": "t", "permissionMode": ""}},
        }),
        encoding="utf-8",
    )

    engine = RenderEngine(FsDocumentRepository(), _FakeSchemaRepository(schema))
    result = engine.run(str(doc_path), deploy=False)
    assert isinstance(result, Ok), result
    assert 'name: "x"' in result.value["content"]
    assert "model" not in result.value["content"]
    assert "permissionMode" not in result.value["content"]




def test_SkillSchemaをMarkdownにレンダリングする():
    """
    Given SkillSchemaのDocument
    When renderする
    Then 見出し・目的・相談種別テーブル・実行手順・参照knowledgeが全て出力に含まれる
    """
    result = _engine().run(".waffle/documents/skills/tech-lead-advisor.json", deploy=False)
    assert isinstance(result, Ok), result
    content = result.value["content"]
    assert "# tech-lead-advisor" in content
    assert "## 目的" in content
    assert "| 相談種別 | 判定条件 | テンプレート |" in content
    assert "### Step 1: サブドメイン分類を確認する" in content
    assert "## 参照knowledge" in content


def test_frontmatterはx_frontmatterのドットパスを解決して生成する():
    """
    Given x-frontmatterを宣言するSchemaのDocument
    When renderする
    Then 出力冒頭にname/description等を含むYAML frontmatterが生成される
    """
    result = _engine().run(".waffle/documents/skills/tech-lead-advisor.json", deploy=False)
    assert isinstance(result, Ok), result
    content = result.value["content"]
    assert "name:" in content
    assert "description:" in content
    assert "tech-lead-advisor" in content


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


def test_usecase_Specは基本フローをシーケンス図に受け入れシナリオをMarkdownに出す():
    """
    Given usecase SpecのDocument
    When renderする
    Then 出力にmermaidのsequenceDiagramとテストシナリオ節が含まれる
    """
    result = _engine().run(
        ".waffle/documents/specs/bc-waffle/subdomain/sd-document-management/usecase/uc-query-document.json",
        deploy=False,
    )
    assert isinstance(result, Ok), result
    content = result.value["content"]
    assert result.value["format"] == "md"
    assert "# uc-query-document" in content
    assert "sequenceDiagram" in content
    assert "mermaid" in content
    assert "## 受け入れシナリオ" in content
    assert "Scenario: 未知の operation はエラーを返す" in content


def test_aggregate_Specは集約の構造とライフサイクルをMarkdownに出す():
    """
    Given aggregate SpecのDocument
    When renderする
    Then 出力にコマンド節・ドメインイベント名・mermaidのstateDiagram-v2が含まれる
    """
    result = _engine().run(
        ".waffle/documents/specs/bc-waffle/aggregate/agg-document.json", deploy=False,
    )
    assert isinstance(result, Ok), result
    content = result.value["content"]
    assert "## コマンド" in content
    assert "DocumentRendered" in content
    assert "stateDiagram-v2" in content


def test_未検証ではrenderできない():
    """
    Given schemaがrenderをVALIDATED起点の遷移として宣言しているのに、CREATED状態のDocument
    When renderする
    Then INVALID_TRANSITIONエラーが返り、成果物は書き出されない
    """
    doc = {
        "documentId": "unvalidated-spec",
        "documentType": "DomainSpec",
        "schemaRef": "DomainSpecSchema/v2",
        "specKind": "bounded-context",
        "status": "CREATED",
        "content": {},
    }
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(doc, f)
        path = f.name

    result = _engine().run(path, deploy=False)
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_TRANSITION"


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
