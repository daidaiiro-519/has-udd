# アーキテクチャ図（architecture-beta）

> ⚠️ **beta構文**: 表示環境によってレンダリングされない場合があります。使用前に確認してください。

## 概要

クラウドサービス・サーバー・データベース等のインフラコンポーネントをアイコン付きで表現する図。サービス間の通信・依存関係を可視化する。

## 使いどころ

- クラウドインフラ構成（AWS・GCP・Azure等）
- マイクロサービス間の通信
- システム全体のアーキテクチャ概要

## 使わないケース

- ソフトウェアの論理的な構造 → `classDiagram`
- 処理フロー → `sequenceDiagram`
- シンプルなコンポーネント配置 → `block-beta` or `flowchart`

---

## 基本テンプレート

```
architecture-beta
    group グループ名(アイコン)[ラベル]
        service サービスID(アイコン)[ラベル]
    end
    サービスID:右辺 --> 左辺:別サービスID
```

辺の方向: `L`（左）, `R`（右）, `T`（上）, `B`（下）

---

## 主要アイコン

| アイコン名 | 用途 |
|---|---|
| `cloud` | クラウド全般 |
| `server` | サーバー |
| `database` | データベース |
| `disk` | ストレージ |
| `internet` | インターネット |
| `user` | ユーザー |

---

## 実例

### 例1: マイクロサービスのインフラ構成

```mermaid
architecture-beta
    group api(cloud)[バックエンド]
        service gw(server)[APIゲートウェイ]
        service order(server)[注文サービス]
        service inventory(server)[在庫サービス]
    end

    group data(cloud)[データ層]
        service orderdb(database)[注文DB]
        service inventorydb(database)[在庫DB]
    end

    service client(user)[クライアント]

    client:R --> L:gw
    gw:R --> L:order
    gw:R --> L:inventory
    order:B --> T:orderdb
    inventory:B --> T:inventorydb
```

### 例2: シンプルな3層構成

```mermaid
architecture-beta
    service ui(server)[フロントエンド]
    service api(server)[APIサーバー]
    service db(database)[データベース]

    ui:R --> L:api
    api:R --> L:db
```
