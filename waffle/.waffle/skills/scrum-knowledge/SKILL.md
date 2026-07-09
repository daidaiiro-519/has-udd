---
name: scrum-knowledge
description: Scrum・アジャイルに関する質問・設計判断・知識注入を行うスキル。「Scrumの〜とは？」「PBIのリファインメントは誰が担うか？」「Sprint Goalとは？」などと言われたとき、またはhas-uddのJobAgent設計でScrumの正確な知識が必要なときに使う。
version: 1.0.0
---

## 目的

Scrum Guide 2020 と Agile Manifesto の内容をもとに、Scrumプロセス・役割・イベント・アーティファクトに関する正確な知識を提供する。has-udd の KnowledgeSkills として、Job Agentに注入するScrum知識の情報源となる。

## 役割

- Scrum Guideに定義された用語・役割・イベントを一次情報として提供する
- アジャイル12原則とScrumのマッピングを示す
- has-uddのPO Agent・SM Agent・Dev Agent・QA Agentが従うべきScrum上の責務を明確にする

## 用語統一方針

**同じ概念に対して複数の表現を使わない。** 以下の用語を唯一の正式表現とする。

| 正式用語 | 使用しない表現 |
|---|---|
| Product Owner（PO） | プロダクトオーナー、PO担当 |
| Scrum Master（SM） | スクラムマスター、SM担当 |
| Developers（Dev） | 開発者、エンジニア |
| Sprint | イテレーション、サイクル |
| Product Backlog | バックログ（Sprintと区別するため必ず修飾語付き） |
| Product Backlog Item（PBI） | チケット、タスク、要件 |
| Sprint Backlog | スプリントバックログ |
| Increment | 成果物（Artifactと区別するため） |
| Definition of Done（DoD） | 完成基準、Done基準 |
| Sprint Goal | スプリント目標 |
| Product Goal | プロダクト目標 |
| Sprint Planning | プランニング |
| Daily Scrum | デイリー、朝会 |
| Sprint Review | レビュー（Retrospectiveと区別するため必ず修飾語付き） |
| Sprint Retrospective | レトロ、振り返り |
| Refinement | バックロググルーミング |

## Knowledge ファイル一覧

| ファイル | 内容 |
|---|---|
| `references/knowledge/scrum-guide.md` | Scrum Guide 2020 全内容（定義・三本柱・価値基準・チーム構成・イベント・アーティファクト・コミットメント・Time-box） |
| `references/knowledge/agile-manifesto.md` | アジャイル宣言4つの価値 + 12の原則 + Scrumとの対応表 |

## has-uddでの活用方針

### PO Agent が参照すべき知識
- Product Backlog の管理責務（`scrum-guide.md` → Product Owner）
- PBI の作成・優先順位付け・Refinementの責務
- Product Goal の策定

### SM Agent が参照すべき知識
- Scrumイベントのファシリテーション責務
- 障害物（impediment）の除去
- チームの自己管理をコーチする責務

### Dev Agent が参照すべき知識
- Sprint Backlogの作成・管理
- Definition of Doneの遵守
- PBI → Usecase分解（Refinementへの参加、実装計画の策定）

### QA Agent が参照すべき知識
- Definition of Done の品質基準
- Incrementの検査とSprint Reviewへの参加

## ガードレール

- Scrum Guide の内容は改変せず引用する。独自解釈を加えない
- has-uddが担う責務と、現実のScrum Team が担う責務を混同しない
- has-uddは支援ツールであり、Scrumイベント自体を代替しない（Daily Scrum・Sprint Review等は人間が実施する）
