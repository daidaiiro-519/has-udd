# uc-check-scenario-drift

---

## 概要

spec の TestScenarios（acceptanceScenarios/guaranteeScenarios/invariantScenarios/domainServiceScenarios）が宣言するシナリオ名と、対応するテストファイルの test_* 関数名を突き合わせ、未実装のシナリオ・宣言に対応しない孤立テストを機械的に検出する。

---

## 主アクターと意図

- **主アクター**: Orchestrator（HarnessAgent）
- **意図**: specのシナリオ宣言とテストコードの対応関係が保たれているかを確認したい

---

## 関与する外部

- DocumentRepository（spec.json・テストファイルの読込に使う既存 port）

---

## 事前条件

- 対象 spec.json のパスと、対応するテストファイル(.py)のパスが両方与えられている

---

## 基本フロー

```mermaid
sequenceDiagram
    Orchestrator->>SpecTree: spec.jsonとテストファイルのパスを指定してドリフト検査を依頼する
    SpecTree->>SpecTree: spec.jsonの4種のシナリオブロックからシナリオ名を集め、非単語文字を_に置換しtest_を前置した名前の集合に変換する（sanitize）
    SpecTree->>SpecTree: テストファイルをASTで解析し、test_で始まる関数定義の名前を集める
    SpecTree->>SpecTree: 2つの名前集合を突き合わせる
    SpecTree-->>Orchestrator: missing_in_tests・orphaned_in_tests・matchedの3つの文字列配列を返す
```

---

## 事後条件

- 返り値は次の3フィールドを持つ: missing_in_tests（specが宣言するが対応するテスト関数が無いシナリオ名）・orphaned_in_tests（テストファイルにあるがどのシナリオ宣言にも対応しない関数名）・matched（両方に存在する名前）
- 対象となるシナリオブロックはacceptanceScenarios/guaranteeScenarios/invariantScenarios/domainServiceScenariosの4種
- シナリオ名からテスト関数名への変換規則（sanitize）は、非単語文字を_に置換しtest_を前置する
- テスト関数名の抽出はASTのみで行い、実行や意味理解はしない
- missing_in_tests・orphaned_in_tests両方が空配列であれば、シナリオとテストの対応関係が保たれている（正常系）

---

## 受け入れ基準

- When specが宣言するシナリオに対応するテスト関数がテストファイルに無いとき、エンジンはそのシナリオ名（sanitize後）をmissing_in_testsに含める shall。
- When テストファイルの関数が、どのシナリオ宣言にも対応しないとき、エンジンはその関数名をorphaned_in_testsに含める shall。
- When シナリオ名（sanitize後）とテスト関数名が一致するとき、エンジンはその名前をmatchedに含める shall。
- While 全シナリオに対応するテストが存在し孤立テストも無いとき、エンジンはmissing_in_tests・orphaned_in_tests両方を空配列で返す shall。
- If 対象のspec.jsonまたはテストファイルが存在しないとき、エンジンはINVALID_PATHエラーを返す shall。
- If テストファイルが構文解析できないとき、エンジンはINVALID_SOURCEエラーを返す shall。

---

## 操作保証

- When 対象のspec.jsonまたはテストファイルが存在しないとき、engine は INVALID_PATH エラーを返す shall（リポジトリによる解決プロセス自体の契約・DocumentRepositoryを介して判定する）。

---

## エラー

| コード | 条件 |
|---|---|
| `INVALID_SOURCE` | 対象のテストファイルが構文解析できない（Pythonのパーサでエラー） |

---

## 受け入れシナリオ

### 全シナリオに対応するテストがあり孤立も無いとき整合していると判定する

| 分類 | 観点 |
|---|---|
| 正常系 | 整合：missing/orphaned両方空配列は正常系 |

```gherkin
Scenario: 全シナリオに対応するテストがあり孤立も無いとき整合していると判定する
  Given 宣言する全シナリオに対応するtest_*関数を持つテストファイル
  When ドリフト検査を実行する
  Then missing_in_tests・orphaned_in_tests両方が空配列で返る
```

### 宣言されたシナリオに対応するテストが無いことを検出する

| 分類 | 観点 |
|---|---|
| 異常系 | ドリフト：宣言はあるがテスト未実装 |

```gherkin
Scenario: 宣言されたシナリオに対応するテストが無いことを検出する
  Given シナリオを宣言するが対応するtest_*関数を持たないテストファイル
  When ドリフト検査を実行する
  Then missing_in_testsにそのシナリオ名（sanitize後）が含まれる
```

### 宣言に対応しないテスト関数を検出する

| 分類 | 観点 |
|---|---|
| 異常系 | ドリフト：テストはあるが宣言が無い |

```gherkin
Scenario: 宣言に対応しないテスト関数を検出する
  Given どのシナリオ宣言にも対応しないtest_*関数を含むテストファイル
  When ドリフト検査を実行する
  Then orphaned_in_testsにその関数名が含まれる
```

### 構文解析できないテストファイルはINVALID_SOURCE

| 分類 | 観点 |
|---|---|
| 異常系 | エラー：テストファイルの構文エラー |

```gherkin
Scenario: 構文解析できないテストファイルはINVALID_SOURCE
  Given 構文が壊れたテストファイル
  When ドリフト検査を実行する
  Then INVALID_SOURCEエラーが返る
```

---

## 操作保証シナリオ

### 存在しないspec.jsonはINVALID_PATH

| 分類 | 観点 |
|---|---|
| 異常系 | エラー：spec.jsonの不在 |

```gherkin
Scenario: 存在しないspec.jsonはINVALID_PATH
  When 存在しないspec.jsonのパスでドリフト検査を実行する
  Then INVALID_PATHエラーが返る
```

### 存在しないテストファイルはINVALID_PATH

| 分類 | 観点 |
|---|---|
| 異常系 | エラー：テストファイルの不在 |

```gherkin
Scenario: 存在しないテストファイルはINVALID_PATH
  When 存在しないテストファイルのパスでドリフト検査を実行する
  Then INVALID_PATHエラーが返る
```
