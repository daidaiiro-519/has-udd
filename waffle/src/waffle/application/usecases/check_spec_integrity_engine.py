"""check spec integrity engine — bounded-context が宣言する members(subdomain/usecase)と、
ディスク上に実在する subdomain/usecase ドキュメントの参照整合性を検証する application use case。

宣言先ファイルがディスクに存在しないことはエラーではなく、それ自体が検出結果（ドリフト）。
実行/意味理解はしない（宣言された名前集合の機械的な突き合わせのみ）。
"""
from __future__ import annotations

from waffle.application.ports.document_repository import DocumentRepository
from waffle.application.services.document_loading import load_document
from waffle.shared.result import Err, Ok, Result


def _split(members_str: str) -> list[str]:
    return [s.strip() for s in members_str.split("/") if s.strip()] if members_str else []


class CheckSpecIntegrityEngine:
    def __init__(self, documents: DocumentRepository) -> None:
        self._documents = documents

    def run(self, bc_path: str) -> Result[dict]:
        loaded = load_document(self._documents, bc_path)
        if isinstance(loaded, Err):
            return loaded
        bc_doc = loaded.value

        bc_dir = bc_path.rsplit("/", 1)[0] if "/" in bc_path else "."
        subdomain_dir = f"{bc_dir}/subdomain"

        members = bc_doc["content"]["members"]["items"]
        declared_subdomains = set(_split(next((m["members"] for m in members if m["kind"] == "subdomain"), "")))
        declared_usecases = set(_split(next((m["members"] for m in members if m["kind"] == "usecase"), "")))

        try:
            candidate_dirs = self._documents.list_dirs(subdomain_dir)
        except FileNotFoundError:
            candidate_dirs = []

        actual_subdomain_dirs: set[str] = set()
        for name in candidate_dirs:
            sd_files = self._documents.list_json(f"{subdomain_dir}/{name}")
            if f"{subdomain_dir}/{name}/{name}.json" in sd_files:
                actual_subdomain_dirs.add(name)

        subdomain_usecases: dict[str, set[str]] = {}
        actual_usecase_files: set[str] = set()
        for name in actual_subdomain_dirs:
            sd_doc = self._documents.load(f"{subdomain_dir}/{name}/{name}.json")
            items = sd_doc.get("content", {}).get("members", {}).get("items", [])
            subdomain_usecases[name] = set(items)
            try:
                uc_files = self._documents.list_json(f"{subdomain_dir}/{name}/usecase")
            except FileNotFoundError:
                uc_files = []
            for p in uc_files:
                stem = p.rsplit("/", 1)[-1].removesuffix(".json")
                actual_usecase_files.add(stem)

        all_subdomain_usecases: set[str] = set().union(*subdomain_usecases.values()) if subdomain_usecases else set()

        return Ok({
            "declared_subdomains_missing_on_disk": sorted(declared_subdomains - actual_subdomain_dirs),
            "subdomains_on_disk_not_declared_in_bc": sorted(actual_subdomain_dirs - declared_subdomains),
            "usecases_orphaned_no_subdomain": sorted(declared_usecases - all_subdomain_usecases),
            "usecases_in_subdomain_not_declared_in_bc": sorted(all_subdomain_usecases - declared_usecases),
            "usecase_files_missing_on_disk": sorted(all_subdomain_usecases - actual_usecase_files),
            "usecase_files_orphaned_on_disk": sorted(actual_usecase_files - all_subdomain_usecases),
        })
