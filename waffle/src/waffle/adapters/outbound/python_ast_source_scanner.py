"""python ast adapter（SourceScanner実装）。

kind="google"（Python生態系のgoogle-style docstring規約）専用のadapter。
Pythonの`ast`モジュールで構文解析し、docstringテキストをgoogle-style規約に
従って構造化する。この規約解析ロジック自体もPython生態系に固有の技術選択
であり、他kindとは共有しない（kindが増えたら別adapterを追加する）。
"""
from __future__ import annotations

import ast
import re

from waffle.application.ports.source_scanner import SourceScanner, UnsupportedKind

_SECTION_HEADERS = {"Args:", "Arguments:", "Returns:", "Yields:", "Raises:", "Attributes:"}
_ENTRY_RE = re.compile(r"^(?P<name>\w+)(?:\s*\(([^)]*)\))?:\s*(?P<rest>.*)$")


def _parse_entries(lines: list[str]) -> list[dict]:
    """"name (type): desc" 形式のエントリ群を解析する。継続行は前のエントリの description に連結。"""
    entries: list[dict] = []
    for line in lines:
        stripped = line.strip()
        if not stripped:
            continue
        m = _ENTRY_RE.match(stripped)
        if m and (not line.startswith(" " * 8) or not entries):
            entries.append({"name": m.group("name"), "description": m.group("rest").strip()})
        elif entries:
            entries[-1]["description"] = (entries[-1]["description"] + " " + stripped).strip()
    return entries


def _parse_raises(lines: list[str]) -> list[dict]:
    entries = _parse_entries(lines)
    return [{"exceptionType": e["name"], "condition": e["description"]} for e in entries]


def _parse_google_docstring(docstring: str) -> dict:
    empty = {"summary": "", "body": "", "args": [], "returns": "", "raises": [], "attributes": []}
    if not docstring:
        return empty

    lines = docstring.strip("\n").splitlines()
    summary = ""
    body_lines: list[str] = []
    sections: dict[str, list[str]] = {}
    current_section: str | None = None
    i = 0

    while i < len(lines) and not lines[i].strip():
        i += 1
    if i < len(lines):
        summary = lines[i].strip()
        i += 1

    while i < len(lines):
        stripped = lines[i].strip()
        if stripped in _SECTION_HEADERS:
            current_section = stripped.rstrip(":")
            sections.setdefault(current_section, [])
        elif current_section is not None:
            sections[current_section].append(lines[i])
        else:
            body_lines.append(lines[i])
        i += 1

    body = "\n".join(body_lines).strip()
    args = _parse_entries(sections.get("Args", []) or sections.get("Arguments", []))
    returns_lines = sections.get("Returns", []) or sections.get("Yields", [])
    returns = " ".join(ln.strip() for ln in returns_lines if ln.strip())
    raises = _parse_raises(sections.get("Raises", []))
    attributes = _parse_entries(sections.get("Attributes", []))

    return {
        "summary": summary,
        "body": body,
        "args": args,
        "returns": returns,
        "raises": raises,
        "attributes": attributes,
    }


def _is_public(name: str) -> bool:
    return not name.startswith("_")


def _signature_params(args: ast.arguments) -> list[str]:
    names = [a.arg for a in args.posonlyargs]
    names += [a.arg for a in args.args]
    if args.vararg:
        names.append(args.vararg.arg)
    names += [a.arg for a in args.kwonlyargs]
    if args.kwarg:
        names.append(args.kwarg.arg)
    return [n for n in names if n not in ("self", "cls")]


def _element(path: str, kind: str, element_kind: str, name: str, doc: str | None, signature_params: list[str]) -> dict:
    parsed = _parse_google_docstring(doc or "")
    return {
        "path": path,
        "kind": kind,
        "elementKind": element_kind,
        "name": name,
        "hasDocstring": doc is not None,
        "signatureParams": signature_params if element_kind == "function" else [],
        "summary": parsed["summary"],
        "body": parsed["body"],
        "args": parsed["args"],
        "returns": parsed["returns"],
        "raises": parsed["raises"],
        "attributes": parsed["attributes"] if element_kind == "class" else [],
    }


class PythonAstSourceScanner(SourceScanner):
    def scan(self, source: str, path: str, kind: str) -> list[dict]:
        if kind != "google":
            raise UnsupportedKind(kind)

        tree = ast.parse(source)
        elements: list[dict] = [
            _element(path, kind, "module", path.rsplit("/", 1)[-1], ast.get_docstring(tree), [])
        ]

        for node in tree.body:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and _is_public(node.name):
                elements.append(_element(
                    path, kind, "function", node.name, ast.get_docstring(node), _signature_params(node.args),
                ))
            elif isinstance(node, ast.ClassDef) and _is_public(node.name):
                elements.append(_element(path, kind, "class", node.name, ast.get_docstring(node), []))
                for sub in node.body:
                    if isinstance(sub, (ast.FunctionDef, ast.AsyncFunctionDef)) and _is_public(sub.name):
                        elements.append(_element(
                            path, kind, "function", sub.name, ast.get_docstring(sub), _signature_params(sub.args),
                        ))

        return elements
