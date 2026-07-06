# sd-docstring-linting

---

## 概要

ソースコードの docstring が規約通りの構造か（必須セクションの有無・引数名と実シグネチャの整合）を、kind ごとに確立された既存 lint ツールへ判定を委ねて確認する業務領域。

---

## カテゴリー

- **カテゴリー**: generic
- **根拠**: 適合判定は kind ごとの実績あるパッケージ（pydoclint/eslint-plugin-jsdoc/Checkstyle JavadocMethod/revive/rustc 組み込み missing_docs）に委ねられ、waffle 独自の差別化を生まない。sd-validation と同じ理屈で一般（ただし対象データ・外部システムが異なるため別 subdomain）。

---

## 所属ユースケース

- uc-lint-docstring

---

## 実装ガイド

一般ゆえ既存ツールを薄く呼び出すだけ。適合判定そのものは既存ツールに委ね、waffle はツール起動・出力の正規化・境界適応だけを担う。

---

## 外部解決策

kind ごとの既存 lint ツール（pydoclint/eslint-plugin-jsdoc/Checkstyle JavadocMethod/revive/rustc 組み込み missing_docs）を採用する。適合判定そのものと違反詳細の抽出を委ねる。
