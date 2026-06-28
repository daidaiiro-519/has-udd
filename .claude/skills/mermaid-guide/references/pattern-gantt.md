# ガントチャート（gantt）

## 概要

タスクの期間・依存関係・進捗をバーで表現するプロジェクト管理図。

## 使いどころ

- プロジェクトのスケジュール管理
- タスクの依存関係と並行作業の可視化
- フェーズ・マイルストーンの表示

## 使わないケース

- 時系列の出来事（期間が不要） → `timeline`
- 処理の順序 → `sequenceDiagram`

---

## 基本テンプレート

```mermaid
gantt
    title プロジェクト名
    dateFormat YYYY-MM-DD
    section フェーズ1
        タスク1 :t1, 2024-01-01, 2024-01-10
        タスク2 :t2, after t1, 5d
```

---

## タスクの状態

| 記法 | 意味 |
|---|---|
| `done` | 完了 |
| `active` | 進行中 |
| `crit` | クリティカルパス（赤表示） |
| なし | 未着手 |

---

## 実例

### 例1: DDDプロジェクトのスケジュール

```mermaid
gantt
    title DDDプロジェクト計画
    dateFormat YYYY-MM-DD

    section 戦略的設計
        ドメイン分析         :done, s1, 2024-01-01, 2024-01-14
        コンテキストマッピング :done, s2, after s1, 7d
        サブドメイン分類      :active, s3, after s2, 7d

    section 戦術的設計
        集約設計             :crit, t1, after s3, 14d
        リポジトリ設計       :t2, after t1, 7d

    section 実装
        コアドメイン実装     :crit, i1, after t1, 21d
        インフラ実装         :i2, after t2, 14d
        テスト               :i3, after i1, 7d
```

---

## 日付形式のオプション

```
dateFormat YYYY-MM-DD   # 絶対日付
after タスクID          # 相対（別タスクの後）
Nd                      # N日間（5d = 5日）
Nw                      # N週間
```
