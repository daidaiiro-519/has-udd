"""uc-render-document の GuaranteeScenarios（part_renderer の各部品整形保証）に対応するネイティブテスト。

x-render/lint(RenderMetaSchema検証)分のテストはagg-schema由来のため test_agg_schema.py に集約した。
"""
from waffle.domain.services.part_renderer import render_parts


# --- 描画 ---

def test_paragraph_listが正しく整形される():
    """
    Given paragraph/listを宣言するx-render
    When renderする
    Then paragraphは地の文、listは箇条書きとして整形される
    """
    md = render_parts(
        [{"as": "paragraph", "from": "text"}, {"as": "list", "from": "items"}],
        {"text": "説明", "items": ["a", "b"]}, 3,
    )
    assert "説明" in md
    assert "- a\n- b" in md


def test_tableはパイプ文字をエスケープしboolを整形する():
    """
    Given パイプ文字やbool値を含む行データ
    When tableとしてrenderする
    Then パイプ文字はエスケープされ、boolは✓/-に整形される
    """
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
    """
    Given itemLabelを持つsection宣言と入れ子のeach部品
    When renderする
    Then 各itemの見出しにitemLabelが付与され、入れ子の部品も正しく描画される
    """
    parts = [{"as": "section", "from": "items", "titleFrom": "title", "itemLabel": "Step", "each": [
        {"as": "paragraph", "from": "summary"}, {"as": "list", "from": "bullets"}]}]
    data = {"items": [{"title": "選ぶ", "summary": "要点", "bullets": ["x", "y"]}]}
    md = render_parts(parts, data, 3)
    assert "### Step 1: 選ぶ" in md
    assert "要点" in md
    assert "- x\n- y" in md


def test_keyvalueが正しく整形される():
    """
    Given keyvalueを宣言するx-render
    When renderする
    Then ラベルと値の組が箇条書きとして整形される
    """
    parts = [{"as": "keyvalue", "from": "refs", "labelFrom": "path", "valueFrom": "desc"}]
    data = {"refs": [{"path": "a.md", "desc": "説明A"}]}
    assert "- **a.md**: 説明A" in render_parts(parts, data, 3)


def test_sectionはbadgeで条件付き強調を付与する():
    """
    Given badge条件を満たすitemを含むsection宣言
    When renderする
    Then 条件を満たすitemの見出しにのみ強調語が付与される
    """
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
    """
    Given markFieldが真の行を含むtable宣言
    When renderする
    Then 該当セルが太字＋markSuffixで強調される
    """
    parts = [{"as": "table", "from": "attrs", "columns": [
        {"field": "name", "header": "属性", "markField": "isId", "markSuffix": "（識別子）"},
        {"field": "type", "header": "型"}]}]
    data = {"attrs": [{"name": "documentId", "type": "DocumentId", "isId": True},
                      {"name": "status", "type": "Status"}]}
    md = render_parts(parts, data, 3)
    assert "| **documentId**（識別子） | DocumentId |" in md
    assert "| status | Status |" in md


def test_statediagramが正しいMermaid構文になる():
    """
    Given 状態遷移の配列を宣言するx-render
    When renderする
    Then stateDiagram-v2として正しいMermaid構文が生成される
    """
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
    """
    Given zones/connectionsを宣言するx-render
    When renderする
    Then architecture-betaとして正しいMermaid構文が生成される
    """
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
    """
    Given stages/transitionsを宣言するx-render
    When renderする
    Then flowchart LRとして正しいMermaid構文が生成される
    """
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
    """
    Given kind:actor/participantを含む参加者宣言
    When renderする
    Then actor/participantそれぞれの宣言がMermaid構文で区別される
    """
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
    """
    Given loop/alt種別のstepを含むsteps配列
    When renderする
    Then loop/altブロックが正しく入れ子のMermaid構文になる
    """
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
    """
    Given activate/deactivateフラグを持つstep
    When renderする
    Then Mermaidのアクティベーション記法(+/-)が正しく付与される
    """
    parts = [{"as": "sequence", "from": "items"}]
    data = {"items": [
        {"from": "A", "to": "B", "message": "呼出", "kind": "command", "activate": True},
        {"from": "B", "to": "A", "message": "応答", "kind": "return", "deactivate": True},
    ]}
    md = render_parts(parts, data, 3)
    assert "A->>+B: 呼出" in md
    assert "B-->>-A: 応答" in md


def test_statediagramは疑似状態を表現する():
    """
    Given pseudoStatesFromで疑似状態を宣言するx-render
    When renderする
    Then choice/fork/joinの疑似状態宣言がMermaid構文の先頭に出力される
    """
    parts = [{"as": "statediagram", "from": "transitions", "pseudoStatesFrom": "pseudoStates"}]
    data = {
        "pseudoStates": [{"id": "判定", "kind": "choice"}],
        "transitions": [{"from": "A", "to": "判定", "command": "check"}],
    }
    md = render_parts(parts, data, 3)
    assert "state 判定 <<choice>>" in md
    assert "A --> 判定: check" in md


def test_kvtableは単一行として整形される():
    """
    Given kvtableを宣言するx-render
    When renderする
    Then block自身の値が1行のtableとして整形される
    """
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
    """
    Given join/sepを指定したcolumns宣言と配列値を持つセル
    When renderする
    Then 配列の各要素がjoinテンプレートで整形されsepで連結される
    """
    parts = [{"as": "table", "from": "items", "columns": [
        {"field": "name", "header": "エンティティ"},
        {"field": "attributes", "header": "属性", "join": "{name}: {type}"}]}]
    data = {"items": [{"name": "Order", "attributes": [
        {"name": "status", "type": "OrderStatus"},
        {"name": "total", "type": "Money"}]}]}
    md = render_parts(parts, data, 3)
    assert "status: OrderStatus / total: Money" in md
