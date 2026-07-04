# tech-stack-python-hexagonal

---

## スタック概要

- **ティア**: backend
- **スタック名**: python-hexagonal

---

## ランタイム

- **言語**: Python 3.12+
- **実行ターゲット**: CLI / ローカルプロセス
- **並行モデル**: 同期主体（MCP 境界のみ async）

---

## フレームワーク

なし（Web フレームワークは使用しない。外部公開は「公開インターフェース」を参照）

---

## 公開インターフェース

| 様式 | 実装 |
|---|---|
| CLI | typer |
| MCP | fastmcp |

---

## ミドルウェア

なし（document.json をファイルとして保存。DB・ブローカー・AP サーバーを持たない）

---

## ライブラリ

| 分類 | 用途 | 実装 | バージョン |
|---|---|---|---|
| validation | schema-validation | jsonschema | ^4 |
| observability | logging | 標準 logging |  |

---

## 開発ツール

- **パッケージ管理**: uv（.venv / uv.lock 固定）
- **lint / format**: ruff（任意）

---

## 依存方針

| 種別 | 方針 |
|---|---|
| 必須 | 依存追加は「既存の用途で代替不可か」を確認してから |
| 禁止 | 一覧に無いライブラリを反射的に import する |
| 推奨 | バージョンは範囲指定し uv.lock で固定する |
