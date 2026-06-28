# Scrum Guide 2020 — Knowledge Reference

**出典:** The 2020 Scrum Guide（Ken Schwaber / Jeff Sutherland）  
**用語方針:** 英語の固有名詞はそのまま使用。日本語訳は補足として添える。

---

## Scrumの定義

> 「Scrumは、複雑な問題に対応する適応型のソリューションを通じて、人々・チーム・組織が価値を生み出すための軽量フレームワークです。」

Scrumは経験主義（empiricism）とリーン思考に基づく。複雑性へは反復的・漸進的（iterative and incremental）なアプローチで対応し、リスク管理と予測可能性を高める。

---

## Scrumの三本柱（Empiricism）

| 柱 | 定義 |
|---|---|
| **Transparency（透明性）** | 作業プロセスと成果物は、実施者と受領者の双方に可視化されていなければならない |
| **Inspection（検査）** | 成果物とゴールへの進捗を頻繁かつ熱心に検査し、問題の早期発見を行う |
| **Adaptation（適応）** | 検査の結果が許容範囲外の場合、プロセスや成果物を直ちに調整する |

---

## Scrumの5つの価値基準

**Commitment（コミットメント）/ Focus（集中）/ Openness（公開）/ Respect（尊重）/ Courage（勇気）**

この価値基準が三本柱を強化し、チームの信頼基盤を形成する。

---

## Scrum Team（スクラムチーム）

1チームの基本構成。理想は10人以下。

- **自己管理型（self-managing）**: 誰が何をいつどのようにするかを内部で決定する
- **クロスファンクショナル（cross-functional）**: 各Sprintで価値を生み出すために必要なスキルをすべて保有する

### Developers（開発者）

SprintごとにUsable Incrementを作成することにコミットする。

**責務:**
- Sprint Backlogを作成・管理する
- Definition of Doneに従い品質を担保する
- Sprint Goalに向けて毎日計画を適応させる

### Product Owner（PO）

製品価値の最大化と Product Backlog の管理に責任を持つ一人の人物。

**責務:**
- Product Goalを策定・周知する
- Product Backlog Itemを作成・精緻化する
- Product Backlog Itemの順序を決める
- Product Backlogを透明で可視的な状態に保つ

### Scrum Master（SM）

Scrum Guideの定義通りにScrumを確立する責任を持つ。チームとOrganizationへのサーバントリーダー。

**Scrum Teamへの責務:**
- 自己管理とクロスファンクショナリティをコーチする
- Definition of Doneを満たすIncrementの作成にフォーカスさせる
- 障害物（impediment）を取り除く
- ScumイベントがTime-box内に収まるよう保証する

---

## Scrum Events（スクラムイベント）

5つのイベントが正式なScrum Eventである。Sprintがすべてのイベントを内包する。

### Sprint

> 「Sprintはアイデアを価値に変えるScrumの心臓部です。」

- 固定期間（最大1か月）。前のSprintが終わると同時に次のSprintが始まる
- Sprint中はSprintGoalを危険にさらす変更を行わない
- SprintはProduct Ownerのみがキャンセルできる（Sprint Goalが時代遅れになった場合）

### Sprint Planning

Sprintの最初のイベント。Scrum Team全体で以下の3つのトピックを議論する。

| トピック | 問い | 成果物 |
|---|---|---|
| **Why** | このSprintはなぜ価値があるか | Sprint Goal |
| **What** | このSprintで何ができるか | 選択したProduct Backlog Items |
| **How** | 選んだ作業をどのように完成させるか | 実行計画（Sprint Backlog） |

**Time-box:** 1か月SprintならMax 8時間

### Daily Scrum

Developersが毎日同じ時間・場所で行う15分のイベント。

**目的:** Sprint Goalへの進捗を検査し、次の1日の作業計画を策定・調整する  
**Time-box:** 15分（毎日）

### Sprint Review

Sprint終了時にScrum TeamとステークホルダーがSprintの成果を検査するイベント。

**目的:** Sprint結果とProduct Goalへの進捗を評価し、次のProduct Backlogを適応させる  
**形式:** プレゼンテーションではなく作業セッション  
**Time-box:** 1か月SprintならMax 4時間

### Sprint Retrospective

Sprintの品質と効率を高める計画を策定するイベント。

**目的:** 人・相互作用・プロセス・ツール・Definition of Doneの観点で何がうまくいったか、何が問題だったかを検査する  
**Time-box:** 1か月SprintならMax 3時間

---

## Scrum Artifacts（スクラムの作成物）

各Artifactは対応するCommitment（コミットメント）を持つ。

```
Artifact                → Commitment
────────────────────────────────────
Product Backlog         → Product Goal
Sprint Backlog          → Sprint Goal
Increment               → Definition of Done
```

### Product Backlog

製品に必要な改善の創発的かつ優先順位付きリスト。Scrum Teamが行う作業の唯一の情報源。

- **Refinement（リファインメント）:** Product Backlog Itemをより小さく詳細なものに分解し、属性を追加していく継続的活動
- **Ready（レディ）:** 1 Sprint内で完了できると判断されたItemのステータス
- サイズ（見積もり）は Developersが決定する

**Commitment: Product Goal**

> 「Product Goalは、Scrum Teamが計画を立てるための目標となる製品の将来の状態を説明します。」

一度に持てるProduct Goalは1つ。達成または廃棄するまで次のGoalに移らない。

### Sprint Backlog

Sprint Goal（Why）＋ 選択したProduct Backlog Items（What）＋ 実行計画（How）で構成される。

- Developersが作成・所有・管理する
- 学習に応じてSprintを通じて継続的に更新される

**Commitment: Sprint Goal**

> 「Sprint Goalは、Sprintの唯一の目標です。」

- Sprint Planningで作成されSprint Backlogに追加される
- Sprint中に作業が期待と異なっても、Sprint Goalを変えずにスコープをProduct Ownerと交渉できる
- チームの一体感を高める

### Increment

Product Goalへの具体的な足がかり。各IncrementはDefinition of Doneを満たした積み重ねであること。

- 1 Sprint内で複数のIncrementが作成されることがある
- Sprint Reviewを待たずにリリース可能（Sprint ReviewはリリースGateとならない）

**Commitment: Definition of Done（DoD）**

> 「Definition of DoneはIncrementが品質基準を満たした状態の正式な説明です。」

- DoDを満たさないItemはProductBacklogに戻る
- Organizationの標準がある場合、チームはそれを最低基準として採用する

---

## Time-box一覧

| Event | Time-box（1か月Sprint時） |
|---|---|
| Sprint | Max 1か月 |
| Sprint Planning | Max 8時間 |
| Daily Scrum | 15分（毎日） |
| Sprint Review | Max 4時間 |
| Sprint Retrospective | Max 3時間 |
