"""部品レンダラ（uc-render-parts）＋ RenderMetaSchema 検証の単体テスト。

将来 UsecaseSpec の TestScenarios になる（移行レディ）。
"""
import pytest

from has_udd.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from has_udd.adapters.outbound.schema_repo import PackageSchemaRepository
from has_udd.domain.services.part_renderer import render_parts


def _lint(parts):
    meta = PackageSchemaRepository().load("RenderMetaSchema/v1")
    schema = {"$defs": meta["$defs"], "type": "array", "items": {"$ref": "#/$defs/RenderPart"}}
    return JsonSchemaValidator().validate(parts, schema)


# --- 描画 ---

def test_paragraph_and_list_md():
    md = render_parts(
        [{"as": "paragraph", "from": "text"}, {"as": "list", "from": "items"}],
        {"text": "説明", "items": ["a", "b"]}, "md", 3,
    )
    assert "説明" in md
    assert "- a\n- b" in md


def test_table_escapes_pipe_and_formats_bool():
    parts = [{"as": "table", "from": "rows", "columns": [
        {"field": "name"}, {"field": "type"}, {"field": "required", "header": "必須"}]}]
    data = {"rows": [
        {"name": "prompt", "type": "string | null", "required": True},
        {"name": "x", "type": "int", "required": False}]}
    md = render_parts(parts, data, "md", 3)
    lines = md.splitlines()
    # ヘッダ行の列数が崩れない（| が 4 本＝3列）
    assert lines[0].count("|") == 4
    assert "string \\| null" in md   # セルの | はエスケープ
    assert "| ✓ |" in md and "| - |" in md   # bool は ✓/-


def test_section_nesting_with_item_label():
    parts = [{"as": "section", "from": "items", "titleFrom": "title", "itemLabel": "Step", "each": [
        {"as": "paragraph", "from": "summary"}, {"as": "list", "from": "bullets"}]}]
    data = {"items": [{"title": "選ぶ", "summary": "要点", "bullets": ["x", "y"]}]}
    md = render_parts(parts, data, "md", 3)
    assert "### Step 1: 選ぶ" in md
    assert "要点" in md
    assert "- x\n- y" in md


def test_keyvalue_and_html():
    parts = [{"as": "keyvalue", "from": "refs", "labelFrom": "path", "valueFrom": "desc"}]
    data = {"refs": [{"path": "a.md", "desc": "説明A"}]}
    assert "- **a.md**: 説明A" in render_parts(parts, data, "md", 3)
    assert "<dl>" in render_parts(parts, data, "html", 3)


# --- 検証（RenderMetaSchema・誤設定を弾く） ---

def test_lint_accepts_valid():
    assert _lint([{"as": "paragraph", "from": "text"},
                  {"as": "table", "from": "rows", "columns": [{"field": "name"}]}]) == []


def test_lint_rejects_unknown_part():
    assert _lint([{"as": "foobar", "from": "x"}])  # enum 違反で非空


def test_lint_rejects_missing_required_attr():
    assert _lint([{"as": "table", "from": "rows"}])  # columns 漏れで非空


@pytest.mark.parametrize("schema_ref", ["SkillSchema/v1", "CodingSchema/v1", "SpecSchema/v1"])
def test_schema_xrender_conforms(schema_ref):
    """全 schema の全 block の x-render が RenderMetaSchema に適合する（誤設定・旧 {md,html} 形式の混入を防ぐ）。"""
    schema = PackageSchemaRepository().load(schema_ref)
    for name, bdef in schema["$defs"].items():
        if "x-render" in bdef:
            assert _lint(bdef["x-render"]) == [], f"{schema_ref}:{name} の x-render が不適合"
