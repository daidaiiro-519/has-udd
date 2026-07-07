"""uc-check-schema-version-drift の受け入れテスト（ネイティブpytest）。"""
import json
from pathlib import Path

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_schema_version_drift_engine import CheckSchemaVersionDriftEngine
from waffle.shared.result import Ok


_EMPTY_SCHEMA = {"properties": {"content": {"type": "object", "properties": {}}}}


class _FakeSchemaRepository:
    def __init__(self, versions_by_name: dict[str, list[str]], schemas_by_ref: dict[str, dict] | None = None) -> None:
        self._versions_by_name = versions_by_name
        self._schemas_by_ref = schemas_by_ref or {}

    def load(self, schema_ref: str) -> dict:
        return self._schemas_by_ref.get(schema_ref, _EMPTY_SCHEMA)

    def list_versions(self, name: str) -> list[str]:
        return self._versions_by_name.get(name, [])


def _engine(versions_by_name: dict[str, list[str]], schemas_by_ref: dict[str, dict] | None = None) -> CheckSchemaVersionDriftEngine:
    return CheckSchemaVersionDriftEngine(FsDocumentRepository(), _FakeSchemaRepository(versions_by_name, schemas_by_ref))


def _write(path: Path, doc: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(doc, ensure_ascii=False), encoding="utf-8")


def test_全Documentが最新版を参照しているとき差分なしと判定する(tmp_path):
    """
    Given 全DocumentのschemaRefが、実在する同名Schemaの最新版を指しているspecツリー
    When schema版ドリフト検査を実行する
    Then broken_references・newer_version_available・missing_declared_fields全てが空配列で返る
    """
    _write(tmp_path / "doc-a.json", {"documentId": "doc-a", "schemaRef": "FooSchema/v2"})

    result = _engine({"FooSchema": ["v1", "v2"]}).run(str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value == {"broken_references": [], "newer_version_available": [], "missing_declared_fields": []}


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


def test_Schemaが宣言する値フィールドをDocumentが持たないことを検出する(tmp_path):
    """
    Given 参照先Schemaが宣言する値フィールドのキーを実データに持たないDocument
    When schema版ドリフト検査を実行する
    Then missing_declared_fieldsにその組が含まれる
    """
    schema = {
        "properties": {
            "content": {
                "type": "object",
                "required": ["note"],
                "properties": {
                    "note": {"type": "string", "x-prompt-write": "備考"},
                },
            }
        }
    }
    _write(tmp_path / "doc-a.json", {"documentId": "doc-a", "schemaRef": "FooSchema/v2", "content": {}})

    result = _engine({"FooSchema": ["v2"]}, {"FooSchema/v2": schema}).run(str(tmp_path))
    assert isinstance(result, Ok), result
    assert result.value["missing_declared_fields"] == [
        {"document": str(tmp_path / "doc-a.json"), "path": "content.note"}
    ]
