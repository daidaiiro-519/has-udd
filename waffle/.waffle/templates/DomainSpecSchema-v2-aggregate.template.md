# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{summary.text}}` | この文脈/業務領域/集約/ユースケースが何かを1〜2文で（ドメイン語彙のみ・技術用語禁止）。 |
| `{{aggregateRoot.name}}` | ルートエンティティ名（ドメイン語彙）。例: Order |
| `{{aggregateRoot.externalRefs[1]}}` | ID で参照する他集約を列挙（オブジェクトは内部に持たない・図6-4）。例: Customer, Product。無ければ空。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{entities.items[1].name}}` | エンティティ名。例: Order / OrderLine |
| `{{entities.items[1].role}}` | 役割を1文で。 |
| `{{entities.items[1].isRoot}}` | 集約ルートなら true（1つだけ）。 |
| `{{entities.items[1].attributes[1].name}}` | 属性名。例: status / orderId |
| `{{entities.items[1].attributes[1].type}}` | 型（値オブジェクト名・他エンティティ・ID）。例: OrderStatus / OrderLine[] / customerId |
| `{{entities.items[1].attributes[1].isId}}` | この属性が識別子なら true（エンティティに1つ）。 |
| `{{valueObjects.items[1].name}}` | 値オブジェクト名。例: Money |
| `{{valueObjects.items[1].represents}}` | 何を表すか。例: 通貨つきの金額 |
| `{{valueObjects.items[1].behavior}}` | 振る舞い・制約・等価判定。例: 不変。加算は同一通貨のみ。負値不可。 |
| `{{invariants.items[1].rule}}` | 不変条件（「〜は常に〜」の形）。 |
| `{{invariants.items[1].enforcement}}` | 静的(値/構造)=schema が守る / 動的(状態遷移など)=guard が守る。 |
| `{{invariants.items[1].rationale}}` | そのルールが要る根拠を短く。 |
| `{{lifecycle.states[1]}}` | 取りうる状態を列挙。例: PLACED, PAID, SHIPPED。状態を持たない集約なら空。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{lifecycle.transitions[1].from}}` | 遷移元の状態。 |
| `{{lifecycle.transitions[1].to}}` | 遷移先の状態。 |
| `{{lifecycle.transitions[1].command}}` | 遷移を起こすコマンド。 |
| `{{lifecycle.transitions[1].condition}}` | 遷移の追加条件があれば。無ければ空。 |
| `{{commands.items[1].name}}` | コマンド名（命令形）。例: place / 確定する |
| `{{commands.items[1].description}}` | 何をするか・どの不変条件を守るかを1文で（実装手順は書かない）。 |
| `{{commands.items[1].requiresState}}` | 実行に必要な前提状態。集約インスタンスがまだ存在しない新規生成コマンドの場合は null（曖昧な文字列プレースホルダーは使わない）。 |
| `{{commands.items[1].postState}}` | 成功後の状態。 |
| `{{commands.items[1].params[1].name}}` | 引数名。 |
| `{{commands.items[1].params[1].meaning}}` | 意味を1文で。 |
| `{{commands.items[1].emits[1]}}` | 発行するドメインイベント名（過去形）。無ければ空。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{domainEvents.items[1].name}}` | イベント名（必ず過去形）。例: OrderPlaced |
| `{{domainEvents.items[1].raisedBy}}` | どのコマンド/契機で発行されるか。 |
| `{{domainEvents.items[1].payload[1].name}}` | 項目名。 |
| `{{domainEvents.items[1].payload[1].meaning}}` | 意味を1文で。 |
| `{{invariantScenarios.background}}` | 複数シナリオ共通の前提（Gherkin Background 相当）。無ければ空文字。 |
| `{{invariantScenarios.scenarios[1].name}}` | シナリオ名（概要）。 |
| `{{invariantScenarios.scenarios[1].category}}` | 分類: 正常系 / 異常系 / 境界値。 |
| `{{invariantScenarios.scenarios[1].viewpoint}}` | 観点: 側面（状態遷移/計算整合/事前条件 等）＋検証の狙い。 |
| `{{invariantScenarios.scenarios[1].gherkin}}` | このシナリオの Given/When/Then（Scenario: 1つ・実行可能なテストとして書き起こせる形）。ドメイン語彙で書き、実装詳細（クラス名/API/SQL/UI 操作）は書かない。 |
| `{{invariantScenarios.scenarios[1].covers}}` | 検証する不変条件への参照（任意・トレーサビリティ）。 |

---

# {{title.title}}

---

## 概要

{{summary.text}}

---

## 集約ルート

- **集約ルート**: {{aggregateRoot.name}}

### 外部参照（ID）

- {{aggregateRoot.externalRefs[1]}}

---

## エンティティ

### {{entities.items[1].name}}（集約ルート）

{{entities.items[1].role}}

| 属性 | 型 |
|---|---|
| **{{entities.items[1].attributes[1].name}}**（識別子） | {{entities.items[1].attributes[1].type}} |

---

## 値オブジェクト

| 値オブジェクト | 表すもの | 振る舞い・制約 |
|---|---|---|
| {{valueObjects.items[1].name}} | {{valueObjects.items[1].represents}} | {{valueObjects.items[1].behavior}} |

---

## 不変条件

| ルール | 守り方 | 根拠 |
|---|---|---|
| {{invariants.items[1].rule}} | {{invariants.items[1].enforcement}} | {{invariants.items[1].rationale}} |

---

## ライフサイクル

```mermaid
stateDiagram-v2
    {{lifecycle.transitions[1].from}} --> {{lifecycle.transitions[1].to}}: {{lifecycle.transitions[1].command}}
```

### 遷移

| from | to | command | 条件 |
|---|---|---|---|
| {{lifecycle.transitions[1].from}} | {{lifecycle.transitions[1].to}} | {{lifecycle.transitions[1].command}} | {{lifecycle.transitions[1].condition}} |

---

## コマンド

### {{commands.items[1].name}}

{{commands.items[1].description}}

| 前提 | 後 | 発行イベント |
|---|---|---|
| {{commands.items[1].requiresState}} | {{commands.items[1].postState}} | {{commands.items[1].emits[1]}} |

| 引数 | 意味 |
|---|---|
| {{commands.items[1].params[1].name}} | {{commands.items[1].params[1].meaning}} |

---

## ドメインイベント

### {{domainEvents.items[1].name}}

#### 発行契機

{{domainEvents.items[1].raisedBy}}

#### ペイロード

| 項目 | 意味 |
|---|---|
| {{domainEvents.items[1].payload[1].name}} | {{domainEvents.items[1].payload[1].meaning}} |

---

## 不変条件シナリオ

### 背景

{{invariantScenarios.background}}

### {{invariantScenarios.scenarios[1].name}}

| 分類 | 観点 |
|---|---|
| {{invariantScenarios.scenarios[1].category}} | {{invariantScenarios.scenarios[1].viewpoint}} |

```gherkin
{{invariantScenarios.scenarios[1].gherkin}}
```
