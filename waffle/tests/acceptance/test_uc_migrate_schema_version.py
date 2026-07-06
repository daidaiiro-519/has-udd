"""uc-migrate-schema-version の acceptanceScenarios のうち、publishVersion/
deprecateVersionに対応するネイティブテスト（実FsDocumentRepository結合）。

prepareMigration/applyMigration（x-migration処理）は、x-migration宣言を持つ実schemaが
バンドル済みschema群にまだ存在しないため、既知の例外としてtests/unit/application/へ
フェイクschemaで固定してある（詳細はそちらのdocstring参照）。
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


