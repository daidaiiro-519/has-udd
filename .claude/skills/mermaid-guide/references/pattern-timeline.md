# タイムライン（timeline）

## 概要

時系列の出来事を年表形式で表現する図。歴史・プロセスの進化・ロードマップを表すのに適している。

## 使いどころ

- 技術・業界の歴史的な変遷
- プロジェクトのマイルストーン・ロードマップ
- 概念の発展史

## 使わないケース

- タスクの期間・並行作業 → `gantt`
- 処理の順序 → `sequenceDiagram`

---

## 基本テンプレート

```mermaid
timeline
    title タイムラインのタイトル
    section 期間1
        出来事1 : 説明
        出来事2 : 説明
    section 期間2
        出来事3 : 説明
```

---

## 実例

### 例1: DDDの歴史

```mermaid
timeline
    title ドメイン駆動設計の歴史
    section 2000年代
        2003 : Eric Evansが「Domain-Driven Design」を出版
        2004 : DDDの概念が広まり始める
    section 2010年代
        2013 : マイクロサービスアーキテクチャとDDDの融合
        2015 : イベントソーシング・CQRSとの組み合わせが普及
        2016 : イベントストーミングの手法が注目される
    section 2020年代
        2021 : Vaughn Vernon「ドメイン駆動設計をはじめよう」出版
```

### 例2: プロジェクトロードマップ

```mermaid
timeline
    title 開発ロードマップ
    section Phase 1
        4月 : ドメイン分析・境界定義
        5月 : コアドメイン実装開始
    section Phase 2
        6月 : 統合テスト
        7月 : リリース
```
