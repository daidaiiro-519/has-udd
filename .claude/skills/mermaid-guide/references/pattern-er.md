# ER図（erDiagram）

## 概要

エンティティ間の関係と多重度を表現する図。データ構造・ドメインモデルの関係を「何対何」の視点で示す。

## 使いどころ

- ドメインモデルのエンティティ間の関係・多重度
- データベーススキーマの設計・確認
- 集約をまたぐ参照関係の整理

## 使わないケース

- クラスのメソッド・継承関係 → `classDiagram`
- 処理の順序 → `sequenceDiagram`

---

## 基本テンプレート

```mermaid
erDiagram
    ENTITY1 ||--o{ ENTITY2 : "関係名"
    ENTITY1 {
        type field
    }
```

---

## 多重度の記法

| 左側 | 右側 | 意味 |
|---|---|---|
| `\|o` | `o\|` | ゼロまたは1 |
| `\|\|` | `\|\|` | ちょうど1 |
| `}o` | `o{` | ゼロ以上（0...*） |
| `}\|` | `\|{` | 1以上（1...*） |

組み合わせ例：
- `\|\|--o{` : 1対多（1つのAに0以上のB）
- `\|\|--\|\|` : 1対1
- `}o--o{` : 多対多

---

## 実例

### 例1: 受注ドメインのエンティティ関係

```mermaid
erDiagram
    ORDER ||--o{ ORDER_ITEM : contains
    ORDER ||--|| CUSTOMER : "placed by"
    ORDER_ITEM ||--|| PRODUCT : references

    ORDER {
        string id PK
        string customerId FK
        string status
        date orderedAt
    }

    ORDER_ITEM {
        string id PK
        string orderId FK
        string productId FK
        int quantity
        int price
    }

    CUSTOMER {
        string id PK
        string name
        string email
    }

    PRODUCT {
        string id PK
        string name
        int price
    }
```

### 例2: 集約をまたぐ参照（IDのみ）

```mermaid
erDiagram
    ORDER ||--o{ ORDER_ITEM : contains
    ORDER {
        string id PK
        string customerId "CustomerのID参照（集約外）"
    }
    ORDER_ITEM {
        string id PK
        string orderId FK
        string productId "ProductのID参照（集約外）"
        int quantity
    }
```
