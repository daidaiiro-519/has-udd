"""uc-check-scenario-drift の受け入れテスト（ネイティブpytest）。"""
import json
from pathlib import Path

from waffle.adapters.outbound.fs import FsDocumentRepository
from waffle.application.usecases.check_scenario_drift_engine import CheckScenarioDriftEngine
from waffle.shared.result import Err, Ok


def _engine() -> CheckScenarioDriftEngine:
    return CheckScenarioDriftEngine(FsDocumentRepository())


def _spec(tmp_path: Path, scenario_name: str, gherkin: str) -> Path:
    path = tmp_path / "spec.json"
    doc = {
        "documentId": "test-spec",
        "content": {
            "acceptanceScenarios": {
                "scenarios": [
                    {"name": scenario_name, "gherkin": gherkin, "category": "正常系", "viewpoint": "", "covers": ""}
                ]
            }
        },
    }
    path.write_text(json.dumps(doc, ensure_ascii=False), encoding="utf-8")
    return path


def test_全シナリオに対応するテストがあり孤立も無いとき整合していると判定する(tmp_path):
    """
    Given 宣言する全シナリオに対応するtest_*関数を持ち、docstringがgherkin本文と一致するテストファイル
    When ドリフト検査を実行する
    Then missing_in_tests・orphaned_in_tests・gherkin_mismatches全てが空配列で返る
    """
    spec_path = _spec(tmp_path, "何かが起きる",
                       "Scenario: 何かが起きる\n  Given 前提\n  When 操作する\n  Then 結果になる")
    test_path = tmp_path / "test_something.py"
    test_path.write_text(
        'def test_何かが起きる():\n'
        '    """\n'
        '    Given 前提\n'
        '    When 操作する\n'
        '    Then 結果になる\n'
        '    """\n'
        '    pass\n',
        encoding="utf-8",
    )

    result = _engine().run(str(spec_path), str(test_path))
    assert isinstance(result, Ok), result
    assert result.value == {
        "missing_in_tests": [],
        "orphaned_in_tests": [],
        "matched": ["test_何かが起きる"],
        "gherkin_mismatches": [],
    }


def test_宣言されたシナリオに対応するテストが無いことを検出する(tmp_path):
    """
    Given シナリオを宣言するが対応するtest_*関数を持たないテストファイル
    When ドリフト検査を実行する
    Then missing_in_testsにそのシナリオ名（sanitize後）が含まれる
    """
    spec_path = _spec(tmp_path, "何かが起きる",
                       "Scenario: 何かが起きる\n  Given 前提\n  When 操作する\n  Then 結果になる")
    test_path = tmp_path / "test_something.py"
    test_path.write_text("def test_別のもの():\n    pass\n", encoding="utf-8")

    result = _engine().run(str(spec_path), str(test_path))
    assert isinstance(result, Ok), result
    assert result.value["missing_in_tests"] == ["test_何かが起きる"]


def test_宣言に対応しないテスト関数を検出する(tmp_path):
    """
    Given どのシナリオ宣言にも対応しないtest_*関数を含むテストファイル
    When ドリフト検査を実行する
    Then orphaned_in_testsにその関数名が含まれる
    """
    spec_path = _spec(tmp_path, "何かが起きる",
                       "Scenario: 何かが起きる\n  Given 前提\n  When 操作する\n  Then 結果になる")
    test_path = tmp_path / "test_something.py"
    test_path.write_text(
        'def test_何かが起きる():\n'
        '    """\n'
        '    Given 前提\n'
        '    When 操作する\n'
        '    Then 結果になる\n'
        '    """\n'
        '    pass\n\n\n'
        'def test_孤立している():\n'
        '    pass\n',
        encoding="utf-8",
    )

    result = _engine().run(str(spec_path), str(test_path))
    assert isinstance(result, Ok), result
    assert result.value["orphaned_in_tests"] == ["test_孤立している"]


def test_docstringがgherkin本文と一致しないシナリオを検出する(tmp_path):
    """
    Given シナリオに対応するtest_*関数を持つが、docstringの内容がgherkin本文と異なるテストファイル
    When ドリフト検査を実行する
    Then gherkin_mismatchesにそのシナリオ名が含まれる
    """
    spec_path = _spec(tmp_path, "何かが起きる",
                       "Scenario: 何かが起きる\n  Given 前提\n  When 操作する\n  Then 結果になる")
    test_path = tmp_path / "test_something.py"
    test_path.write_text(
        'def test_何かが起きる():\n'
        '    """\n'
        '    Given 全く違う前提\n'
        '    When 全く違う操作をする\n'
        '    Then 全く違う結果になる\n'
        '    """\n'
        '    pass\n',
        encoding="utf-8",
    )

    result = _engine().run(str(spec_path), str(test_path))
    assert isinstance(result, Ok), result
    assert result.value["gherkin_mismatches"] == ["test_何かが起きる"]


def test_構文解析できないテストファイルはINVALID_SOURCE(tmp_path):
    """
    Given 構文が壊れたテストファイル
    When ドリフト検査を実行する
    Then INVALID_SOURCEエラーが返る
    """
    spec_path = _spec(tmp_path, "何かが起きる",
                       "Scenario: 何かが起きる\n  Given 前提\n  When 操作する\n  Then 結果になる")
    test_path = tmp_path / "test_broken.py"
    test_path.write_text("def test_壊れている(:\n    pass\n", encoding="utf-8")

    result = _engine().run(str(spec_path), str(test_path))
    assert isinstance(result, Err), result
    assert result.details[0] == "INVALID_SOURCE"
