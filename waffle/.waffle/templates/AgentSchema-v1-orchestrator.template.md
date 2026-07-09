# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{scope.path}}` | このOrchestratorが管轄するディレクトリをリポジトリルートからの相対パスで指定してください。ルート自身なら空文字。例: "", "waffle/" |
| `{{scope.description}}` | このディレクトリ・プロダクトが何であるかを1〜2文で記述してください。 |
| `{{operatingRules.items[1].rule}}` | 何をすべきかの一文。「〜してください」より「〜する」という明確な指示文にする。 |
| `{{operatingRules.items[1].why}}` | なぜこのルールが必要か（根拠）。 |
| `{{operatingRules.items[1].howToApply}}` | いつ・どう適用するか（具体的な手順・コマンド・対象範囲）。 |
| `{{keyCommands.items[1].command}}` | 実行するコマンド。 |
| `{{keyCommands.items[1].purpose}}` | 何のためのコマンドかを1文で。 |
| `{{subOrchestratorRefs.items[1].scope}}` | 子Orchestratorが管轄するディレクトリ。例: "waffle/" |
| `{{subOrchestratorRefs.items[1].documentId}}` | 子OrchestratorのdocumentId。 |
| `{{subOrchestratorRefs.items[1].note}}` | そのscopeで作業する際に何を確認すべきかの短い説明。 |
| `{{skillFollowUp.items[1].afterSkill}}` | どのSkillを呼び出した後の話か（例: advisor全般／特定のSkill名）。 |
| `{{skillFollowUp.items[1].thenAction}}` | 次に何をするか（検証可能な具体的手順）。 |
| `{{skillFollowUp.items[1].outputFormat[1]}}` | フォローアップの結果をどんな段階構成で返すべきか、短い見出し語を順番に列挙してください（例: ["各意見", "統合見解", "合意事項", "次のアクション"]）。段落の説明文ではなく、一目で流れが追える短い語の並びにする。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{skillFollowUp.items[1].template}}` | outputFormatの各段階を実際に埋めるためのテンプレートファイルへの参照パス（プレースホルダー付きmarkdown）。 |
| `{{skillFollowUp.items[1].why}}` | なぜこのフォローアップが必要か。 |

---

# {{title.title}}

---

## 管轄範囲

- **管轄ディレクトリ**: {{scope.path}}
- **概要**: {{scope.description}}

---

## 運用ルール

| ルール | なぜ | 適用方法 |
|---|---|---|
| {{operatingRules.items[1].rule}} | {{operatingRules.items[1].why}} | {{operatingRules.items[1].howToApply}} |

---

## 主要コマンド

| コマンド | 用途 |
|---|---|
| {{keyCommands.items[1].command}} | {{keyCommands.items[1].purpose}} |

---

## 委譲先

### 以下のscope配下で作業する場合は、必ず対応するOrchestrator documentのcontentを先に読むこと

| 対象ディレクトリ | 参照Orchestrator | 備考 |
|---|---|---|
| {{subOrchestratorRefs.items[1].scope}} | {{subOrchestratorRefs.items[1].documentId}} | {{subOrchestratorRefs.items[1].note}} |

---

## Skillフォローアップ

### Skillを呼び出した後に必ず行うフォローアップ

| 呼び出した後 | 次に行うこと | 返却フォーマット | テンプレート | 理由 |
|---|---|---|---|---|
| {{skillFollowUp.items[1].afterSkill}} | {{skillFollowUp.items[1].thenAction}} | {{skillFollowUp.items[1].outputFormat[1]}} | `{{skillFollowUp.items[1].template}}` | {{skillFollowUp.items[1].why}} |
