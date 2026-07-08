---
name: mermaid-guide
description: Mermaidの構文パターンを参照・選択したいとき、図の表現方法を決めたいとき、Mermaidの記法を確認したいとき、またはknowledgeSkillsで図を作成するときに使うスキル。「どのMermaid構文を使うか」「この図はどう書くか」「Mermaidで〜を表現したい」と言われたときに使う。
version: 1.0.0
---

## 目的

Mermaidの全構文パターンを一元管理し、コンテンツの種類に応じた適切な構文選択と一貫した記法を提供する。他のSkillsから参照されることを前提とした汎用リファレンス。

## 役割

- 図の内容・目的に応じた最適なMermaid構文を選定する
- 各構文の基本テンプレートと実例を提供する
- Skills間で統一されたMermaid記法を保証する

## 実行手順

### Step 1: 構文を選定する

`references/syntax-overview.md` を読み込み、表現したい内容に対応する構文を特定する。

### Step 2: パターンファイルを読み込む

特定した構文の `references/pattern-{構文名}.md` を読み込み、テンプレートと実例を参照する。

### Step 3: 図を生成する

パターンファイルのテンプレートに従って図を生成する。

## ガードレール

- `graph` 構文は使用しない。フローチャートは必ず `flowchart` を使う（旧構文のため）
- 判断フロー（YES/NO分岐）はMermaidではなくテキスト形式（コードブロック）で表現する
- beta構文（`sankey-beta`, `xychart-beta`, `architecture-beta`, `block-beta`）は表示環境に依存するため、使用時は注意を促す

## リファレンスファイル

- `references/syntax-overview.md` — 全構文一覧と選択基準
- `references/pattern-flowchart.md` — フローチャート
- `references/pattern-sequence.md` — シーケンス図
- `references/pattern-class.md` — クラス図
- `references/pattern-er.md` — ER図
- `references/pattern-state.md` — 状態遷移図
- `references/pattern-quadrant.md` — 象限チャート
- `references/pattern-mindmap.md` — マインドマップ
- `references/pattern-timeline.md` — タイムライン
- `references/pattern-journey.md` — ユーザージャーニー
- `references/pattern-gantt.md` — ガントチャート
- `references/pattern-pie.md` — 円グラフ
- `references/pattern-git.md` — Gitグラフ
- `references/pattern-requirement.md` — 要件図
- `references/pattern-sankey.md` — サンキー図（beta）
- `references/pattern-xychart.md` — XYチャート（beta）
- `references/pattern-block.md` — ブロック図（beta）
- `references/pattern-architecture.md` — アーキテクチャ図（beta）
