# sd-document-engine

---

## 概要

AI に構造を推論させず、engine が決定的に Document を生成(scaffold)・読取(query)・描画(render)する中核の業務領域。waffle の差別化そのもの（Harness 原則と UDD ループ）を担う。

---

## カテゴリー

- **カテゴリー**: core
- **根拠**: 『AI に構造を推論させず engine が構造を持つ／Spec を正本にして陳腐化させない UDD ループ』は waffle の競争優位の源泉であり、既製品で置換できない。ゆえに中核。旧 sd-rendering（補完）はこの subdomain に統合した——描画(render)は宣言的 x-render の閉じた語彙・複数種の Mermaid 図生成・決定的な .feature 抽出まで複雑化しており、単純な CRUD/ETL ではなくなっている。この複雑さは Harness 原則という同じ差別化に資するものであり、理由のない複雑化ではないため『補完→中核』の変化基準に合致する。対象データ（Document）・差別化原理（Harness 原則）が scaffold/query と同一である以上、subdomain を分ける意味も薄れていた。

---

## 所属ユースケース

- uc-scaffold-document
- uc-query-document
- uc-render-document

---

## 実装ガイド

中核ゆえドメインモデルで厚く実装する。schema 走査による骨格生成・意味単位アクセス・x-render 宣言に従った機械描画は、いずれも自前の決定的コードで書き、外部ライブラリやテンプレートエンジンに委ねない。

---

## ドメインサービス

| 業務サービス | 責務 |
|---|---|
| パステンプレート解決 | x-source-target/x-render-target のパステンプレートを document の値で解決(resolve)し、逆に実パスからテンプレート変数を復元(reverse-parse)する。scaffold/render/query の複数usecaseが共通して依存する（特定の集約に属さない）。 |

---

## 業務サービスシナリオ

### パステンプレートは変数を解決する

| 分類 | 観点 |
|---|---|
| 正常系 | パステンプレート解決：{var}形式のプレースホルダを実際の値に置き換える |

```gherkin
Scenario: パステンプレートは変数を解決する
  Given 変数を含むパステンプレートと解決に必要な値
  When resolve する
  Then 全ての変数が値に置き換わった実パスが返る
```

### 逆解析は実パスからテンプレート変数を復元する

| 分類 | 観点 |
|---|---|
| 正常系 | パステンプレート解決：reverse-parseはresolveの逆写像として変数を一意に復元する |

```gherkin
Scenario: 逆解析は実パスからテンプレート変数を復元する
  Given パステンプレートと、そのテンプレートから解決された実パス
  When reverse-parse する
  Then resolve時に使った値と同じ変数が復元される
```

### テンプレートと一致しないパスは復元できない

| 分類 | 観点 |
|---|---|
| 異常系 | パステンプレート解決：構造が一致しない実パスはreverse-parse不能として扱う |

```gherkin
Scenario: テンプレートと一致しないパスは復元できない
  Given テンプレートの区切り構造と一致しない実パス
  When reverse-parse する
  Then 復元は失敗する
```
