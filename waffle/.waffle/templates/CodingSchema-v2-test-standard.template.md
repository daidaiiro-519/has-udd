# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{testStrategy.pyramid}}` | 重心の方針。例: ピラミッド形（単体テストを重視・統合はやや軽め・E2E は行わない） |
| `{{testStrategy.rationale}}` | その方針を選ぶ根拠。例: 業務ロジックの実装方法＝ドメインモデル |
| `{{testPlan.items[1].item}}` | 計画項目名。例: 実行タイミング |
| `{{testPlan.items[1].value}}` | 内容。 |
| `{{testTypes.items[1].testType}}` | 共通カタログの固定語彙から選ぶ。 |
| `{{testTypes.items[1].tool}}` | 使用ツール。例: pytest |
| `{{testTypes.items[1].target}}` | 対象。例: domain / application |
| `{{framework.unit}}` | 単体テストFW。例: pytest |
| `{{framework.acceptance}}` | 受け入れテストFW（仕様(要件)をこのFWでネイティブに実行可能テストとして執筆する）。無ければ空。例: pytest |
| `{{scenarioBinding.items[1].item}}` | 項目名。例: 対応関係 |
| `{{scenarioBinding.items[1].rule}}` | 規約の内容。 |
| `{{placementByTarget.items[1].target}}` | テスト対象。例: domain |
| `{{placementByTarget.items[1].testKind}}` | テスト種別。例: 単体 |
| `{{placementByTarget.items[1].path}}` | 配置パス。例: tests/domain/ |
| `{{rules.items[1].rule}}` | 規約の内容。 |

---

# {{title.title}}

---

## テスト方針

- **方針**: {{testStrategy.pyramid}}
- **根拠**: {{testStrategy.rationale}}

---

## テスト計画

| 項目 | 値 |
|---|---|
| {{testPlan.items[1].item}} | {{testPlan.items[1].value}} |

---

## テストタイプ

| テストタイプ | ツール | 対象 |
|---|---|---|
| `{{testTypes.items[1].testType}}` | {{testTypes.items[1].tool}} | {{testTypes.items[1].target}} |

---

## フレームワーク

- **単体テスト**: {{framework.unit}}
- **受け入れテスト**: {{framework.acceptance}}

---

## シナリオの束ね方

| 項目 | 規約 |
|---|---|
| {{scenarioBinding.items[1].item}} | {{scenarioBinding.items[1].rule}} |

---

## テスト対象別の配置

| 対象 | テスト種別 | 配置 |
|---|---|---|
| {{placementByTarget.items[1].target}} | {{placementByTarget.items[1].testKind}} | `{{placementByTarget.items[1].path}}` |

---

## 決定ルール

| 種別 | 規約 |
|---|---|
| 必須 | {{rules.items[1].rule}} |
