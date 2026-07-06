"""uc-check-spec-integrity の受け入れテスト（ネイティブpytest）。"""
import json
from pathlib import Path

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_spec_integrity_engine import CheckSpecIntegrityEngine
from waffle.shared.result import Ok


def _engine() -> CheckSpecIntegrityEngine:
    return CheckSpecIntegrityEngine(FsDocumentRepository())


def _write(path: Path, doc: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(doc, ensure_ascii=False, indent=2), encoding="utf-8")


def _bc(members_subdomain: str, members_usecase: str) -> dict:
    return {
        "documentId": "bc-test",
        "content": {
            "members": {
                "items": [
                    {"kind": "subdomain", "members": members_subdomain},
                    {"kind": "usecase", "members": members_usecase},
                ]
            }
        },
    }


def _sd(name: str, usecases: list[str]) -> dict:
    return {"documentId": name, "content": {"members": {"items": usecases}}}


def test_全ての宣言と実態が一致するとき差分なしと判定する(tmp_path):
    """
    Given bc.jsonの宣言とディスク上の実ファイルが完全に一致するspecツリー
    When 参照整合性検査を実行する
    Then 10フィールド全てが空配列で返る
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a"})

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value == {
        "declared_subdomains_missing_on_disk": [],
        "subdomains_on_disk_not_declared_in_bc": [],
        "usecases_orphaned_no_subdomain": [],
        "usecases_in_subdomain_not_declared_in_bc": [],
        "usecase_files_missing_on_disk": [],
        "usecase_files_orphaned_on_disk": [],
        "orphaned_value_objects": [],
        "undeclared_document_fields": [],
        "subdomain_ref_mismatches": [],
        "missing_aggregate_refs": [],
    }


def test_宣言されたsubdomainがディスクに無いことを検出する(tmp_path):
    """
    Given bc.jsonがsubdomainを宣言するが、そのディレクトリが実在しないspecツリー
    When 参照整合性検査を実行する
    Then declared_subdomains_missing_on_diskにその名前が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a / sd-ghost", "uc-a"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a"})

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["declared_subdomains_missing_on_disk"] == ["sd-ghost"]


def test_未宣言のsubdomainがディスクにあることを検出する(tmp_path):
    """
    Given ディスク上に実在するがbc.jsonに宣言されていないsubdomainを含むspecツリー
    When 参照整合性検査を実行する
    Then subdomains_on_disk_not_declared_in_bcにその名前が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a"})
    _write(tmp_path / "subdomain/sd-undeclared/sd-undeclared.json", _sd("sd-undeclared", []))

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["subdomains_on_disk_not_declared_in_bc"] == ["sd-undeclared"]


def test_どのsubdomainにも属さない宙に浮いたusecaseを検出する(tmp_path):
    """
    Given bc.jsonがusecaseを宣言するが、どのsubdomainのmembersにも含まれないspecツリー
    When 参照整合性検査を実行する
    Then usecases_orphaned_no_subdomainにその名前が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a / uc-orphan"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a"})

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["usecases_orphaned_no_subdomain"] == ["uc-orphan"]


def test_subdomainには属するがbcに未宣言のusecaseを検出する(tmp_path):
    """
    Given いずれかのsubdomainのmembersが宣言するがbc.jsonには宣言されていないusecaseを含むspecツリー
    When 参照整合性検査を実行する
    Then usecases_in_subdomain_not_declared_in_bcにその名前が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a", "uc-hidden"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a"})
    _write(tmp_path / "subdomain/sd-a/usecase/uc-hidden.json", {"documentId": "uc-hidden"})

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["usecases_in_subdomain_not_declared_in_bc"] == ["uc-hidden"]


def test_宣言されたusecaseの実ファイルが無いことを検出する(tmp_path):
    """
    Given subdomainがusecaseを宣言するが、対応するjsonファイルが実在しないspecツリー
    When 参照整合性検査を実行する
    Then usecase_files_missing_on_diskにその名前が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a / uc-missing"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a", "uc-missing"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a"})

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["usecase_files_missing_on_disk"] == ["uc-missing"]


def test_未宣言のusecaseファイルがディスクにあることを検出する(tmp_path):
    """
    Given ディスク上に実在するがどのsubdomainのmembersにも宣言されていないusecaseファイルを含むspecツリー
    When 参照整合性検査を実行する
    Then usecase_files_orphaned_on_diskにその名前が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a"})
    _write(tmp_path / "subdomain/sd-a/usecase/uc-stray.json", {"documentId": "uc-stray"})

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["usecase_files_orphaned_on_disk"] == ["uc-stray"]


def _agg_document(entity_attributes: list[dict], value_objects: list[dict]) -> dict:
    return {
        "documentId": "agg-document",
        "content": {
            "entities": {"items": [{"name": "Document", "attributes": entity_attributes}]},
            "valueObjects": {"items": value_objects},
        },
    }


def test_使われていない値オブジェクトを検出する(tmp_path):
    """
    Given valueObjectsに宣言されているが、entities[].attributes[].typeのどこにも現れない値オブジェクトを含む集約document
    When 参照整合性検査を実行する
    Then orphaned_value_objectsにその値オブジェクト名が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a"})
    _write(tmp_path / "aggregate/agg-document.json", _agg_document(
        entity_attributes=[{"name": "documentId", "type": "DocumentId"}],
        value_objects=[{"name": "DocumentId"}, {"name": "GhostValueObject"}],
    ))

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["orphaned_value_objects"] == ["GhostValueObject"]


def test_実document_jsonにある未宣言のフィールドを検出する(tmp_path):
    """
    Given トップレベルにDocument集約のentity属性に宣言されていないフィールドを持つ実document.json
    When 参照整合性検査を実行する
    Then undeclared_document_fieldsにそのフィールド名が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a"})
    _write(tmp_path / "aggregate/agg-document.json", _agg_document(
        entity_attributes=[{"name": "documentId", "type": "DocumentId"}],
        value_objects=[{"name": "DocumentId"}],
    ))
    _write(tmp_path / "real_doc.json", {"documentId": "x", "unexpectedField": "value"})

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert "unexpectedField" in result.value["undeclared_document_fields"]


def test_subdomainRefの食い違いを検出する(tmp_path):
    """
    Given subdomainRefが指すsubdomainのmembersに自分自身が含まれていないusecase document
    When 参照整合性検査を実行する
    Then subdomain_ref_mismatchesにその組が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a", "subdomainRef": "sd-wrong"})

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert {"usecase": "uc-a", "subdomainRef": "sd-wrong"} in result.value["subdomain_ref_mismatches"]


def test_存在しない集約を指すaggregateRefを検出する(tmp_path):
    """
    Given 実在しない集約documentIdをaggregateRefに持つusecase document
    When 参照整合性検査を実行する
    Then missing_aggregate_refsにその組が含まれる
    """
    bc_path = tmp_path / "bc-test.json"
    _write(bc_path, _bc("sd-a", "uc-a"))
    _write(tmp_path / "subdomain/sd-a/sd-a.json", _sd("sd-a", ["uc-a"]))
    _write(tmp_path / "subdomain/sd-a/usecase/uc-a.json", {"documentId": "uc-a", "aggregateRef": "agg-ghost"})

    result = _engine().run(str(bc_path), str(tmp_path))
    assert isinstance(result, Ok), result
    assert {"usecase": "uc-a", "aggregateRef": "agg-ghost"} in result.value["missing_aggregate_refs"]
