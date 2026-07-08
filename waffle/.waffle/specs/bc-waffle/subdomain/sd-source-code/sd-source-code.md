# sd-source-code

---

## 概要

ソースコードから、AI に生ファイルを読ませず、docstring を DocstringSchema の kind の構造宣言に従って機械的に抽出したインデックスビューを提供する業務領域。sd-document-engine が Document(document.json) に対して適用する Harness 原則（AI に構造を推論させず engine が決定的に処理する）を、対象をソースコードに広げて適用したもの。

---

## カテゴリー

- **カテゴリー**: core
- **根拠**: docstring を kind ごとの構造宣言（summary/body/args/returns/raises/attributes）に機械的に分解するロジックは既製の静的解析ツールでは代替できない、waffle 独自の差別化。sd-document-engine と同じ差別化原理（AI 0 で構造アクセス）をソースコードに適用したもの。

---

## 所属ユースケース

- uc-scan-source-code

---

## 実装ガイド

中核ゆえドメインモデルで厚く実装する。docstring の kind ごとの構造化抽出は自前の決定的コードで書き、AI に生ソースを読ませない。
