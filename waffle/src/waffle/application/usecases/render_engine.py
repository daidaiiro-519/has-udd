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

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.ports.schema_repository import SchemaRepository
from waffle.application.services.document_loading import load_document, load_schema, require_schema_ref
from waffle.domain.services import path_template
from waffle.domain.services.part_renderer import render_parts
from waffle.domain.services.schema_discriminator import discriminator_key
from waffle.shared.result import Err, Ok, Result

def _err(code: str, message: str) -> Err:
    return Err(message, [code])

def _select_template(value, spec_kind: str | None) -> str:
    """x-render-target の path/featurePath は、フラットな文字列（旧来）か
    specKind ごとの辞書（ネスト構造）のどちらでも書ける。辞書なら該当 specKind を選ぶ。"""
    if isinstance(value, dict):
        return value.get(spec_kind, "") if spec_kind else ""
    return value or ""

class RenderEngine:
    def __init__(
        self,
        documents: DocumentRepository,
        schemas: SchemaRepository,
    ) -> None:
        self._documents = documents
        self._schemas = schemas

    def run(self, document_path: str, deploy: bool = True) -> Result[dict]:
        loaded = load_document(self._documents, document_path)
        if isinstance(loaded, Err):
            return loaded
        doc = loaded.value

        schema_ref_result = require_schema_ref(doc)
        if isinstance(schema_ref_result, Err):
            return schema_ref_result
        schema_ref = schema_ref_result.value

        schema_result = load_schema(self._schemas, schema_ref)
        if isinstance(schema_result, Err):
            return schema_result
        schema = schema_result.value

        # render は schema 適合検証をしない（検証は uc-validate-document の責務・疎結合）。
        # 不正な構造の document は best-effort で描画される（Orchestrator が事前 validate する前提）。
        target = schema.get("x-render-target", {})
        formats = target.get("formats") or ["md"]
        fmt = formats[0]  # MD 正本（HTML は将来 viewer が担うため engine は MD のみ描画）
        defs = schema.get("$defs", {})

        output = self._render_frontmatter(doc, schema) + self._render_body(doc, defs)

        path_vars = self._resolve_path_vars(doc, schema, document_path)
        spec_kind = doc.get(discriminator_key(schema))
        path_template_str = _select_template(target.get("path"), spec_kind)
        feature_template_str = _select_template(target.get("featurePath"), spec_kind)

        canonical = path_template.resolve(path_template_str, **path_vars) if path_template_str else ""
        deployed: list[str] = []
        if deploy and canonical:
            try:
                # canonical（.waffle 配下）に書く
                self._documents.write_text(canonical, output)
                # deploy: 同一フォーマットは verbatim copy（更新漏れ防止のため render に内蔵）
                for dep in target.get("deploy", []):
                    dp = path_template.resolve(dep, **path_vars)
                    self._documents.write_text(dp, output)
                    deployed.append(dp)
            except OSError as e:
                return _err("WRITE_ERROR", f"書き込みに失敗しました: {e}")

        # 第2フォーマット: feature（x-test-scenario block の Gherkin を .feature へ）
        feature = _extract_feature(doc, defs) if "feature" in formats else None
        feature_path = ""
        if feature and deploy and feature_template_str:
            feature_path = path_template.resolve(feature_template_str, **path_vars)
            if feature_path:
                self._documents.write_text(feature_path, feature)

        return Ok({
            "path": canonical, "deployed": deployed, "format": fmt, "content": output,
            "feature": feature, "featurePath": feature_path or None,
        })

    def _resolve_path_vars(self, doc: dict, schema: dict, document_path: str) -> dict:
        """x-render-target のパステンプレートに渡す変数を組み立てる。

        x-source-target が specKind 等でネストしている場合、contextRef のような
        「document には保存しない」変数は、実際の document_path をそのテンプレートに
        逆解析して復元する（create 時に渡した値は保存せず、パスそのものから読み戻す）。
        """
        path_vars = {"documentId": doc["documentId"]}
        x_source = schema.get("x-source-target")
        if isinstance(x_source, dict):
            key = discriminator_key(schema)
            template = x_source.get(doc.get(key)) if key else None
            if template:
                recovered = path_template.reverse_parse(template, document_path)
                if recovered:
                    path_vars.update(recovered)
        return path_vars

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
    """x-test-scenario: true の全block（TestScenarios/UnitTestScenarios/GuaranteeScenarios等）の
    Gherkinを集約して返す（1文書に複数のx-test-scenario blockがあっても全て含める）。

    .feature は仕様内 Gherkin を実行可能形に書き出すだけ（render は内容を作らない・SP-6）。
    """
    body_lines: list[str] = []
    for block in doc.get("content", {}).values():
        if not isinstance(block, dict):
            continue
        bdef = defs.get(f"{block.get('blockType')}Block", {})
        if not bdef.get("x-test-scenario"):
            continue
        scenarios = block.get("scenarios")
        if not scenarios:
            continue
        bg = (block.get("background") or "").strip()
        if bg:
            body_lines.append(f"  # 背景: {bg}")
        for s in scenarios:
            g = (s.get("gherkin") or "").strip()
            if not g:
                continue
            body_lines.append("")
            body_lines.extend("  " + ln for ln in g.splitlines())
    if not body_lines:
        return None
    lines = [f"Feature: {doc.get('documentId', 'spec')}", *body_lines]
    return "\n".join(lines) + "\n"

def _resolve_path(root: dict, path: str):
    """'doc.content.purpose.text' のようなドット区切りパスで dict を辿り値を返す。

    x-frontmatter は各 schema が『フィールド→パス』を宣言する（ロジックはデータに置かず
    描画は engine が担う＝Harness 原則）。新しい frontmatter パターンはこの宣言を増やすだけで対応する。
    """
    cur = root
    for part in path.split("."):
        cur = cur[part]
    return cur
