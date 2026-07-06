# sd-schema-governance

---

## 概要

Schema自身のバージョニング・後方互換性・既存Documentの移行を統治する業務領域。Documentではなく Schema そのもの（agg-schema）を対象データとする。

---

## カテゴリー

- **カテゴリー**: core
- **根拠**: Schemaのバージョン管理・実証的な後方互換性検証・機械的な移行(x-migration)は、waffle独自の差別化機構であり既製品で代替できない。ゆえに中核。

---

## 所属ユースケース

- uc-migrate-schema-version

---

## 実装ガイド

中核ゆえドメインモデルで厚く実装する。x-migration宣言の解釈・実証的検証(既存ValidateEngineの再利用)は自前の決定的コードで書く。
