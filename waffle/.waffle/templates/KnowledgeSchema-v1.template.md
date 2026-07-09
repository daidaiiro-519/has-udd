# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{overview.questions[1]}}` | この概念を知ることで答えられるようになる、利用者の具体的な問いを列挙してください。「〜べきか」「〜はどう判断するか」の形で。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{overview.text}}` | この概念が何であるかを1〜2文で要約してください。 |
| `{{principles.text}}` | この概念の定義・成り立ち・分類（あれば）を、出典の一般原則に基づいて記述してください。 |
| `{{classifications.items[1].name}}` | 分類名。 |
| `{{classifications.items[1].description}}` | その分類の特徴。 |
| `{{decisionCriteria.stages[1].id}}` | ノードの識別子。例: q1 |
| `{{decisionCriteria.stages[1].label}}` | 問いや帰結の短い文言。 |
| `{{decisionCriteria.transitions[1].from}}` | 遷移元ノードのid。 |
| `{{decisionCriteria.transitions[1].to}}` | 遷移先ノードのid。 |
| `{{decisionCriteria.transitions[1].label}}` | 遷移条件（回答）の短い文言。 |
| `{{examples.text}}` | 架空の題材を使い、この概念を具体的に当てはめた実例を記述してください。特定の実在企業名は避けてください。 |
| `{{antiPatterns.items[1].name}}` | アンチパターン名。 |
| `{{antiPatterns.items[1].problem}}` | 何が問題かの説明。 |
| `{{provenance.text}}` | 内容の出典（書籍・章等）と、要約/再構成であり直接引用でないことを明記してください。 |
| `{{relatedConcepts.items[1].conceptId}}` | 関連するKnowledge documentのdocumentId。 |
| `{{relatedConcepts.items[1].note}}` | どう関連するかの短い説明。 |

---

# {{title.title}}

---

## 概要

### この概念が答える判断

- {{overview.questions[1]}}

{{overview.text}}

---

## 原則

{{principles.text}}

---

## 分類

| 分類 | 特徴 |
|---|---|
| {{classifications.items[1].name}} | {{classifications.items[1].description}} |

---

## 判断基準

```mermaid
flowchart LR
    {{decisionCriteria.stages[1].id}}[{{decisionCriteria.stages[1].label}}]
    {{decisionCriteria.transitions[1].from}} -->|{{decisionCriteria.transitions[1].label}}| {{decisionCriteria.transitions[1].to}}
```

---

## 実例

{{examples.text}}

---

## アンチパターン

| アンチパターン | 問題点 |
|---|---|
| {{antiPatterns.items[1].name}} | {{antiPatterns.items[1].problem}} |

---

## 出典・根拠の透明性

{{provenance.text}}

---

## 関連概念

| 関連概念 | 関係 |
|---|---|
| {{relatedConcepts.items[1].conceptId}} | {{relatedConcepts.items[1].note}} |
