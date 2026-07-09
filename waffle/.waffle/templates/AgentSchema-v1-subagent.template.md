# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{description.text}}` | 具体的なタスク＋対象領域を行動動詞で始めて1文で記述してください（例:「コードのセキュリティ脆弱性をレビューする」であって「コードを手伝う」ではない）。単一の専門領域に絞ること（複数の異なる責務を1つのSubagentに詰め込まない）。積極的に自動委譲させたい場合は「〜の際は積極的に使う」のように明示してもよい。 |
| `{{inputExpectation.items[1].aspect}}` | 受け取ると想定する情報の種類（例: 対象ファイル/ディレクトリ、確認したい脆弱性の種類）。 |
| `{{inputExpectation.items[1].interpretation}}` | その情報が明示されなかった場合にどう解釈するか（既定値・確認を挟む等）。 |
| `{{toolAccess.tools[1]}}` | このSubagentに許可するツール名を列挙してください。読み取り専用の調査タスクならRead/Grep/Glob等に絞り、Write/Editのような変更系ツールは本当に必要な場合だけ含める。全ツールを継承する場合はこのブロック自体を省略してください。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{toolAccess.disallowedTools[1]}}` | このSubagentに禁止するツール名を列挙してください。無ければ空配列。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{runtimeConfig.model}}` | モデルを上書きする場合に指定してください（例: sonnet, opus）。既定を継承する場合はこのブロック自体を省略してください。 |
| `{{runtimeConfig.permissionMode}}` | 権限モードを指定してください（例: default, plan）。 |
| `{{skillPreloads.items[1]}}` | 起動時にプリロードするSkillのdocumentIdを列挙してください。無ければこのブロック自体を省略してください。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{skillPreloads.guidance}}` | 列挙した各Skillを、どのような場面で・どう使うべきかを記述してください（例:「tech-lead-advisorは技術選定の判断が必要な場面で参照する」）。プリロードはSkillの中身を注入するだけで使い方までは指定しないため、ここで明示すること。 |
| `{{persona.text}}` | このSubagentの役割・専門性を2〜3文で定義してください。例:「あなたはシニアセキュリティエンジニアです。」のように、一人称の人格として明示すること。 |
| `{{responsibilities.items[1]}}` | このSubagentが担当する具体的な観点・確認項目を列挙してください（例:「インジェクション脆弱性（SQL/XSS/コマンド）」「認証・認可の欠陥」）。抽象的な項目でなく、実際に確認できる粒度で書く。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{constraints.items[1]}}` | 品質基準・見落としやすいエッジケース・作業上の注意点を列挙してください。無ければこのブロック自体を省略してください。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{outputFormat.text}}` | 報告・出力の形式を指定してください（例:「具体的な行番号と修正案を示すこと」）。無ければこのブロック自体を省略してください。 |

---

# {{title.title}}

---

## 目的

{{description.text}}

---

## 役割定義

{{persona.text}}

---

## 担当領域

- {{responsibilities.items[1]}}

---

## 入力の想定

| 受け取る情報 | 解釈・既定値 |
|---|---|
| {{inputExpectation.items[1].aspect}} | {{inputExpectation.items[1].interpretation}} |

---

## プリロードSkill

- {{skillPreloads.items[1]}}

{{skillPreloads.guidance}}

---

## 品質基準・制約

- {{constraints.items[1]}}

---

## 出力形式

{{outputFormat.text}}
