# uc-scaffold-document

---

## 概要

schema から Document の骨格を機械生成し（create）、AI が生成した値を宣言済みフィールドにのみ機械的に書き込む（fill）。AI は構造を触らない。

---

## 主アクターと意図

- **主アクター**: Orchestrator（HarnessAgent）
- **意図**: 新しい Document を schema 通りに起こし、値だけを安全に埋める

---

## 事前条件

- 生成対象の schema と documentId が与えられている
- 分岐のある schema では discriminator が与えられている

---

## 基本フロー

```mermaid
sequenceDiagram
    Orchestrator->>Document: create(schemaRef, documentId)
    Document-->>Orchestrator: 骨格と記入テンプレートを返す
    Orchestrator->>Orchestrator: 値だけ生成（構造は触らない）
    Orchestrator->>Document: fill(値)
    Note over Document: DocumentCreated
```

---

## 事後条件

- Document が schema の初期 status（enum 先頭）で生成される
- 宣言済みの値フィールドにのみ値が書き込まれる
- 構造（const / discriminator）は AI に変更されない

---

## 受け入れ基準

- When schemaRef と documentId が与えられたとき、engine は schema に適合する骨格を生成する shall（status=schema の enum 先頭）。
- When fill で値が与えられたとき、engine は宣言済み値フィールドにのみ書き込む shall。
- If 構造を変える値や const / discriminator が与えられたとき、engine は拒否し skipped に記録する shall。
- If 分岐のある schema で discriminator が無いとき、engine は MISSING_DISCRIMINATOR を返し候補を案内する shall。

---

## エラー

| コード | 条件 |
|---|---|
| `MISSING_DISCRIMINATOR` | 分岐のある schema で discriminator が未指定（候補 enum を案内） |
| `INVALID_SCHEMA_REF` | 未知の schemaRef |
| `SKIPPED` | 未知 path / const / discriminator への書き込み（書き込まず skipped に記録） |

---

## テストシナリオ

### 生成した骨格は自分の schema で valid

| 分類 | 観点 |
|---|---|
| 正常系 | 骨格生成：生成骨格は schema 適合・status は初期値 |

```gherkin
Scenario: 生成した骨格は自分の schema で valid
  Given engine 種別の Document（discriminator 指定済み）
  When create する
  Then 骨格は schema に適合し、status は schema の初期値である
```

### 構造を変える値は拒否される

| 分類 | 観点 |
|---|---|
| 異常系 | 構造保護：const フィールドへの書き込みは skipped |

```gherkin
Scenario: 構造を変える値は拒否される
  Given 作成済みの Document
  When const フィールドへ値を書き込もうとする
  Then 書き込まれず skipped に記録される
```
