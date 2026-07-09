# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{summary.text}}` | この文脈/業務領域/集約/ユースケースが何かを1〜2文で（ドメイン語彙のみ・技術用語禁止）。 |
| `{{category.classification}}` | 中核=core / 補完=supporting / 一般=generic。競争優位を生むか・既製品があるかで判定。 |
| `{{category.rationale}}` | そのカテゴリーである根拠（差別化の有無）。 |
| `{{members.items[1]}}` | この業務領域に属する usecase の id を列挙。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{implementationGuidance.text}}` | 実装方針を1〜2文で。中核=ドメインモデルで厚く/一般=既製ライブラリを薄く包む/補完=薄いトランザクションスクリプト。 |
| `{{externalSolution.text}}` | 採用する既製ライブラリ/サービス名と、それが @stack のどの能力に当たるか。中核/補完なら不要。 |
| `{{domainServices.items[1].name}}` | 業務サービス名。 |
| `{{domainServices.items[1].responsibility}}` | 何を計算/判定するか・どの集約に跨るか・入力→出力（何を受け何を返すか）を1文で。実装手順は書かない。 |
| `{{domainServiceScenarios.background}}` | 複数シナリオ共通の前提。無ければ空文字。 |
| `{{domainServiceScenarios.scenarios[1].name}}` | シナリオ名（概要）。 |
| `{{domainServiceScenarios.scenarios[1].category}}` | 分類: 正常系 / 異常系 / 境界値。 |
| `{{domainServiceScenarios.scenarios[1].viewpoint}}` | 観点: 何を計算/判定するか＋検証の狙い。 |
| `{{domainServiceScenarios.scenarios[1].gherkin}}` | Given/When/Then。ドメイン語彙で書き、実装詳細は書かない。 |
| `{{domainServiceScenarios.scenarios[1].covers}}` | 対応するdomainServicesの項目への参照。 |

---

# {{title.title}}

---

## 概要

{{summary.text}}

---

## カテゴリー

- **カテゴリー**: {{category.classification}}
- **根拠**: {{category.rationale}}

---

## 所属ユースケース

- {{members.items[1]}}

---

## 実装ガイド

{{implementationGuidance.text}}

---

## 外部解決策

{{externalSolution.text}}

---

## ドメインサービス

| 業務サービス | 責務 |
|---|---|
| {{domainServices.items[1].name}} | {{domainServices.items[1].responsibility}} |

---

## 業務サービスシナリオ

### 背景

{{domainServiceScenarios.background}}

### {{domainServiceScenarios.scenarios[1].name}}

| 分類 | 観点 |
|---|---|
| {{domainServiceScenarios.scenarios[1].category}} | {{domainServiceScenarios.scenarios[1].viewpoint}} |

```gherkin
{{domainServiceScenarios.scenarios[1].gherkin}}
```
