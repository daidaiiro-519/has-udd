"""render engine — document.json を成果物（SKILL.md / HTML 等）にレンダリングし、
x-render-target.path の場所へ deploy する application use case。

汎用エンジン（schema 固有ロジックを持たない）:
- frontmatter は schema の x-frontmatter から生成
- body は content の各ブロックを x-render-order でソートし、
  「見出し(x-render-level + block.title) + x-render(宣言的部品) 本体」を生成
  （部品の描画は domain/services/part_renderer に委譲）
- 出力先は x-render-target.path
"""
from __future__ import annotations

import json
from pathlib import Path

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.schema_repository import SchemaRepository
from waffle.domain.services.part_renderer import render_parts
from waffle.shared.result import Err, Ok, Result

def _err(code: str, message: str) -> Err:
    return Err(message, [code])

class RenderEngine:
    def __init__(
        self,
        documents: DocumentRepository,
        schemas: SchemaRepository,
    ) -> None:
        self._documents = documents
        self._schemas = schemas

    def run(self, document_path: str, deploy: bool = True) -> Result[dict]:
        # G6: パストラバーサル拒否
        if ".." in Path(document_path).parts:
            return _err("INVALID_PATH", f"パストラバーサルは許可されません: {document_path}")
        try:
            doc = self._documents.load(document_path)
        except FileNotFoundError:
            return _err("INVALID_PATH", f"ファイルが見つかりません: {document_path}")
        except json.JSONDecodeError:
            return _err("INVALID_JSON", f"JSON として解釈できません: {document_path}")

        schema_ref = doc.get("schemaRef")
        if not schema_ref:
            return _err("MISSING_SCHEMA_REF", "document に schemaRef がありません")
        try:
            schema = self._schemas.load(schema_ref)
        except (FileNotFoundError, ModuleNotFoundError):
            return _err("INVALID_SCHEMA_REF", f"schema を解決できません: {schema_ref}")

        # render は schema 適合検証をしない（検証は uc-validate-document の責務・疎結合）。
        # 不正な構造の document は best-effort で描画される（Orchestrator が事前 validate する前提）。
        target = schema.get("x-render-target", {})
        formats = target.get("formats") or ["md"]
        fmt = formats[0]  # MD 正本（HTML は将来 viewer が担うため engine は MD のみ描画）
        defs = schema.get("$defs", {})

        output = self._render_frontmatter(doc, schema) + self._render_body(doc, defs)

        canonical = (target.get("path") or "").format(documentId=doc["documentId"])
        deployed: list[str] = []
        if deploy and canonical:
            try:
                # canonical（.waffle 配下）に書く
                self._documents.write_text(canonical, output)
                # deploy: 同一フォーマットは verbatim copy（更新漏れ防止のため render に内蔵）
                for dep in target.get("deploy", []):
                    dp = dep.format(documentId=doc["documentId"])
                    self._documents.write_text(dp, output)
                    deployed.append(dp)
            except OSError as e:
                return _err("WRITE_ERROR", f"書き込みに失敗しました: {e}")

        # 第2フォーマット: feature（x-test-scenario block の Gherkin を .feature へ）
        feature = _extract_feature(doc, defs) if "feature" in formats else None
        feature_path = ""
        if feature and deploy:
            feature_path = (target.get("featurePath") or "").format(documentId=doc["documentId"])
            if feature_path:
                self._documents.write_text(feature_path, feature)

        return Ok({
            "path": canonical, "deployed": deployed, "format": fmt, "content": output,
            "feature": feature, "featurePath": feature_path or None,
        })

    def _render_frontmatter(self, doc: dict, schema: dict) -> str:
        fm = schema.get("x-frontmatter")
        if not fm:
            return ""
        lines = ["---"]
        for key, path in fm.items():
            value = _resolve_path({"doc": doc}, path)
            # JSON 文字列は YAML のスカラとしても安全（コロン・括弧・日本語を含んでも壊れない）
            lines.append(f"{key}: {json.dumps(value, ensure_ascii=False)}")
        lines.append("---")
        return "\n".join(lines) + "\n\n"

    def _render_body(self, doc: dict, defs: dict) -> str:
        content = doc.get("content", {})
        ordered = []
        for _key, block in content.items():
            bdef = defs.get(block["blockType"] + "Block", {})
            ordered.append((bdef.get("x-render-order", 999), bdef, block))
        ordered.sort(key=lambda t: t[0])

        parts = []
        for _order, bdef, block in ordered:
            level = bdef.get("x-render-level", 2)
            title = block.get("title", "")
            heading = "#" * level + " " + title
            # x-render は宣言的部品配列。小見出しは block 見出し+1 から。
            xr = bdef.get("x-render") or []
            body = render_parts(xr, block, level + 1).strip()
            parts.append(heading + ("\n\n" + body if body else ""))
        # トップレベルのセクション間に区切り線を入れて境界を明確にする
        return "\n\n---\n\n".join(parts) + "\n"

def _extract_feature(doc: dict, defs: dict):
    """x-test-scenario: true の block（TestScenarios/UnitTestScenarios）の Gherkin を返す。

    .feature は仕様内 Gherkin を実行可能形に書き出すだけ（render は内容を作らない・SP-6）。
    """
    for block in doc.get("content", {}).values():
        if not isinstance(block, dict):
            continue
        bdef = defs.get(f"{block.get('blockType')}Block", {})
        if not bdef.get("x-test-scenario"):
            continue
        # TestScenariosBlock の scenarios[{gherkin}] を Feature にまとめる
        scenarios = block.get("scenarios")
        if scenarios:
            lines = [f"Feature: {doc.get('documentId', 'spec')}"]
            bg = (block.get("background") or "").strip()
            if bg:
                lines.append(f"  # 背景: {bg}")
            for s in scenarios:
                g = (s.get("gherkin") or "").strip()
                if not g:
                    continue
                lines.append("")
                lines.extend("  " + ln for ln in g.splitlines())
            return "\n".join(lines) + "\n"
    return None

def _resolve_path(root: dict, path: str):
    """'doc.content.purpose.text' のようなドット区切りパスで dict を辿り値を返す。

    x-frontmatter は各 schema が『フィールド→パス』を宣言する（ロジックはデータに置かず
    描画は engine が担う＝Harness 原則）。新しい frontmatter パターンはこの宣言を増やすだけで対応する。
    """
    cur = root
    for part in path.split("."):
        cur = cur[part]
    return cur
