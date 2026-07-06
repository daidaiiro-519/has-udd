"""uc-migrate-schema-version の受け入れテスト（ネイティブpytest・実adapter結合）。

.waffle/specs/.../uc-migrate-schema-version.feature は参照専用の仕様書であり、実行対象ではない。
publishVersion/deprecateVersionは実際のFsDocumentRepositoryでファイルを読み書きする。
prepareMigrationは実際のPackageSchemaRepository（importlib.resources経由）でschemaを解決する
（toSchemaRefは実在のbundled schemaを使う。x-migration宣言を持つ実schemaはまだ無いため、
機械変換0件の素通りとschemaRef解決自体の実証にとどめる）。
"""
import json

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.adapters.outbound.jsonschema_validator import JsonSchemaValidator
from waffle.adapters.outbound.schema_repo import PackageSchemaRepository
from waffle.application.usecases.migration_engine import MigrationEngine
from waffle.shared.result import Err, Ok


def _engine() -> MigrationEngine:
    return MigrationEngine(FsDocumentRepository(), PackageSchemaRepository(), JsonSchemaValidator())


def test_publishVersionは未公開のschemaをPUBLISHEDにする(tmp_path):
    """
    Given x-schema-statusが未設定のSchemaファイル
    When publishVersionを実行する
    Then x-schema-statusがPUBLISHEDになる
    """
    schema_path = tmp_path / "schema.json"
    schema_path.write_text(json.dumps({"type": "object"}))

    result = _engine().run("publishVersion", {"schemaPath": str(schema_path)})
    assert isinstance(result, Ok), result
    assert json.loads(schema_path.read_text())["x-schema-status"] == "PUBLISHED"


def test_publishVersionは既に公開済みのschemaを拒否する(tmp_path):
    """
    Given x-schema-statusが既に設定されたSchemaファイル
    When publishVersionを実行する
    Then ALREADY_PUBLISHEDエラーが返る
    """
    schema_path = tmp_path / "schema.json"
    schema_path.write_text(json.dumps({"x-schema-status": "PUBLISHED"}))

    result = _engine().run("publishVersion", {"schemaPath": str(schema_path)})
    assert isinstance(result, Err), result
    assert result.details[0] == "ALREADY_PUBLISHED"


def test_deprecateVersionはPUBLISHEDをDEPRECATEDにする(tmp_path):
    """
    Given PUBLISHEDのSchemaファイル
    When deprecateVersionを実行する
    Then x-schema-statusがDEPRECATEDになる
    """
    schema_path = tmp_path / "schema.json"
    schema_path.write_text(json.dumps({"x-schema-status": "PUBLISHED"}))

    result = _engine().run("deprecateVersion", {"schemaPath": str(schema_path)})
    assert isinstance(result, Ok), result
    assert json.loads(schema_path.read_text())["x-schema-status"] == "DEPRECATED"


def test_deprecateVersionはPUBLISHED以外を拒否する(tmp_path):
    """
    Given PUBLISHED以外の状態のSchemaファイル
    When deprecateVersionを実行する
    Then INVALID_STATEエラーが返る
    """
    schema_path = tmp_path / "schema.json"
    schema_path.write_text(json.dumps({"x-schema-status": "DEPRECATED"}))

    result = _engine().run("deprecateVersion", {"schemaPath": str(schema_path)})
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_STATE"


def test_prepareMigrationは実在しないschemaRefを拒否する(tmp_path):
    """
    Given 解決できないtoSchemaRef
    When prepareMigrationを実行する
    Then INVALID_SCHEMA_REFエラーが返る（実際のPackageSchemaRepositoryで解決を試みる）
    """
    docs_dir = tmp_path / "docs"
    docs_dir.mkdir()

    result = _engine().run("prepareMigration", {
        "fromSchemaRef": "Bogus/v1", "toSchemaRef": "Bogus/v2", "documentsDir": str(docs_dir),
    })
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_SCHEMA_REF"


def test_prepareMigrationは実在のschemaRefを解決しx_migrationが無ければ素通りする(tmp_path):
    """
    Given x-migration宣言を持たない実在のtoSchemaRef(DomainSpecSchema/v2)と、
        古いバージョンを参照するDocument
    When prepareMigrationを実行する
    Then 実際のPackageSchemaRepositoryでschemaが解決され、機械変換もai-inferワークシートも
        0件のまま(schemaRefだけ更新された)partialDocumentが返る
    """
    docs_dir = tmp_path / "docs"
    docs_dir.mkdir()
    (docs_dir / "doc1.json").write_text(json.dumps({"schemaRef": "DomainSpecSchema/v1", "documentId": "x"}))

    result = _engine().run("prepareMigration", {
        "fromSchemaRef": "DomainSpecSchema/v1", "toSchemaRef": "DomainSpecSchema/v2", "documentsDir": str(docs_dir),
    })
    assert isinstance(result, Ok), result
    partial = result.value["partialDocuments"][str(docs_dir / "doc1.json")]
    assert partial["schemaRef"] == "DomainSpecSchema/v2"
    assert result.value["worksheets"] == {}
