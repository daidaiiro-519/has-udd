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

## コンプライアンス要件

| フレームワーク | 対象範囲 | 要件 |
|---|---|---|
|  |  |  |

---

## アクセス制御方針

| 主体 | 許可される操作 | 条件 |
|---|---|---|
|  |  |  |

---

## データ分類ごとの境界

| データ分類 | 境界 | 要件 |
|---|---|---|
|  |  |  |

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
