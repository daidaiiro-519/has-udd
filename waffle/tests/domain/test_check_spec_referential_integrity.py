"""check_spec_referential_integrity.py（bounded-context/subdomain/usecase間の参照整合性検証）の単体テスト。

bc-waffle-engines.membersにuc-scan-source-code/uc-lint-docstringが宙に浮いていた
実際の不整合（どのsubdomainのmembersにも属さない）を機械的に検出できなかった反省から実装。
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from check_spec_referential_integrity import check


def _write(path: Path, content: dict):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(content))


def test_全て整合していれば全項目が空(tmp_path):
    _write(tmp_path / "bc.json", {
        "content": {"members": {"items": [
            {"kind": "subdomain", "members": "sd-a"},
            {"kind": "usecase", "members": "uc-1"},
        ]}}
    })
    _write(tmp_path / "subdomain/sd-a/sd-a.json", {"content": {"members": {"items": ["uc-1"]}}})
    _write(tmp_path / "subdomain/sd-a/usecase/uc-1.json", {})

    result = check(str(tmp_path / "bc.json"))
    assert all(v == [] for v in result.values())


def test_宙に浮いたusecaseを検出する(tmp_path):
    """bc.membersが宣言するusecaseが、どのsubdomainのmembersにも属さない場合を検出する。"""
    _write(tmp_path / "bc.json", {
        "content": {"members": {"items": [
            {"kind": "subdomain", "members": "sd-a"},
            {"kind": "usecase", "members": "uc-1 / uc-orphan"},
        ]}}
    })
    _write(tmp_path / "subdomain/sd-a/sd-a.json", {"content": {"members": {"items": ["uc-1"]}}})
    _write(tmp_path / "subdomain/sd-a/usecase/uc-1.json", {})

    result = check(str(tmp_path / "bc.json"))
    assert result["usecases_orphaned_no_subdomain"] == ["uc-orphan"]


def test_subdomainが持つがbcに未宣言のusecaseを検出する(tmp_path):
    _write(tmp_path / "bc.json", {
        "content": {"members": {"items": [
            {"kind": "subdomain", "members": "sd-a"},
            {"kind": "usecase", "members": "uc-1"},
        ]}}
    })
    _write(tmp_path / "subdomain/sd-a/sd-a.json", {"content": {"members": {"items": ["uc-1", "uc-phantom"]}}})
    _write(tmp_path / "subdomain/sd-a/usecase/uc-1.json", {})

    result = check(str(tmp_path / "bc.json"))
    assert result["usecases_in_subdomain_not_declared_in_bc"] == ["uc-phantom"]


def test_ファイルが実在しないusecaseを検出する(tmp_path):
    _write(tmp_path / "bc.json", {
        "content": {"members": {"items": [
            {"kind": "subdomain", "members": "sd-a"},
            {"kind": "usecase", "members": "uc-missing"},
        ]}}
    })
    _write(tmp_path / "subdomain/sd-a/sd-a.json", {"content": {"members": {"items": ["uc-missing"]}}})
    # uc-missing.json をわざと作らない

    result = check(str(tmp_path / "bc.json"))
    assert result["usecase_files_missing_on_disk"] == ["uc-missing"]
