# Mermaid 構文選択ガイド

## 構文選択早見表

| 表現したい内容 | 推奨構文 | 方向 | パターンファイル |
|---|---|---|---|
| 階層・包含関係（AはBを含む） | `flowchart` | `TD` | pattern-flowchart.md |
| プロセス・変換の流れ（A→B→C） | `flowchart` | `LR` | pattern-flowchart.md |
| 役割・関係図（誰が何とつながるか） | `flowchart` | `LR` | pattern-flowchart.md |
| アクター間のメッセージ順序 | `sequenceDiagram` | — | pattern-sequence.md |
| クラス・継承・コンポジション | `classDiagram` | — | pattern-class.md |
| エンティティ間の関係・多重度 | `erDiagram` | — | pattern-er.md |
| 状態と遷移条件 | `stateDiagram-v2` | — | pattern-state.md |
| 2軸の分類マトリクス | `quadrantChart` | — | pattern-quadrant.md |
| 階層的な概念の展開 | `mindmap` | — | pattern-mindmap.md |
| 時系列の出来事・歴史 | `timeline` | — | pattern-timeline.md |
| ユーザー体験の段階とスコア | `journey` | — | pattern-journey.md |
| タスクとスケジュール | `gantt` | — | pattern-gantt.md |
| 比率・割合 | `pie` | — | pattern-pie.md |
| Gitブランチ・コミット履歴 | `gitGraph` | — | pattern-git.md |
| 要件と要素の関係 | `requirementDiagram` | — | pattern-requirement.md |
| フローの量・割合（beta） | `sankey-beta` | — | pattern-sankey.md |
| 折れ線・棒グラフ（beta） | `xychart-beta` | — | pattern-xychart.md |
| ブロック構成図（beta） | `block-beta` | — | pattern-block.md |
| サービス・インフラ構成（beta） | `architecture-beta` | — | pattern-architecture.md |

## 使用ルール

### 禁止事項

- **`graph` 構文は使用しない** — `flowchart` に統一する（`graph` はMermaid旧構文）
- **判断フロー（YES/NO分岐）はMermaidで書かない** — テキスト形式（コードブロック）で表現する

### beta構文の注意

`sankey-beta`, `xychart-beta`, `block-beta`, `architecture-beta` はMermaidのベータ機能。表示環境によってレンダリングされない場合がある。使用時はユーザーに注意を促す。

## flowchartの方向選択

| 方向指定 | 使いどころ |
|---|---|
| `TD`（top-down） | 階層・包含・ツリー構造 |
| `LR`（left-right） | プロセス・変換・関係・フロー |
| `BT`（bottom-top） | 積み上げ・依存関係（下から上） |
| `RL`（right-left） | 特殊なケース（通常は不要） |
