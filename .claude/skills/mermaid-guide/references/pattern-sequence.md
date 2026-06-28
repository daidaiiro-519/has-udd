# シーケンス図（sequenceDiagram）

## 概要

アクター（登場人物・システム）間のメッセージのやり取りを時系列で表現する図。「誰が誰に何を送るか」の順序を示す。

## 使いどころ

- ユースケースの処理フロー（どのコンポーネントが何を呼ぶか）
- ドメインイベントの発生から通知までの流れ
- APIコール・サービス間通信の順序
- コマンドとイベントの連鎖

## 使わないケース

- 静的な構造・関係 → `flowchart` or `classDiagram`
- 状態の変化 → `stateDiagram-v2`

---

## 基本テンプレート

```mermaid
sequenceDiagram
    participant A as アクターA
    participant B as アクターB
    A->>B: メッセージ
    B-->>A: レスポンス
```

---

## メッセージの種類

| 記法 | 線種 | 矢印 | 用途 |
|---|---|---|---|
| `->>` | 実線 | 矢印あり | 同期呼び出し・リクエスト |
| `-->>` | 点線 | 矢印あり | 非同期レスポンス・戻り値 |
| `->` | 実線 | 矢印なし | メッセージ送信 |
| `-->` | 点線 | 矢印なし | 非同期通知 |
| `-x` | 実線 | × | 失敗・エラー |

---

## 実例

### 例1: コマンドとイベントの流れ

```mermaid
sequenceDiagram
    participant U as 利用者
    participant App as アプリケーション
    participant Domain as ドメイン
    participant DB as リポジトリ

    U->>App: 注文確定コマンド
    App->>Domain: Order.confirm()
    Domain-->>App: OrderConfirmedイベント
    App->>DB: 注文を保存
    DB-->>App: 保存完了
    App-->>U: 確定完了
```

### 例2: 条件分岐（alt）

```mermaid
sequenceDiagram
    participant C as クライアント
    participant S as サービス

    C->>S: リクエスト
    alt 在庫あり
        S-->>C: 成功レスポンス
    else 在庫なし
        S-->>C: エラー（在庫不足）
    end
```

### 例3: ループと注釈（Note）

```mermaid
sequenceDiagram
    participant A as 処理A
    participant B as 処理B

    Note over A,B: 同期処理開始
    loop 3件分
        A->>B: アイテム処理
        B-->>A: 完了
    end
    Note right of B: すべて処理完了
```

---

## 主要オプション

```
autonumber          # メッセージに連番を付ける
activate A          # アクターをアクティブ表示（処理中）
deactivate A        # アクティブ解除
```
