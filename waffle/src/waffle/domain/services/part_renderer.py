"""部品レンダラ — 宣言的 x-render（RenderPart の配列）を Markdown に描画する純ロジック。

RenderMetaSchema/v1 の部品語彙（paragraph/list/table/section/keyvalue/code/divider/
sequence/statediagram/kvtable）を engine 1実装で描画する。table はセルの '|' エスケープ・
bool の ✓/- 整形を一律で行うので、全 block・全 skill のテーブルが崩れず統一される。
`from` 先が空なら部品ごと省略するので、条件付きセクション（operations / note 等）は
別ロジック不要で消える。HTML は viewer 側（クライアントサイド）が担うため、ここは MD 正本に統一する。
"""
from __future__ import annotations

def render_parts(parts: list[dict], data: dict, level: int) -> str:
    """parts(宣言の配列) を data(block の値) から Markdown に描画する。level=小見出しの基準レベル。"""
    return _join((render_part(p, data, level)) for p in parts)


def render_part(part: dict, data: dict, level: int) -> str:
    """単一の RenderPart 宣言を Markdown 断片に描画する（対応する data が空なら空文字を返し部品ごと省略する）。"""
    kind = part["as"]
    # kvtable は from を取らず現在の data 自身を1行として描く
    src = data.get(part["from"]) if "from" in part else None
    if "from" in part and not src:
        return ""  # データ無し → 見出しごと省略（条件付き部品）

    out: list[str] = []
    if part.get("heading"):
        out.append(_heading(part["heading"], level))

    if kind == "paragraph":
        out.append(_para(part.get("text", src)))
    elif kind == "list":
        out.append(_list(src, part.get("ordered", False)))
    elif kind == "table":
        out.append(_table(src, part["columns"]))
    elif kind == "kvtable":
        out.append(_table([data], part["columns"]))
    elif kind == "keyvalue":
        out.append(_keyvalue(part, data, src))
    elif kind == "code":
        out.append(_code(src, part.get("lang")))
    elif kind == "sequence":
        out.append(_sequence(src))
    elif kind == "statediagram":
        out.append(_statediagram(src))
    elif kind == "divider":
        out.append("---")
    elif kind == "section":
        for i, item in enumerate(src or [], 1):
            title = item.get(part.get("titleFrom", "title"), "")
            if part.get("itemLabel"):
                title = f"{part['itemLabel']} {i}: {title}"
            badge = part.get("badge")
            if badge and item.get(badge.get("from")):
                title = f"{title}（{badge.get('text', '')}）"
            out.append(_heading(title, level))
            body = render_parts(part["each"], item, level + 1)
            if body:
                out.append(body)

    return _join(out)

# --- 整形ヘルパ ---

def _join(chunks):
    return "\n\n".join(s for s in chunks if s)

def _fmt(v):
    return ("✓" if v else "-") if isinstance(v, bool) else v

def _mdcell(v, code=False, join=None, sep=" / "):
    # セル値が配列なら畳む。dict 要素は join テンプレ（あれば）/ 文字列要素は str。sep で連結
    if isinstance(v, list):
        v = sep.join((join.format(**it) if (join and isinstance(it, dict)) else str(it)) for it in v)
    s = str(_fmt(v)).replace("|", "\\|").replace("\n", " ")
    return f"`{s}`" if code and s else s

def _heading(text, level):
    return "#" * level + " " + str(text)

def _para(text):
    return str(text)

def _list(items, ordered):
    return "\n".join((f"{i}. " if ordered else "- ") + str(x) for i, x in enumerate(items, 1))

def _table(rows, columns):
    headers = [c.get("header", c["field"]) for c in columns]
    out = ["| " + " | ".join(headers) + " |", "|" + "|".join("---" for _ in headers) + "|"]
    for r in rows:
        out.append("| " + " | ".join(_cell(r, c) for c in columns) + " |")
    return "\n".join(out)

def _cell(r, c):
    v = _mdcell(r.get(c["field"], ""), c.get("code"), c.get("join"), c.get("sep", " / "))
    if c.get("markField") and r.get(c["markField"]):
        v = f"**{v}**{c.get('markSuffix', '')}"
    return v

def _keyvalue(part, data, src):
    if "pairs" in part:
        pairs = [(p["label"], data.get(p["from"])) for p in part["pairs"]]
        pairs = [(k, v) for k, v in pairs if v not in (None, "", [])]
    elif isinstance(src, list):
        lf, vf = part.get("labelFrom"), part.get("valueFrom")
        pairs = [(it.get(lf, ""), it.get(vf, "")) for it in src]
    elif isinstance(src, dict):
        pairs = list(src.items())
    else:
        pairs = []
    lc, vc = part.get("labelCode"), part.get("valueCode")

    def lab(k):
        return f"`{k}`" if lc else f"**{k}**"

    def val(v):
        return f"`{v}`" if vc else str(v)
    return "\n".join(f"- {lab(k)}: {val(v)}" for k, v in pairs)

def _code(text, lang):
    items = text if isinstance(text, list) else [text]
    return "\n\n".join(f"```{lang or ''}\n{t}\n```" for t in items)

def _seq_token(name: str) -> str:
    """Mermaid の participant 識別子向けに空白を除く（ドメイン役者名は単語想定）。"""
    return str(name).replace(" ", "_")

def _sequence(steps):
    """構造化ステップ（from/to/message/kind）→ Mermaid sequenceDiagram。"""
    lines = ["sequenceDiagram"]
    for s in steps:
        if not isinstance(s, dict):
            continue
        frm = _seq_token(s.get("from", ""))
        to = _seq_token(s.get("to", "") or s.get("from", ""))
        msg = str(s.get("message", "")).replace("\n", " ")
        kind = s.get("kind", "command")
        if kind == "event":
            lines.append(f"    Note over {frm}: {msg}")
        elif kind == "return":
            lines.append(f"    {frm}-->>{to}: {msg}")
        else:  # command / self
            lines.append(f"    {frm}->>{to}: {msg}")
    diagram = "\n".join(lines)
    return f"```mermaid\n{diagram}\n```"

def _statediagram(transitions):
    """状態遷移配列（from/to/command）→ Mermaid stateDiagram-v2。状態名の空白は _ に。"""
    lines = ["stateDiagram-v2"]
    for t in transitions:
        if not isinstance(t, dict):
            continue
        frm = _seq_token(t.get("from", ""))
        to = _seq_token(t.get("to", ""))
        cmd = str(t.get("command", "")).replace("\n", " ")
        lines.append(f"    {frm} --> {to}: {cmd}")
    diagram = "\n".join(lines)
    return f"```mermaid\n{diagram}\n```"
