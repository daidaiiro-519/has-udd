# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{identity.tier}}` | このスタックが担うティア。 |
| `{{identity.stackName}}` | スタック名（kebab-case）。documentId の {stack} 部分と一致させる。 |
| `{{runtime.language}}` | 言語とバージョン。例: Python 3.12+ |
| `{{runtime.target}}` | 実行ターゲット。例: CLI / ローカルプロセス、ブラウザ、コンテナ |
| `{{runtime.concurrency}}` | 並行モデル。例: 同期主体（境界のみ async） |
| `{{framework.name}}` | 看板フレームワーク名（1つ）。無ければ空文字。例: FastAPI, React |
| `{{framework.note}}` | name が空のとき、無い理由を1文で。name があれば空文字。 |
| `{{framework.rationale}}` | name が空でなければ、他の候補（習熟度・既存資産・技術要件等）ではなくこれを選んだ理由を1文で（ADR）。name が空なら空文字。 |
| `{{interface.items[1].style}}` | 様式。例: CLI, MCP, REST |
| `{{interface.items[1].implementation}}` | 実装ライブラリ。例: typer, fastmcp |
| `{{interface.items[1].rationale}}` | 他の候補（習熟度・既存資産・技術要件等）ではなくこの実装を選んだ理由を1文で（ADR）。 |
| `{{middleware.items[1].role}}` | ミドルウェアの役割分類。 |
| `{{middleware.items[1].product}}` | 製品名。例: PostgreSQL, Redis |
| `{{middleware.items[1].access}}` | アクセスするクライアントライブラリ。例: SQLAlchemy |
| `{{middleware.items[1].rationale}}` | 他の候補（別のDB製品等・実行環境の制約含む）ではなくこれを選んだ理由を1文で（ADR）。 |
| `{{middleware.note}}` | items が空のとき、無い理由を1文で。items があれば空文字。 |
| `{{libraries.items[1].category}}` | 能力の分類（AWS カテゴリ相当）。例: validation, observability, security |
| `{{libraries.items[1].capability}}` | 用途名。例: schema-validation, logging |
| `{{libraries.items[1].implementation}}` | 実装ライブラリ。例: jsonschema |
| `{{libraries.items[1].version}}` | バージョン。固定不要なら空。 |
| `{{libraries.items[1].rationale}}` | 他の候補ではなくこの実装を選んだ理由を1文で（ADR）。 |
| `{{tooling.packageManager}}` | パッケージ管理ツール。例: uv（.venv / uv.lock 固定） |
| `{{tooling.lintFormat}}` | lint/format ツール。任意なら空。 |
| `{{tooling.rationale}}` | 他の候補ではなくこのパッケージ管理ツールを選んだ理由を1文で（ADR）。 |
| `{{policy.items[1].rule}}` | 方針の内容。 |

---

# {{title.title}}

---

## スタック概要

- **対象領域（ティア: backend=サーバー側 / frontend=画面側 / platform=基盤側）**: {{identity.tier}}
- **スタック名**: {{identity.stackName}}

---

## ランタイム

- **言語**: {{runtime.language}}
- **実行ターゲット**: {{runtime.target}}
- **並行モデル**: {{runtime.concurrency}}

---

## フレームワーク

{{framework.note}}

- **フレームワーク**: {{framework.name}}
- **選定理由**: {{framework.rationale}}

---

## 公開インターフェース

### {{interface.items[1].style}}

- **実装**: {{interface.items[1].implementation}}

#### 選定理由

{{interface.items[1].rationale}}

---

## ミドルウェア

{{middleware.note}}

### {{middleware.items[1].product}}

- **役割**: {{middleware.items[1].role}}
- **アクセス手段**: {{middleware.items[1].access}}

#### 選定理由

{{middleware.items[1].rationale}}

---

## ライブラリ

### {{libraries.items[1].implementation}}

- **分類**: {{libraries.items[1].category}}
- **用途**: {{libraries.items[1].capability}}
- **バージョン**: {{libraries.items[1].version}}

#### 選定理由

{{libraries.items[1].rationale}}

---

## 開発ツール

- **パッケージ管理**: {{tooling.packageManager}}
- **lint / format**: {{tooling.lintFormat}}
- **選定理由**: {{tooling.rationale}}

---

## 依存方針

| 種別 | 方針 |
|---|---|
| 必須 | {{policy.items[1].rule}} |
