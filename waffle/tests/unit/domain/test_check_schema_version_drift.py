"""check_schema_version_drift.py（第4のドリフト検知: schema進化によるDocument陳腐化）の単体テスト。"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from check_schema_version_drift import check


def _write(path: Path, content: dict):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(content))


def test_全て整合していれば全項目が空(tmp_path):
    docs = tmp_path / "documents"
    schemas = tmp_path / "schemas"
    _write(schemas / "Foo" / "v2.json", {"x-schema-status": "PUBLISHED"})
    _write(docs / "a.json", {"schemaRef": "Foo/v2"})

    result = check(str(docs), str(schemas))
    assert all(v == [] for v in result.values())


def test_schemaファイルが実在しない参照を検出する(tmp_path):
    docs = tmp_path / "documents"
    schemas = tmp_path / "schemas"
    _write(docs / "a.json", {"schemaRef": "Foo/v1"})
    # schemas配下にFoo/v1.json自体を作らない

    result = check(str(docs), str(schemas))
    assert result["broken_references"] == [{"document": str(docs / "a.json"), "schemaRef": "Foo/v1"}]


def test_DEPRECATED版への参照を検出する(tmp_path):
    docs = tmp_path / "documents"
    schemas = tmp_path / "schemas"
    _write(schemas / "Foo" / "v1.json", {"x-schema-status": "DEPRECATED"})
    _write(schemas / "Foo" / "v2.json", {"x-schema-status": "PUBLISHED"})
    _write(docs / "a.json", {"schemaRef": "Foo/v1"})

    result = check(str(docs), str(schemas))
    assert result["deprecated_references"] == [{"document": str(docs / "a.json"), "schemaRef": "Foo/v1"}]


def test_より新しいバージョンの存在を情報として報告する(tmp_path):
    docs = tmp_path / "documents"
    schemas = tmp_path / "schemas"
    _write(schemas / "Foo" / "v1.json", {"x-schema-status": "PUBLISHED"})
    _write(schemas / "Foo" / "v2.json", {"x-schema-status": "PUBLISHED"})
    _write(docs / "a.json", {"schemaRef": "Foo/v1"})

    result = check(str(docs), str(schemas))
    assert result["newer_version_available"] == [
        {"document": str(docs / "a.json"), "schemaRef": "Foo/v1", "latest": "Foo/v2"}
    ]
    # v1自体はPUBLISHEDなのでdeprecated_referencesには入らない
    assert result["deprecated_references"] == []
