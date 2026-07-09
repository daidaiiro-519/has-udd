# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{summary.text}}` | この画面/フローが何のために存在するかを1〜2文で（業務ユースケースの内容の繰り返しでなく、プロダクトとしての意図）。 |
| `{{fields.items[1].name}}` | 項目名。例: パスワード（確認） |
| `{{fields.items[1].inputType}}` | 入力/表示の種別。例: text / number / select / checkbox |
| `{{fields.items[1].required}}` | 入力必須なら true。 |
| `{{fields.items[1].description}}` | 何のための項目か、かつ、なぜ業務ユースケースの入力ではないのかを1文で。 |
| `{{actions.items[1].name}}` | 操作名。例: 注文を確定する |
| `{{actions.items[1].trigger}}` | 操作の契機。例: 「確定」ボタン押下 |
| `{{actions.items[1].description}}` | 何をするかを1文で。 |
| `{{actions.items[1].usecaseRef}}` | この操作が対応する業務ユースケース（DomainSpecSchemaのusecase）の documentId（あれば）。例: uc-place-order。無ければ空文字。 |
| `{{actions.items[1].leadsTo}}` | この操作の実行後に遷移する screen の documentId（あれば）。同じ画面に留まるなら空文字。 |
| `{{componentAcceptanceScenarios.background}}` | 複数シナリオ共通の前提。無ければ空文字。 |
| `{{componentAcceptanceScenarios.scenarios[1].name}}` | シナリオ名（概要）。 |
| `{{componentAcceptanceScenarios.scenarios[1].category}}` | 分類: 正常系 / 異常系 / 境界値。 |
| `{{componentAcceptanceScenarios.scenarios[1].viewpoint}}` | 観点（入力検証/表示条件/活性状態 等）＋検証の狙い。 |
| `{{componentAcceptanceScenarios.scenarios[1].gherkin}}` | このシナリオの Given/When/Then（Scenario: 1つ・実行可能・.feature化）。 |
| `{{visualRef.url}}` | Figma等のURL。 |
| `{{visualRef.note}}` | 補足（バージョン・参照箇所等）。無ければ空文字。 |

---

# {{title.title}}

---

## 概要

{{summary.text}}

---

## 項目（業務ユースケース外）

| 項目 | 種別 | 必須 | 説明（業務ユースケースにない理由） |
|---|---|---|---|
| {{fields.items[1].name}} | {{fields.items[1].inputType}} | {{fields.items[1].required}} | {{fields.items[1].description}} |

---

## 操作

| 操作 | 契機 | 説明 | 対応する業務ユースケース | 遷移先 |
|---|---|---|---|---|
| {{actions.items[1].name}} | {{actions.items[1].trigger}} | {{actions.items[1].description}} | `{{actions.items[1].usecaseRef}}` | `{{actions.items[1].leadsTo}}` |

---

## コンポーネント受け入れシナリオ

### 背景

{{componentAcceptanceScenarios.background}}

### {{componentAcceptanceScenarios.scenarios[1].name}}

| 分類 | 観点 |
|---|---|
| {{componentAcceptanceScenarios.scenarios[1].category}} | {{componentAcceptanceScenarios.scenarios[1].viewpoint}} |

```gherkin
{{componentAcceptanceScenarios.scenarios[1].gherkin}}
```

---

## ビジュアル参照

- **参照先**: {{visualRef.url}}
- **補足**: {{visualRef.note}}
