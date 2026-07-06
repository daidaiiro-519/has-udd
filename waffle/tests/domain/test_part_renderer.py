"""部品レンダラ（uc-render-parts）＋ RenderMetaSchema 検証の単体テスト。

将来 UsecaseSpec の TestScenarios になる（移行レディ）。
"""
import pytest

from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.domain.services.part_renderer import render_parts


def _lint(parts):
    meta = PackageSchemaRepository().load("RenderMetaSchema/v1")
    schema = {"$defs": meta["$defs"], "type": "array", "items": {"$ref": "#/$defs/RenderPart"}}
    return JsonSchemaValidator().validate(parts, schema)


# --- 描画 ---

def test_paragraph_listが正しく整形される():
    md = render_parts(
        [{"as": "paragraph", "from": "text"}, {"as": "list", "from": "items"}],
        {"text": "説明", "items": ["a", "b"]}, 3,
    )
    assert "説明" in md
    assert "- a\n- b" in md


def test_tableはパイプ文字をエスケープしboolを整形する():
    parts = [{"as": "table", "from": "rows", "columns": [
        {"field": "name"}, {"field": "type"}, {"field": "required", "header": "必須"}]}]
    data = {"rows": [
        {"name": "prompt", "type": "string | null", "required": True},
        {"name": "x", "type": "int", "required": False}]}
    md = render_parts(parts, data, 3)
    lines = md.splitlines()
    # ヘッダ行の列数が崩れない（| が 4 本＝3列）
    assert lines[0].count("|") == 4
    assert "string \\| null" in md   # セルの | はエスケープ
    assert "| ✓ |" in md and "| - |" in md   # bool は ✓/-


def test_sectionは入れ子とitemLabelを整形する():
    parts = [{"as": "section", "from": "items", "titleFrom": "title", "itemLabel": "Step", "each": [
        {"as": "paragraph", "from": "summary"}, {"as": "list", "from": "bullets"}]}]
    data = {"items": [{"title": "選ぶ", "summary": "要点", "bullets": ["x", "y"]}]}
    md = render_parts(parts, data, 3)
    assert "### Step 1: 選ぶ" in md
    assert "要点" in md
    assert "- x\n- y" in md


def test_keyvalueが正しく整形される():
    parts = [{"as": "keyvalue", "from": "refs", "labelFrom": "path", "valueFrom": "desc"}]
    data = {"refs": [{"path": "a.md", "desc": "説明A"}]}
    assert "- **a.md**: 説明A" in render_parts(parts, data, 3)


def test_sectionはbadgeで条件付き強調を付与する():
    parts = [{"as": "section", "from": "items", "titleFrom": "name",
              "badge": {"from": "isRoot", "text": "集約ルート"}, "each": [
                  {"as": "paragraph", "from": "role"}]}]
    data = {"items": [{"name": "Document", "role": "一貫性単位", "isRoot": True},
                      {"name": "Line", "role": "明細", "isRoot": False}]}
    md = render_parts(parts, data, 3)
    assert "### Document（集約ルート）" in md   # ルートに badge
    assert "### Line\n" in md                   # 子は badge なし
    assert "一貫性単位" in md


def test_tableはmarkFieldで識別子を太字強調する():
    parts = [{"as": "table", "from": "attrs", "columns": [
        {"field": "name", "header": "属性", "markField": "isId", "markSuffix": "（識別子）"},
        {"field": "type", "header": "型"}]}]
    data = {"attrs": [{"name": "documentId", "type": "DocumentId", "isId": True},
                      {"name": "status", "type": "Status"}]}
    md = render_parts(parts, data, 3)
    assert "| **documentId**（識別子） | DocumentId |" in md
    assert "| status | Status |" in md


def test_statediagramが正しいMermaid構文になる():
    parts = [{"as": "statediagram", "from": "transitions"}]
    data = {"transitions": [
        {"from": "A", "to": "B", "command": "cmd"},
        {"from": "open state", "to": "C", "command": "go"},
    ]}
    md = render_parts(parts, data, 3)
    assert "```mermaid" in md
    assert "stateDiagram-v2" in md
    assert "A --> B: cmd" in md
    # 状態名の空白は _ に
    assert "open_state --> C: go" in md


def test_architectureが正しいMermaid構文になる():
    parts = [{"as": "architecture", "from": "zones", "connectionsFrom": "connections"}]
    data = {
        "zones": [
            {"id": "public", "label": "Public", "contains": [{"id": "lb", "label": "ロードバランサ"}]},
            {"id": "private", "label": "Private", "contains": [{"id": "app", "label": "アプリ"}]},
        ],
        "connections": [{"from": "lb", "to": "app"}],
    }
    md = render_parts(parts, data, 3)
    assert "```mermaid" in md
    assert "architecture-beta" in md
    assert 'group public(cloud)[Public]' in md  # ASCIIラベルはクォート不要
    assert 'service lb(server)["ロードバランサ"] in public' in md  # 非ASCIIはクォート必須
    assert "lb:R --> L:app" in md


def test_flowchartが正しいMermaid構文になる():
    parts = [{"as": "flowchart", "from": "stages", "transitionsFrom": "transitions"}]
    data = {
        "stages": [{"id": "staging", "label": "Staging"}, {"id": "production", "label": "Production"}],
        "transitions": [{"from": "staging", "to": "production", "label": "承認"}],
    }
    md = render_parts(parts, data, 3)
    assert "```mermaid" in md
    assert "flowchart LR" in md
    assert 'staging[Staging]' in md
    assert 'staging -->|"承認"| production' in md  # 非ASCIIラベルはクォート必須


def test_sequenceはactor_participantを区別する():
    parts = [{"as": "sequence", "from": "items", "participantsFrom": "participants"}]
    data = {
        "participants": [
            {"id": "顧客", "kind": "actor"},
            {"id": "uc-place-order", "kind": "participant", "label": "uc-place-order"},
        ],
        "items": [{"from": "顧客", "to": "uc-place-order", "message": "注文する", "kind": "command"}],
    }
    md = render_parts(parts, data, 3)
    assert "actor 顧客" in md
    assert "participant uc_place_order as uc-place-order" in md
    assert "顧客->>uc_place_order: 注文する" in md


def test_sequenceはloop_altを入れ子で表現する():
    parts = [{"as": "sequence", "from": "items"}]
    data = {"items": [
        {"kind": "loop", "message": "3件分", "steps": [
            {"from": "A", "to": "B", "message": "処理", "kind": "command"},
        ]},
        {"kind": "alt", "branches": [
            {"label": "成功", "steps": [{"from": "B", "to": "A", "message": "OK", "kind": "return"}]},
            {"label": "失敗", "steps": [{"from": "B", "to": "A", "message": "NG", "kind": "return"}]},
        ]},
    ]}
    md = render_parts(parts, data, 3)
    assert "loop 3件分" in md
    assert "alt 成功" in md
    assert "else 失敗" in md
    assert md.count("end") == 2


def test_sequenceはactivate_deactivateを表現する():
    parts = [{"as": "sequence", "from": "items"}]
    data = {"items": [
        {"from": "A", "to": "B", "message": "呼出", "kind": "command", "activate": True},
        {"from": "B", "to": "A", "message": "応答", "kind": "return", "deactivate": True},
    ]}
    md = render_parts(parts, data, 3)
    assert "A->>+B: 呼出" in md
    assert "B-->>-A: 応答" in md


def test_statediagramは疑似状態を表現する():
    parts = [{"as": "statediagram", "from": "transitions", "pseudoStatesFrom": "pseudoStates"}]
    data = {
        "pseudoStates": [{"id": "判定", "kind": "choice"}],
        "transitions": [{"from": "A", "to": "判定", "command": "check"}],
    }
    md = render_parts(parts, data, 3)
    assert "state 判定 <<choice>>" in md
    assert "A --> 判定: check" in md


def test_kvtableは単一行として整形される():
    parts = [{"as": "kvtable", "columns": [
        {"field": "category", "header": "分類"},
        {"field": "viewpoint", "header": "観点"}]}]
    data = {"category": "正常系", "viewpoint": "状態遷移"}
    md = render_parts(parts, data, 3)
    lines = [ln for ln in md.splitlines() if ln.strip()]
    # ヘッダ + 区切り + 1行
    assert lines[0] == "| 分類 | 観点 |"
    assert "正常系" in lines[2] and "状態遷移" in lines[2]


def test_tableはjoin指定で配列セルを結合整形する():
    parts = [{"as": "table", "from": "items", "columns": [
        {"field": "name", "header": "エンティティ"},
        {"field": "attributes", "header": "属性", "join": "{name}: {type}"}]}]
    data = {"items": [{"name": "Order", "attributes": [
        {"name": "status", "type": "OrderStatus"},
        {"name": "total", "type": "Money"}]}]}
    md = render_parts(parts, data, 3)
    assert "status: OrderStatus / total: Money" in md


# --- 検証（RenderMetaSchema・誤設定を弾く） ---

def test_lint_accepts_valid():
    assert _lint([{"as": "paragraph", "from": "text"},
                  {"as": "table", "from": "rows", "columns": [{"field": "name"}]}]) == []


def test_lint_rejects_unknown_part():
    assert _lint([{"as": "foobar", "from": "x"}])  # enum 違反で非空


def test_lint_rejects_missing_required_attr():
    assert _lint([{"as": "table", "from": "rows"}])  # columns 漏れで非空


@pytest.mark.parametrize("schema_ref", ["SkillSchema/v1", "CodingSchema/v2", "DomainSpecSchema/v2", "PresentationSpecSchema/v1", "PlatformSpec/v1"])
def test_x_render_は閉じた語彙にのみ従う(schema_ref):
    """agg-schema(Schema集約)の不変条件「各ブロックのx-renderは常にRenderMetaSchemaの閉じた語彙にのみ従う」の実証。
    全 schema の全 block の x-render が RenderMetaSchema に適合する（誤設定・旧 {md,html} 形式の混入を防ぐ）。"""
    schema = PackageSchemaRepository().load(schema_ref)
    for name, bdef in schema["$defs"].items():
        if "x-render" in bdef:
            assert _lint(bdef["x-render"]) == [], f"{schema_ref}:{name} の x-render が不適合"
