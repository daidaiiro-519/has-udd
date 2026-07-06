# test-standard-python-hexagonal

---

## テスト方針

- **方針**: ピラミッド形（単体テストを重視・統合はやや軽め・E2E は行わない）
- **根拠**: 業務ロジックの実装方法＝ドメインモデル

---

## テスト計画

| 項目 | 値 |
|---|---|
| 実行タイミング | すべての変更（コミット前ローカル・PR で CI） |
| CI トリガー | push / pull_request |
| ゲート | シナリオに対応するネイティブテスト緑が必須（Stage B ゲート1） |
| 対象外 | パフォーマンステスト（中核だが性能要件なし） |

---

## テストタイプ

| テストタイプ | ツール | 対象 |
|---|---|---|
| `unit` | pytest | domain / application |
| `integration` | pytest | adapters |
| `acceptance` | pytest（AIがspecのTestScenariosを読んで直接執筆） | usecase |
| `contract` | pytest（スキーマ差分検査） | CLI / MCP の公開インターフェース |

---

## フレームワーク

- **単体テスト**: pytest
- **受け入れテスト**: pytest

---

## シナリオの束ね方

| 項目 | 規約 |
|---|---|
| ネイティブテストの配置 | tests/acceptance/test_{spec-id}.py（AIがspecのTestScenariosを見て執筆・手編集前提） |
| 対応関係 | 1 spec（usecase）＝1 ネイティブテストファイル（AI執筆） |
| ドリフト検知 | シナリオ名⇔テスト関数名の名前突き合わせで機械検出（未実装/孤立を検出、中身の妥当性はAIが評価） |

---

## テスト対象別の配置

| 対象 | テスト種別 | 配置 |
|---|---|---|
| domain | unit | `tests/unit/domain/` |
| application | unit（port はテストダブル） | `tests/unit/application/（port経由の編成ロジックが独自の分岐/判定を持つ場合のみ。integration層で既に実アダプタ経由に検証済みの保証を、fakeで再検証するためだけに追加しない）` |
| adapters | integration | `tests/integration/` |
| usecase | acceptance（ネイティブ） | `tests/acceptance/` |
| CLI / MCP の公開インターフェース | contract | `tests/contract/（ネイティブpytest。旧features/{cli,mcp}.feature(behave)から移行しtool不一致を解消）` |

---

## 決定ルール

| 種別 | 規約 |
|---|---|
| 禁止 | 単体テストが実物の DB・外部サービスに依存する |
| 推奨 | 不変条件はテストダブルなしで検証する |
| 必須 | テストファイル名は test_{対応するspecのdocumentIdをsnake_case化したもの}.py で統一する（domain/application/adapters(integration)/acceptanceの全層に適用。実装モジュール名を由来にした命名は禁止） |
| 必須 | tests/ 配下は testTypes（unit/integration/acceptance/contract）を第一階層とする。domain/applicationはunit/配下の第二階層（対象層の軸）。adaptersは単体テストではなく統合(integration)テストなので tests/unit/配下に置かず tests/integration/ を独立の第一階層にする（testTypesが明示的にintegrationと分類しているため） |
| 必須 | specのGuaranteeScenarios/AcceptanceScenarios/InvariantScenarios/DomainServiceScenariosに対応するテスト関数は、対応するgherkinのGiven/When/Thenをdocstringに転記する（関数名の一致だけでは、シナリオ文言の事後編集に対する追従を検知できないため） |
| 必須 | DomainSpecSchemaのシナリオブロック種別とテスト配置層は機械的に対応する: invariantScenarios(aggregate)→domain/unit、domainServiceScenarios(subdomain)→domain/unit、guaranteeScenarios(usecase・operationGuaranteesと対)→integration、acceptanceScenarios(usecase)→acceptance。コードの性質(純粋かport必須か)をケースバイケースで判定してはならない（ドリフト検知を非決定的にするため） |
| 禁止 | spec側(DomainSpecSchema等)のブロック名・シナリオの記述に、アーキテクチャ/テスト層の用語（unit/integration/adapter/render_engine等の内部コンポーネント名）を持ち込む。specは常にDDD/業務語彙のみで書く。「どう検証するか」はtest-standard(コーディング側)にのみ書く |
| 推奨 | application層のport経由コードは、integration層(実アダプタ)で既に検証済みの保証を、fakeに差し替えて再検証するためだけの単体テストを追加しない。追加するのは、そのコード自身が独自の分岐/判定ロジックを持ち、かつどの既存シナリオにも対応しない場合のみ。対応しないテストを追加する前に、まず対応するSpecシナリオが要るかを検討する（テストカバレッジは目標ではなく診断ツール。空のフォルダを埋めるためだけにテストを書かない） |
