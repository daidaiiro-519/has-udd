<!-- test-standard codingKind の「レンダリング後イメージ」v2。TestStrategy/TestPlan復活・TestTypesは全ティア共通カタログ(固定enum)からの選択制＝has-uddはbackend該当行のみ選択→レンダリング(frontend/platform行は書かないので出ない)。ゲート1(シナリオ実行)の定義。 -->

# python-hexagonal — テスト規約

テスト方針・計画、および仕様（spec）の TestScenarios を実行可能な検証に束ねる方法を定める。

---

## テスト方針

| 項目 | 値 |
|---|---|
| ピラミッド形 | 単体テストを重視・統合はやや軽め・エンドツーエンドは行わない |
| 根拠 | 業務ロジックの実装方法＝ドメインモデル |

---

## テスト計画

| 項目 | 値 |
|---|---|
| 実行タイミング | すべての変更（コミット前ローカル・PR で CI） |
| CI トリガー | push / pull_request |
| ゲート | シナリオ（`.feature`）緑が必須（Stage B ゲート1） |
| 対象外 | パフォーマンステスト（中核だが性能要件なし） |

---

## テストタイプ

このプロジェクトが実施するテスト種別（共通カタログから該当分を選択）。

| テストタイプ | ツール | 対象 |
|---|---|---|
| unit | pytest | domain / application |
| integration | pytest | adapters |
| acceptance-bdd | behave | usecase（`.feature`） |
| contract | pytest（スキーマ差分検査） | CLI / MCP の公開インターフェース |

---

## フレームワーク

| 種別 | 選択 |
|---|---|
| 単体テスト | pytest |
| 受け入れテスト（BDD） | behave |

---

## シナリオの束ね方

| 項目 | 規約 |
|---|---|
| `.feature` の生成元 | spec の TestScenarios（手書きしない） |
| 配置 | `features/{spec-id}.feature`（生成物） |
| ステップ実装の配置 | `features/steps/{spec-id}_steps.py` |
| 対応関係 | 1 spec（usecase）＝1 `.feature`＝1 ステップファイル |

---

## テスト対象別の配置

| 対象 | テスト種別 | 配置 |
|---|---|---|
| domain | 単体 | `tests/domain/` |
| application | 単体（port はテストダブル） | `tests/application/` |
| adapters | 結合 | `tests/adapters/` |
| usecase の受け入れ | BDD | `features/` |

---

## 決定ルール

| 種別 | 規約 |
|---|---|
| 必須 | TestScenarios は `.feature` で実行可能にする |
| 必須 | `.feature` は spec から生成する（手書き禁止） |
| 禁止 | 単体テストが実物の DB / 外部サービスに依存する |
| 推奨 | 不変条件はテストダブルなしで検証する |
