"""部品レンダラ — 宣言的 x-render（RenderPart の配列）を md/html に描画する純ロジック。

RenderMetaSchema/v1 の部品語彙（paragraph/list/table/section/keyvalue/code/divider）を
engine 1実装で描画する。table はセルの '|' エスケープ・bool の ✓/- 整形を一律で行うので、
全 block・全 skill のテーブルが崩れず統一される。`from` 先が空なら部品ごと省略するので、
条件付きセクション（operations / note 等）は別ロジック不要で消える。

@spec:uc-render-parts
"""
from __future__ import annotations


def render_parts(parts: list[dict], data: dict, fmt: str, level: int) -> str:
    """parts(宣言の配列) を data(block の値) から fmt(md/html) に描画する。level=小見出しの基準レベル。"""
    # has-udd:impl-start
    return _join(((render_part(p, data, fmt, level)) for p in parts), fmt)
    # has-udd:impl-end


def render_part(part: dict, data: dict, fmt: str, level: int) -> str:
    # has-udd:impl-start
    kind = part["as"]
    src = data.get(part["from"]) if "from" in part else None
    if "from" in part and not src:
        return ""  # データ無し → 見出しごと省略（条件付き部品）

    out: list[str] = []
    if part.get("heading"):
        out.append(_heading(part["heading"], level, fmt))

    if kind == "paragraph":
        out.append(_para(part.get("text", src), fmt))
    elif kind == "list":
        out.append(_list(src, part.get("ordered", False), fmt))
    elif kind == "table":
        out.append(_table(src, part["columns"], fmt))
    elif kind == "keyvalue":
        out.append(_keyvalue(part, data, src, fmt))
    elif kind == "code":
        out.append(_code(src, part.get("lang"), fmt))
    elif kind == "sequence":
        out.append(_sequence(src, fmt))
    elif kind == "divider":
        out.append("---" if fmt == "md" else "<hr>")
    elif kind == "section":
        for i, item in enumerate(src or [], 1):
            title = item.get(part.get("titleFrom", "title"), "")
            if part.get("itemLabel"):
                title = f"{part['itemLabel']} {i}: {title}"
            out.append(_heading(title, level, fmt))
            body = render_parts(part["each"], item, fmt, level + 1)
            if body:
                out.append(body)

    return _join(out, fmt)
    # has-udd:impl-end


# --- 整形ヘルパ ---

def _join(chunks, fmt):
    sep = "\n\n" if fmt == "md" else ""
    return sep.join(s for s in chunks if s)


def _fmt(v):
    return ("✓" if v else "-") if isinstance(v, bool) else v


def _esc(v):
    return str(_fmt(v)).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def _mdcell(v, code=False):
    s = str(_fmt(v)).replace("|", "\\|").replace("\n", " ")
    return f"`{s}`" if code and s else s


def _heading(text, level, fmt):
    return ("#" * level + " " + str(text)) if fmt == "md" else f"<h{level}>{_esc(text)}</h{level}>"


def _para(text, fmt):
    return str(text) if fmt == "md" else f"<p>{_esc(text)}</p>"


def _list(items, ordered, fmt):
    if fmt == "md":
        return "\n".join((f"{i}. " if ordered else "- ") + str(x) for i, x in enumerate(items, 1))
    tag = "ol" if ordered else "ul"
    return f"<{tag}>" + "".join(f"<li>{_esc(x)}</li>" for x in items) + f"</{tag}>"


def _table(rows, columns, fmt):
    headers = [c.get("header", c["field"]) for c in columns]
    if fmt == "md":
        out = ["| " + " | ".join(headers) + " |", "|" + "|".join("---" for _ in headers) + "|"]
        for r in rows:
            out.append("| " + " | ".join(_mdcell(r.get(c["field"], ""), c.get("code")) for c in columns) + " |")
        return "\n".join(out)
    th = "".join(f"<th>{_esc(h)}</th>" for h in headers)
    body = "".join(
        "<tr>" + "".join(_htmlcell(r.get(c["field"], ""), c.get("code")) for c in columns) + "</tr>"
        for r in rows
    )
    return f"<table><thead><tr>{th}</tr></thead><tbody>{body}</tbody></table>"


def _htmlcell(v, code=False):
    inner = f"<code>{_esc(v)}</code>" if code and v not in ("", None) else _esc(v)
    return f"<td>{inner}</td>"


def _keyvalue(part, data, src, fmt):
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
    if fmt == "md":
        def lab(k):
            return f"`{k}`" if lc else f"**{k}**"

        def val(v):
            return f"`{v}`" if vc else str(v)
        return "\n".join(f"- {lab(k)}: {val(v)}" for k, v in pairs)
    return "<dl>" + "".join(
        f"<dt>{f'<code>{_esc(k)}</code>' if lc else _esc(k)}</dt>"
        f"<dd>{f'<code>{_esc(v)}</code>' if vc else _esc(v)}</dd>"
        for k, v in pairs
    ) + "</dl>"


def _code(text, lang, fmt):
    items = text if isinstance(text, list) else [text]
    if fmt == "md":
        return "\n\n".join(f"```{lang or ''}\n{t}\n```" for t in items)
    return "".join(f"<pre><code>{_esc(t)}</code></pre>" for t in items)


def _seq_token(name: str) -> str:
    """Mermaid の participant 識別子向けに空白を除く（ドメイン役者名は単語想定）。"""
    return str(name).replace(" ", "_")


def _sequence(steps, fmt):
    """構造化ステップ（from/to/message/kind）→ Mermaid sequenceDiagram。format 変換は adapter の責務。"""
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
    if fmt == "md":
        return f"```mermaid\n{diagram}\n```"
    return f'<pre class="mermaid">\n{diagram}\n</pre>'
