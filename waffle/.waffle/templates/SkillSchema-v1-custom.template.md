# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{purpose.text}}` | このSkillが「何をするか」だけでなく「いつ使うべきか」を1〜2文で記述してください。動詞で始め、判断のトリガーとなる状況・キーワードを含めること（例:「〜を実装・修正する際に必ず使う」「ユーザーが〜と言ったときに使う」）。自動委譲/自動参照の判断根拠として使われる最重要フィールドです。 |
| `{{inputExpectation.items[1].aspect}}` | 受け取ると想定する情報の種類（例: 対象ブランチ、対象ファイル範囲）。 |
| `{{inputExpectation.items[1].interpretation}}` | その情報が明示されなかった場合にどう解釈するか（既定値・確認を挟む等）。 |
| `{{role.items[1]}}` | このSkillが担う責務を列挙してください。各項目は「〜する」「〜を提供する」の形で記述してください。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{processingTarget.target}}` | このSkillが受け取る処理対象を自然言語で記述してください。何を・どのような状態のものを受け取るかを明示してください。 |
| `{{processingTarget.artifact}}` | このSkillが生み出す成果物を自然言語で記述してください。何を・どのような形式で出力するかを明示してください。 |
| `{{steps.items[1].stepId}}` | step-1, step-2 のように連番で付けてください。 |
| `{{steps.items[1].title}}` | このStepで行うことを動詞で始めて簡潔に記述してください。 |
| `{{steps.items[1].summary}}` | この Step でSubAgentが行うドメイン操作の要点を1〜2文で簡潔に。インフラ語彙（document.json・MCP・ファイルパス・Python API 等）は書かない。 |
| `{{steps.items[1].bullets[1]}}` | 選択肢・一覧など箇条書きにすべき項目があれば1行ずつ列挙（任意）。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{steps.items[1].children[1].stepId}}` | step-1-1, step-1-2 のように親 stepId を引き継いで連番で付けてください。 |
| `{{steps.items[1].children[1].title}}` | このSubStepで行うことを動詞で始めて簡潔に記述してください。 |
| `{{steps.items[1].children[1].bullets[1]}}` | 箇条書きにすべき項目があれば1行ずつ列挙（任意）。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{steps.items[1].children[1].summary}}` | この SubStep でSubAgentが行うドメイン操作の要点を1〜2文で簡潔に。インフラ語彙は書かない。 |
| `{{outputFormat.text}}` | 実行結果・報告の形式を指定してください（例:「チェック項目ごとに✓/✗と理由を箇条書きで報告する」）。無ければこのブロック自体を省略してください。 |
| `{{guardrails.items[1]}}` | このSkillを実行する際に必ず守るべき制約・禁止事項・前提条件を列挙してください。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{references.items[1].path}}` | 参照先のファイルパスまたはスキーマ参照を記述してください。例: .waffle/schemas/SkillSchema/v1.json |
| `{{references.items[1].description}}` | このリソースが何を提供するかを1文で記述してください。 |
| `{{invocationMode.manualOnly}}` | ユーザーが `/skill-name` で明示的に呼び出す手順型Skill（自動発火させたくない）なら true。ドメイン知識として自動的に参照されてよいリファレンス型Skillなら false。 |

---

# {{title.title}}

---

## 目的

{{purpose.text}}

---

## 役割

- {{role.items[1]}}

---

## 処理対象と成果物

### 処理対象

{{processingTarget.target}}

### 成果物

{{processingTarget.artifact}}

---

## 入力の想定

| 受け取る情報 | 解釈・既定値 |
|---|---|
| {{inputExpectation.items[1].aspect}} | {{inputExpectation.items[1].interpretation}} |

---

## 実行手順

### Step 1: {{steps.items[1].title}}

{{steps.items[1].summary}}

- {{steps.items[1].bullets[1]}}

#### {{steps.items[1].children[1].title}}

{{steps.items[1].children[1].summary}}

- {{steps.items[1].children[1].bullets[1]}}

---

## 出力形式

{{outputFormat.text}}

---

## ガードレール

- {{guardrails.items[1]}}

---

## 参照

- `{{references.items[1].path}}`: {{references.items[1].description}}
