# agg-schema

---

## 概要

Document の型定義（Schema）の不変条件とバージョン進化を表す集約。Document が機械生成・検証・描画できることを保証し、版の移行を司る。

---

## 集約ルート

- **集約ルート**: Schema

### 外部参照（ID）

- Document

---

## エンティティ

### Schema（集約ルート）

Document の型定義（構造・描画・記入/読取指示）の一貫性単位

| 属性 | 型 |
|---|---|
| **schemaId**（識別子） | SchemaId |
| version | Version |
| status | SchemaStatus |
| blocks | 構造定義（各ブロックの properties・x-render・x-prompt） |

---

## 値オブジェクト

| 値オブジェクト | 表すもの | 振る舞い・制約 |
|---|---|---|
| SchemaId | スキーマ名（例: SpecSchema） | 不変。値が等しければ等価。 |
| Version | 単調増加する版識別子（例: v1, v2） | 不変。値が等しければ等価。上げる方向にのみ進む。 |
| SchemaRef | スキーマ名＋版の組（Document が指す参照） | 不変。name と version が共に等しければ等価。 |
| SchemaStatus | 版のライフサイクル状態 | enum: PUBLISHED/DEPRECATED。値が等しければ等価。遷移は不変条件で守る。 |

---

## 不変条件

| ルール | 守り方 | 根拠 |
|---|---|---|
| 値フィールドは常に oneOf / anyOf を持たない | schema | scaffold が骨格を機械生成できるようにする |
| content の各ブロックは additionalProperties を常に閉じる（固定 properties のみ） | schema | 未知フィールドの混入を防ぎ構造を決定的にする |
| 再帰は常に有界である（無限ネストを許さない） | schema | 機械走査が停止することを保証する |
| 各ブロックの x-render は常に RenderMetaSchema の閉じた語彙にのみ従う | schema | 描画がロジックを持たず決定的であることを保つ |
| status の enum は常に遷移順に並び、先頭が初期状態である | schema | scaffold が初期状態を enum 先頭から一意に決められる |
| 公開済みの版は遡って構造を変えない（後方互換を壊さない） | guard | 既存 Document が破損しないよう版の進化を安全にする |

---

## ライフサイクル

```mermaid
stateDiagram-v2
    [*] --> PUBLISHED: publishVersion
    PUBLISHED --> DEPRECATED: deprecateVersion
```

### 遷移

| from | to | command | 条件 |
|---|---|---|---|
| [*] | PUBLISHED | publishVersion | scaffoldability 不変条件を満たす |
| PUBLISHED | DEPRECATED | deprecateVersion |  |

---

## コマンド

### publishVersion

新しい版の Schema を公開し、以後その版で Document を作れるようにする（scaffoldability 不変条件を守る）。

| 前提 | 後 | 発行イベント |
|---|---|---|
| （新規） | PUBLISHED | SchemaVersionPublished |

| 引数 | 意味 |
|---|---|
| version | 公開する版 |

### deprecateVersion

古い版を非推奨にし、新規 Document の作成を止める（既存は移行まで有効）。

| 前提 | 後 | 発行イベント |
|---|---|---|
| PUBLISHED | DEPRECATED |  |

| 引数 | 意味 |
|---|---|
| version | 非推奨にする版 |

### migrateDocuments

既存 Document を旧版から新版へ移行する（版を上げる方向にのみ・後方互換の不変条件を守る）。

| 前提 | 後 | 発行イベント |
|---|---|---|
| PUBLISHED | PUBLISHED | DocumentsMigrated |

| 引数 | 意味 |
|---|---|
| fromVersion | 移行元の版 |
| toVersion | 移行先の版 |

---

## ドメインイベント

### SchemaVersionPublished

#### 発行契機

publishVersion 成功時

#### ペイロード

| 項目 | 意味 |
|---|---|
| schemaRef | 公開された name + version |

### DocumentsMigrated

#### 発行契機

migrateDocuments 成功時

#### ペイロード

| 項目 | 意味 |
|---|---|
| fromVersion | 移行元の版 |
| toVersion | 移行先の版 |
| count | 移行された Document 数 |

---

## テストシナリオ

### 値フィールドに oneOf を持てない

| 分類 | 観点 |
|---|---|
| 異常系 | 不変条件: scaffoldability（値フィールドは oneOf/anyOf を持たない） |

```gherkin
Scenario: 値フィールドに oneOf を持てない
  Given 値フィールドに oneOf を含む Schema
  When scaffoldability を検証する
  Then scaffold 不能として拒否される
```

### 公開済みの版は後方互換を壊さない

| 分類 | 観点 |
|---|---|
| 異常系 | 不変条件: 公開済みの版は遡って構造を変えない |

```gherkin
Scenario: 公開済みの版は後方互換を壊さない
  Given PUBLISHED の Schema 版
  When 既存ブロックに必須フィールドを追加しようとする
  Then 後方互換を壊す変更として拒否される
```

### 移行は版を上げる方向にのみ行う

| 分類 | 観点 |
|---|---|
| 異常系 | 状態: migrateDocuments は版を上げる方向にのみ |

```gherkin
Scenario: 移行は版を上げる方向にのみ行う
  Given v1 と v2 の Schema
  When v2 から v1 へ移行しようとする
  Then 拒否される
```
