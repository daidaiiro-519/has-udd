"""bounded-context が宣言する members(subdomain/usecase)と、実際の subdomain/usecase ドキュメントの
間の参照整合性を検証する。

check_scenario_drift.py が spec↔テストコード間のドリフトを見るのに対し、これは spec 同士
（bounded-context/subdomain/usecase）の参照整合性を見る、別種のドリフト検知。
実行/意味理解はしない（宣言された名前集合の突き合わせのみ）。

使い方:
    uv run python scripts/check_spec_referential_integrity.py <bc.json>
"""
import json
import sys
from pathlib import Path


def _load(path: Path) -> dict:
    return json.loads(path.read_text())


def _split(members_str: str) -> list[str]:
    return [s.strip() for s in members_str.split("/") if s.strip()] if members_str else []


def check(bc_path: str) -> dict:
    bc_file = Path(bc_path)
    bc_dir = bc_file.parent
    bc_doc = _load(bc_file)

    members = bc_doc["content"]["members"]["items"]
    declared_subdomains = set(_split(next((m["members"] for m in members if m["kind"] == "subdomain"), "")))
    declared_usecases = set(_split(next((m["members"] for m in members if m["kind"] == "usecase"), "")))

    # 実際に存在する subdomain ドキュメント(ディスク上の実態。自己格納パターン=フォルダ名と同名のjson)
    actual_subdomain_dirs = {
        p.parent.name for p in bc_dir.glob("subdomain/*/*.json") if p.stem == p.parent.name
    }

    subdomain_usecases: dict[str, set[str]] = {}
    for sd_name in actual_subdomain_dirs:
        sd_doc = _load(bc_dir / "subdomain" / sd_name / f"{sd_name}.json")
        items = sd_doc["content"].get("members", {}).get("items", [])
        subdomain_usecases[sd_name] = set(items)

    all_subdomain_usecases: set[str] = set().union(*subdomain_usecases.values()) if subdomain_usecases else set()

    # 実際に存在する usecase ドキュメント(ディスク上の実態)
    actual_usecase_files = {
        p.stem
        for sd_name in actual_subdomain_dirs
        for p in (bc_dir / "subdomain" / sd_name / "usecase").glob("*.json")
    }

    return {
        "declared_subdomains_missing_on_disk": sorted(declared_subdomains - actual_subdomain_dirs),
        "subdomains_on_disk_not_declared_in_bc": sorted(actual_subdomain_dirs - declared_subdomains),
        "usecases_orphaned_no_subdomain": sorted(declared_usecases - all_subdomain_usecases),
        "usecases_in_subdomain_not_declared_in_bc": sorted(all_subdomain_usecases - declared_usecases),
        "usecase_files_missing_on_disk": sorted(all_subdomain_usecases - actual_usecase_files),
        "usecase_files_orphaned_on_disk": sorted(actual_usecase_files - all_subdomain_usecases),
    }


if __name__ == "__main__":
    result = check(sys.argv[1])
    print(json.dumps(result, ensure_ascii=False, indent=2))
    if any(result.values()):
        sys.exit(1)
