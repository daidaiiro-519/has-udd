# test-standard-waffle

---

## テスト方針

- **方針**: test-standard-python-hexagonalを前提とし、それに追加でWaffle固有のspec↔テスト対応規約を上乗せする
- **根拠**: Waffleはspec(DomainSpecSchema)のTestScenariosを実行可能テストへ機械的に束ねるUDD(Usage-Driven Development)ループを持つ。この対応関係はWaffle固有の仕組みであり、汎用のPython×Hexagonal規約(test-standard-python-hexagonal)には含めない

---

## テストタイプ

| テストタイプ | ツール | 対象 |
|---|---|---|
| `acceptance` | pytest（AIがspecのTestScenariosを見て執筆） | usecase |

---

## フレームワーク

- **単体テスト**: pytest
- **受け入れテスト**: pytest

---

## シナリオの束ね方

| 項目 | 規約 |
|---|---|
| 対応関係 | 1 spec（DomainSpecSchemaのusecase）＝1 ネイティブテストファイル（AI執筆） |
| ネイティブテストの配置 | tests/acceptance/test_{documentId}.py（documentIdはspecのdocumentIdをそのままsnake_case化） |
| ドリフト検知 | シナリオ名⇔テスト関数名の名前突き合わせで機械検出（check-scenario-drift）。未実装/孤立を検出し、中身の妥当性はAIが評価する |
| シナリオブロック種別とテスト配置層の対応 | invariantScenarios(aggregate)→domain/unit、domainServiceScenarios(subdomain)→domain/unit、guaranteeScenarios(usecase・operationGuaranteesと対)→integration、acceptanceScenarios(usecase)→acceptance。コードの性質(純粋かport必須か)をケースバイケースで判定してはならない（ドリフト検知を非決定的にするため） |
| シナリオ文言の追従 | specのGuaranteeScenarios/AcceptanceScenarios/InvariantScenarios/DomainServiceScenariosに対応するテスト関数は、対応するgherkinのGiven/When/Thenをdocstringに転記する（関数名の一致だけでは、シナリオ文言の事後編集に対する追従を検知できないため） |

---

## テスト対象別の配置

| 対象 | テスト種別 | 配置 |
|---|---|---|
| domain | unit | `tests/unit/domain/` |
| application | unit（port はテストダブル） | `tests/unit/application/` |
| adapters | integration | `tests/integration/` |
| usecase | acceptance（ネイティブ） | `tests/acceptance/` |
| CLI / MCP の公開インターフェース | contract | `tests/contract/` |

---

## 決定ルール

| 種別 | 規約 |
|---|---|
| 必須 | テストファイル名は test_{対応するspecのdocumentIdをsnake_case化したもの}.py で統一する（domain/application/adapters(integration)/acceptanceの全層に適用。実装モジュール名を由来にした命名は禁止） |
| 必須 | DomainSpecSchemaのシナリオブロック種別とテスト配置層は機械的に対応する（scenarioBinding参照） |
| 禁止 | spec側(DomainSpecSchema等)のブロック名・シナリオの記述に、アーキテクチャ/テスト層の用語（unit/integration/adapter/render_engine等の内部コンポーネント名）を持ち込む。specは常にDDD/業務語彙のみで書く |
