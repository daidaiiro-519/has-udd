# 要件図（requirementDiagram）

## 概要

要件とシステム要素（コンポーネント・テスト等）の関係を表現する図。要件がどの要素によって満たされるかを追跡するために使う。

## 使いどころ

- 要件とコンポーネントのトレーサビリティ
- 要件の充足状況の可視化
- ドメイン要件とサービス/コンテキストの対応関係

## 使わないケース

- 処理フロー → `flowchart` or `sequenceDiagram`
- 設計レベルの構造 → `classDiagram`

---

## 基本テンプレート

```mermaid
requirementDiagram
    requirement 要件名 {
        id: 番号
        text: 要件の説明
        risk: high | medium | low
        verifymethod: test | analysis | inspection | demonstration
    }
    element 要素名 {
        type: 要素の種類
    }
    要素名 - satisfies -> 要件名
```

---

## 関係の種類

| 記法 | 意味 |
|---|---|
| `- satisfies ->` | 要素が要件を満たす |
| `- traces ->` | 要素が要件にトレースされる |
| `- refines ->` | 要件を詳細化する |
| `- copies ->` | 要件をコピーする |
| `- derives ->` | 要件から派生する |
| `- contains ->` | 要件を含む |

---

## 実例

### 例1: ドメイン要件とコンテキストの対応

```mermaid
requirementDiagram
    requirement 注文管理要件 {
        id: REQ-001
        text: 顧客が商品を注文し、在庫確認・決済・配送を一貫して処理できること
        risk: high
        verifymethod: test
    }

    requirement 在庫管理要件 {
        id: REQ-002
        text: リアルタイムで在庫数を管理し、過剰注文を防止できること
        risk: high
        verifymethod: test
    }

    element 注文コンテキスト {
        type: BoundedContext
    }

    element 在庫コンテキスト {
        type: BoundedContext
    }

    注文コンテキスト - satisfies -> 注文管理要件
    在庫コンテキスト - satisfies -> 在庫管理要件
    注文コンテキスト - traces -> 在庫管理要件
```
