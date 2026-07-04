<!-- tech-stack codingKind の「レンダリング後イメージ」確定版（8ブロック）。has-udd の実スタックを正とする。内部ブロック名=英語(Identity/Runtime/Framework/Interface/Middleware/Capabilities/Tooling/Policy)・表示=日本語。style は architecture が SSOT なので tech-stack からは除外。Capabilities 表示=「ライブラリ」・分類列(category)/用途(capability)。Middleware は role 分類列。真のネストは無し(全てフラット)。★@stack アンカーは全廃済み(brainstorm-ai-era-detail-design D-1)＝ここはコードから参照される登録簿でなく、ドキュメントとして能力→実装を宣言するだけ。 -->

# python-hexagonal — 技術スタック

このスタックの技術選択と、コードが依存する非ドメイン能力を宣言する。

---

## スタック概要

| 項目 | 値 |
|---|---|
| ティア | backend |
| スタック名 | python-hexagonal |

---

## ランタイム

| 項目 | 選択 |
|---|---|
| 言語 | Python 3.12+ |
| 実行ターゲット | CLI / ローカルプロセス |
| 並行モデル | 同期主体（MCP 境界のみ async） |

---

## フレームワーク

なし（Web フレームワークは使用しない。外部公開は「公開インターフェース」を参照）。

---

## 公開インターフェース

| 様式 | 実装 |
|---|---|
| CLI | typer |
| MCP | fastmcp |

---

## ミドルウェア

なし（document.json をファイルとして保存。DB・ブローカー・AP サーバーを持たない）。

---

## ライブラリ

このスタックが担う非ドメイン能力（driven・横断）と、その実装ライブラリの対応表。

| 分類 | 用途 | 実装 | バージョン |
|---|---|---|---|
| validation | schema-validation | jsonschema | ^4 |
| observability | logging | 標準 `logging` | — |

---

## 開発ツール

| 項目 | 選択 |
|---|---|
| パッケージ管理 | uv（`.venv` / `uv.lock` 固定） |
| lint / format | ruff（任意） |

---

## 依存方針

| 種別 | 方針 |
|---|---|
| 必須 | 依存追加は「既存の用途で代替不可か」を確認してから |
| 禁止 | 一覧に無いライブラリを反射的に import する |
| 推奨 | バージョンは範囲指定し `uv.lock` で固定 |
