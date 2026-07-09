# ブレスト: has-uddのrole agent再定義とBacklog.md利活用による中核価値の再検討

**作成日:** 2026-07-10
**経緯:** waffleリポジトリ側でBacklog.md（Markdown-native task manager for AI agents,
https://github.com/MrLesk/Backlog.md）を味見導入したところ、has-uddが元々core機能として
想定していた「タスク管理（PBI/kanban/状態遷移の記録）」がOSS利活用でほぼ済んでしまうことが
見えてきた。これを起点に、has-uddのrole agent構成と中核価値を再検討する。

---

## 1. role agentをprocess軸で切り直す（合意）

**結論（合意済み）:** has-uddのrole agentは、当初想定していた「バックエンドエンジニア／
フロントエンドエンジニア／QAエンジニア」という**機能領域（技術ロール）**軸ではなく、
**PO（プロダクトオーナー）／スクラムマスター**という**プロセス軸**で構成し直す。

- `has-udd` = **H**arness **A**gentic **S**crum（HAS）+ **U**secase-**D**riven-**D**evelopment（UDD）
  という頭字語の通り、agent system側の第一関心事は本来「プロセス」であるべきだった
- 機能領域ロール（バックエンド/フロントエンド/QA）が担うはずだった「レイヤー配置・依存方向・
  技術選定」の判断は、既にWaffle側の`tech-lead-advisor`（レイヤー境界）・`ddd-advisor`
  （ドメイン整合性）・`ux-advisor`（プレゼンテーション層内部）・`platform-advisor`
  （デプロイ単位/外部境界）がadvisorとしてカバーしている。これらは「相談を受けて助言する」
  形式のスキルであり、「role agentとして常駐しタスクを実行する」形式とは別軸
- したがって、has-udd側に技術ロールのrole agentを重複して置く必要性が薄れた。has-udd固有の
  role agentは「プロセスの意思決定・進行管理」を担うPO/SMに絞ってよい

---

## 2. Backlog.md利活用によるタスク管理のコモディティ化

**論点:** Backlog.mdは以下を既に提供する（waffleリポジトリでの味見導入で確認済み）:
- タスクのMarkdownネイティブ管理（`backlog/tasks/`配下、YAMLメタデータ付き）
- 受け入れ基準（AC）・Definition of Done（DoD）のタスクごとの管理
- ターミナルKanban board（`backlog board`）+ Web UI
- AIエージェント向けの運用ガイド（`backlog instructions overview/task-creation/
  task-execution/task-finalization`）— 検索優先・スコープ判断・plan記録・進捗ノート・
  AC/DoD消込・final summary・「勝手にフォローアップを作らない」といった規律を、CLIコマンドの
  実行を介して強制する設計

このガイドの規律（直接編集禁止でCLI経由に統一する・スコープ外は勝手に広げない・
フォローアップはユーザー承認を得てから）は、CLAUDE.mdの運用ルール表が持つ思想
（document.json操作は必ずCLI/MCP経由、等）と同型。車輪の再発明をする価値は薄い。

**懸念:** タスク管理（記録・可視化）をhas-uddの中核機能として想定していた場合、
それがまるごとOSS利活用で代替可能になると、has-uddの存在意義が薄まる。
別の箇所で価値を見出す必要がある。

---

## 3. has-uddの残存価値（未決着・要検討）

タスクの**記録**がコモディティ化されても、以下はBacklog.mdのようなタスク管理OSSが
持たない領域であり、has-uddの中核価値になりうる（AI初期見解・ユーザー未検討）:

| 候補 | 内容 |
|---|---|
| ハーネス/ループそのもの | Backlog.mdは記録・可視化止まりで、実行系を持たない。PO/SMエージェントが実際にスプリントを回す・ceremonyを執行する・進捗の停滞を検知して次の一手を決める、という**駆動**部分は空白地帯のまま。「HAS」の"Harness"はまさにここ |
| Waffleとの橋渡し（spec⇄backlogのリンク） | BacklogのPBIが実際に仕様化され（Waffleのuc-*.json等）検証されているかを機械的に突き合わせる接着層。既製のタスク管理OSSにはない発想で、`sd-reconciliation`のドリフト検知思想（`check-spec-integrity`等）と親和性が高い |
| Scrumプロセス上の判断力の実行時適用 | 「このPBIは分割すべきか」「このretroで何を変えるべきか」といった判断は記録ツールの仕事ではなく、判断主体（PO/SMエージェント）の仕事。`scrum-knowledge`スキルが持つ知識を実際の意思決定に適用する層 |

**次のアクション:** 上記3候補のどれ（または組み合わせ）をhas-uddの中核とするか、
ユーザーとの合意形成が必要。合意でき次第、本ブレストに追記するか、
`brainstorm-has-udd-design.md`側の該当箇所を更新する。

---

## 4. 関連ブレストへの接続

- `brainstorm-waffle-next-evolution.md`（waffleリポジトリ側の引き継ぎ文書
  `waffle/docs/handoff-has-udd-brainstorms.md`参照）論点5（SDD標準ツール群との競合）に、
  Backlog.mdも比較対象として追加する
- `brainstorm-has-udd-concept.md`・`brainstorm-has-udd-design.md`のrole agent記述箇所は、
  本ブレストの合意（PO/SM軸への切り替え）を反映して更新が必要
