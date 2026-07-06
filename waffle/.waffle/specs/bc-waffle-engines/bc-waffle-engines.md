# bc-waffle-engines

---

## 概要

waffle のコアエンジン群が属する文脈。document.json を唯一の正とし、AI が構造を推論せず engine が機械的に読み書き・生成・描画する（Harness 原則）。

---

## ユビキタス言語

| 用語 | 定義 |
|---|---|
| `Document` | schema で構造を定義された JSON の成果物単位（spec / skill / coding 等）。 |
| `Schema` | Document の構造・描画(x-render)・記入/読取指示(x-prompt)を定義する JSON Schema。 |
| `Harness原則` | AI はファイルを直接読まず値だけを埋め、engine が一切の構造アクセス・生成を担う原則。 |
| `意味単位` | ブロック / フィールド / 条件一致 / 全階層など、Document の意味のある取得単位。 |
| `prompt(読み方指針)` | 取得した value をどう解釈するかの指針。schema の x-prompt-query 由来。 |
| `骨格(scaffold)` | schema を機械走査して生成した、値が空の schema 準拠 Document の雛形。 |
| `UDD ループ` | Spec を正本とし、検証・描画・受け入れテストを通じて仕様と実装の整合を保つ開発サイクル。 |
| `不変条件` | 集約が常に満たす業務ルール。static は schema、dynamic は guard が守る。 |
| `reconcile` | ソースコードとSpecの対応関係を保つこと。AIに生ソースを読ませず engine が構造化抽出・規約適合検証を担う。 |

---

## 構成要素

| 種別 | メンバー |
|---|---|
| subdomain | sd-document-engine / sd-validation / sd-reconciliation |
| aggregate | agg-document / agg-schema |
| usecase | uc-scaffold-document / uc-query-document / uc-validate-document / uc-render-document / uc-scan-source-code / uc-lint-docstring |
