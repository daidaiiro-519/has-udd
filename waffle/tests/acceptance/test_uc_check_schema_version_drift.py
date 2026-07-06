"""uc-check-schema-version-drift の受け入れテスト（ネイティブpytest）。"""
import json
from pathlib import Path

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_schema_version_drift_engine import CheckSchemaVersionDriftEngine
from waffle.shared.result import Ok


class _FakeSchemaRepository:
    def __init__(self, versions_by_name: dict[str, list[str]]) -> None:
        self._versions_by_name = versions_by_name

    def load(self, schema_ref: str) -> dict:
        raise NotImplementedError

    def list_versions(self, name: str) -> list[str]:
        return self._versions_by_name.get(name, [])


def _engine(versions_by_name: dict[str, list[str]]) -> CheckSchemaVersionDriftEngine:
    return CheckSchemaVersionDriftEngine(FsDocumentRepository(), _FakeSchemaRepository(versions_by_name))


def _write(path: Path, doc: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(doc, ensure_ascii=False), encoding="utf-8")


def test_全Documentが最新版を参照しているとき差分なしと判定する(tmp_path):
    """
    Given 全DocumentのschemaRefが、実在する同名Schemaの最新版を指しているspecツリー
    When schema版ドリフト検査を実行する
    Then broken_references・newer_version_available共に空配列で返る
    """
    _write(tmp_path / "doc-a.json", {"documentId": "doc-a", "schemaRef": "FooSchema/v2"})

    result = _engine({"FooSchema": ["v1", "v2"]}).run(str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value == {"broken_references": [], "newer_version_available": []}


def test_実在しない版を指すschemaRefを検出する(tmp_path):
    """
    Given 実在しない版をschemaRefに持つDocument
    When schema版ドリフト検査を実行する
    Then broken_referencesにその組が含まれる
    """
    _write(tmp_path / "doc-a.json", {"documentId": "doc-a", "schemaRef": "FooSchema/v9"})

    result = _engine({"FooSchema": ["v1", "v2"]}).run(str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["broken_references"] == [{"document": str(tmp_path / "doc-a.json"), "schemaRef": "FooSchema/v9"}]


def test_最新でない版を参照しているDocumentを検出する(tmp_path):
    """
    Given 同名Schemaに新しい版が実在するが、旧い版をschemaRefに持つDocument
    When schema版ドリフト検査を実行する
    Then newer_version_availableにその組が含まれる
    """
    _write(tmp_path / "doc-a.json", {"documentId": "doc-a", "schemaRef": "FooSchema/v1"})

    result = _engine({"FooSchema": ["v1", "v2"]}).run(str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["newer_version_available"] == [
        {"document": str(tmp_path / "doc-a.json"), "schemaRef": "FooSchema/v1", "latest": "FooSchema/v2"}
    ]
