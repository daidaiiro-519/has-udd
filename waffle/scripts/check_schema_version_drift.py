"""schemaRefが指すschemaの実在性・バージョンの新しさを検証し、第4のドリフト
（schema進化による既存Documentの陳腐化）を検知する。

check_scenario_drift.py（spec↔テストコード間）・check_spec_referential_integrity.py
（spec内参照整合性）に続く第3のドリフト検知。

x-schema-status(PUBLISHED/DEPRECATED)による検知は撤去した。ディスク上に存在する
バージョンファイルとの数値比較（newer_version_available）だけで「このDocumentは
古いバージョンを使っている」を検知するのに十分であり、手動維持が必要な状態
フィールドを別途持つ必要が無いと判断したため（docs/brainstorm/
brainstorm-schema-versioning-migration.md 後日談参照）。

使い方:
    uv run python scripts/check_schema_version_drift.py <documents_root> <schemas_root>
"""
import json
import sys
from pathlib import Path

from waffle.domain.services.schema_versioning import version_number


def _load(path: Path) -> dict:
    return json.loads(path.read_text())


def check(documents_root: str, schemas_root: str) -> dict:
    documents_dir = Path(documents_root)
    schemas_dir = Path(schemas_root)

    all_versions_by_name: dict[str, list[str]] = {}
    for f in schemas_dir.glob("*/v*.json"):
        all_versions_by_name.setdefault(f.parent.name, []).append(f.stem)

    broken_references: list[dict] = []
    newer_version_available: list[dict] = []

    for doc_path in documents_dir.rglob("*.json"):
        doc = _load(doc_path)
        schema_ref = doc.get("schemaRef")
        if not schema_ref or "/" not in schema_ref:
            continue
        name, version = schema_ref.split("/", 1)
        schema_path = schemas_dir / name / f"{version}.json"
        if not schema_path.exists():
            broken_references.append({"document": str(doc_path), "schemaRef": schema_ref})
            continue

        versions = sorted(
            (v for v in all_versions_by_name.get(name, []) if version_number(v) is not None),
            key=version_number,
        )
        current_number = version_number(version)
        if versions and current_number is not None and current_number != version_number(versions[-1]):
            newer_version_available.append(
                {"document": str(doc_path), "schemaRef": schema_ref, "latest": f"{name}/{versions[-1]}"}
            )

    return {
        "broken_references": broken_references,
        "newer_version_available": newer_version_available,
    }


if __name__ == "__main__":
    documents_root, schemas_root = sys.argv[1], sys.argv[2]
    result = check(documents_root, schemas_root)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    if result["broken_references"] or result["newer_version_available"]:
        sys.exit(1)
