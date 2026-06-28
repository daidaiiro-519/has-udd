# Gitグラフ（gitGraph）

## 概要

Gitのブランチ・コミット・マージの履歴を視覚化する図。

## 使いどころ

- ブランチ戦略の説明（Git Flow, GitHub Flow等）
- リリースフロー・開発フローの設計図
- マージ・チェリーピックの手順説明

## 使わないケース

- プロセスの流れ全般 → `flowchart`
- スケジュール → `gantt`

---

## 基本テンプレート

```mermaid
gitGraph
    commit
    branch feature
    checkout feature
    commit
    commit
    checkout main
    merge feature
```

---

## 主要コマンド

| コマンド | 説明 |
|---|---|
| `commit` | 現在のブランチにコミット |
| `commit id: "メッセージ"` | IDまたはメッセージ付きコミット |
| `branch ブランチ名` | ブランチを作成 |
| `checkout ブランチ名` | ブランチを切り替え |
| `merge ブランチ名` | 現在のブランチにマージ |
| `cherry-pick id: "コミットID"` | 特定コミットのチェリーピック |

---

## 実例

### 例1: GitHub Flow

```mermaid
gitGraph
    commit id: "initial"
    branch feature/user-registration
    checkout feature/user-registration
    commit id: "add form"
    commit id: "add validation"
    checkout main
    merge feature/user-registration id: "Merge PR #1"
    branch feature/order-flow
    checkout feature/order-flow
    commit id: "add cart"
    commit id: "add checkout"
    checkout main
    merge feature/order-flow id: "Merge PR #2"
```

### 例2: Git Flow

```mermaid
gitGraph
    commit id: "v1.0"
    branch develop
    checkout develop
    commit
    branch feature/login
    checkout feature/login
    commit
    commit
    checkout develop
    merge feature/login
    branch release/1.1
    checkout release/1.1
    commit id: "bump version"
    checkout main
    merge release/1.1 id: "v1.1"
    checkout develop
    merge release/1.1
```
