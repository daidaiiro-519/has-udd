# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{guaranteeScenarios.background}}` | 複数シナリオ共通の前提。無ければ空文字。 |
| `{{guaranteeScenarios.scenarios[1].name}}` | シナリオ名（概要）。 |
| `{{guaranteeScenarios.scenarios[1].category}}` | 分類: 正常系 / 異常系 / 境界値。 |
| `{{guaranteeScenarios.scenarios[1].viewpoint}}` | 観点: 何を保証するか＋検証の狙い。 |
| `{{guaranteeScenarios.scenarios[1].gherkin}}` | Given/When/Then。ドメイン語彙で書き、IaCの実装詳細は書かない。 |
| `{{guaranteeScenarios.scenarios[1].covers}}` | 対応する保証項目への参照。 |

---

# {{documentId}}

---

## 概要

---

## {{documentId}}

| 指標 | 目標値 | 根拠 |
|---|---|---|
|  |  |  |

---

## リリース/環境昇格方針

| 環境 | 昇格基準 | 承認要件 |
|---|---|---|
|  |  |  |

---

## リリースパイプライン

```mermaid
flowchart LR
    []
     --> 
```

---

## プラットフォーム操作保証シナリオ

### 背景

{{guaranteeScenarios.background}}

### {{guaranteeScenarios.scenarios[1].name}}

| 分類 | 観点 |
|---|---|
| {{guaranteeScenarios.scenarios[1].category}} | {{guaranteeScenarios.scenarios[1].viewpoint}} |

```gherkin
{{guaranteeScenarios.scenarios[1].gherkin}}
```
