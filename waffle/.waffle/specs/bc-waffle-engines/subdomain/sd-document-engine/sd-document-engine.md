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
| 整形描画 | x-render宣言(Schema集約の値オブジェクト)とDocument集約のcontent dataの両方を参照してMarkdownへ整形する。RenderMetaSchemaが定義する部品種別(paragraph/list/table/keyvalue/section/kvtable/sequence/statediagram/architecture/flowchart)ごとに決定的な整形規則を持つ。特定の集約に属さない（Schema集約とDocument集約にまたがる計算）。 |

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

### paragraph・listが正しく整形される

| 分類 | 観点 |
|---|---|
| 正常系 | paragraph/listの整形保証 |

```gherkin
Scenario: paragraph/listが正しく整形される
  Given paragraph/listを宣言するx-render
  When renderする
  Then paragraphは地の文、listは箇条書きとして整形される
```

### tableはパイプ文字をエスケープしboolを整形する

| 分類 | 観点 |
|---|---|
| 境界値 | tableのセルエスケープ・bool整形保証 |

```gherkin
Scenario: tableはパイプ文字をエスケープしboolを整形する
  Given パイプ文字やbool値を含む行データ
  When tableとしてrenderする
  Then パイプ文字はエスケープされ、boolは✓/-に整形される
```

### sectionは入れ子とitemLabelを整形する

| 分類 | 観点 |
|---|---|
| 正常系 | sectionの入れ子・itemLabel整形保証 |

```gherkin
Scenario: sectionは入れ子とitemLabelを整形する
  Given itemLabelを持つsection宣言と入れ子のeach部品
  When renderする
  Then 各itemの見出しにitemLabelが付与され、入れ子の部品も正しく描画される
```

### keyvalueが正しく整形される

| 分類 | 観点 |
|---|---|
| 正常系 | keyvalueの整形保証 |

```gherkin
Scenario: keyvalueが正しく整形される
  Given keyvalueを宣言するx-render
  When renderする
  Then ラベルと値の組が箇条書きとして整形される
```

### sectionはbadgeで条件付き強調を付与する

| 分類 | 観点 |
|---|---|
| 境界値 | sectionのbadge（条件付き強調）保証 |

```gherkin
Scenario: sectionはbadgeで条件付き強調を付与する
  Given badge条件を満たすitemを含むsection宣言
  When renderする
  Then 条件を満たすitemの見出しにのみ強調語が付与される
```

### tableはmarkFieldで識別子を太字強調する

| 分類 | 観点 |
|---|---|
| 境界値 | tableのmarkField（識別子強調）保証 |

```gherkin
Scenario: tableはmarkFieldで識別子を太字強調する
  Given markFieldが真の行を含むtable宣言
  When renderする
  Then 該当セルが太字＋markSuffixで強調される
```

### statediagramが正しいMermaid構文になる

| 分類 | 観点 |
|---|---|
| 正常系 | statediagramのMermaid構文生成保証 |

```gherkin
Scenario: statediagramが正しいMermaid構文になる
  Given 状態遷移の配列を宣言するx-render
  When renderする
  Then stateDiagram-v2として正しいMermaid構文が生成される
```

### statediagramは疑似状態を表現する

| 分類 | 観点 |
|---|---|
| 境界値 | statediagramの疑似状態（choice/fork/join）保証 |

```gherkin
Scenario: statediagramは疑似状態を表現する
  Given pseudoStatesFromで疑似状態を宣言するx-render
  When renderする
  Then choice/fork/joinの疑似状態宣言がMermaid構文の先頭に出力される
```

### sequenceはactor・participantを区別する

| 分類 | 観点 |
|---|---|
| 境界値 | sequenceのactor/participant区別保証 |

```gherkin
Scenario: sequenceはactor/participantを区別する
  Given kind:actor/participantを含む参加者宣言
  When renderする
  Then actor/participantそれぞれの宣言がMermaid構文で区別される
```

### sequenceはloop・altを入れ子で表現する

| 分類 | 観点 |
|---|---|
| 正常系 | sequenceのloop/alt入れ子保証 |

```gherkin
Scenario: sequenceはloop/altを入れ子で表現する
  Given loop/alt種別のstepを含むsteps配列
  When renderする
  Then loop/altブロックが正しく入れ子のMermaid構文になる
```

### sequenceはactivate・deactivateを表現する

| 分類 | 観点 |
|---|---|
| 境界値 | sequenceのactivate/deactivate保証 |

```gherkin
Scenario: sequenceはactivate/deactivateを表現する
  Given activate/deactivateフラグを持つstep
  When renderする
  Then Mermaidのアクティベーション記法(+/-)が正しく付与される
```

### architectureが正しいMermaid構文になる

| 分類 | 観点 |
|---|---|
| 正常系 | architectureのMermaid構文生成保証 |

```gherkin
Scenario: architectureが正しいMermaid構文になる
  Given zones/connectionsを宣言するx-render
  When renderする
  Then architecture-betaとして正しいMermaid構文が生成される
```

### flowchartが正しいMermaid構文になる

| 分類 | 観点 |
|---|---|
| 正常系 | flowchartのMermaid構文生成保証 |

```gherkin
Scenario: flowchartが正しいMermaid構文になる
  Given stages/transitionsを宣言するx-render
  When renderする
  Then flowchart LRとして正しいMermaid構文が生成される
```

### kvtableは単一行として整形される

| 分類 | 観点 |
|---|---|
| 境界値 | kvtableの単一行整形保証 |

```gherkin
Scenario: kvtableは単一行として整形される
  Given kvtableを宣言するx-render
  When renderする
  Then block自身の値が1行のtableとして整形される
```

### tableはjoin指定で配列セルを結合整形する

| 分類 | 観点 |
|---|---|
| 境界値 | tableのjoin（配列セル結合）保証 |

```gherkin
Scenario: tableはjoin指定で配列セルを結合整形する
  Given join/sepを指定したcolumns宣言と配列値を持つセル
  When renderする
  Then 配列の各要素がjoinテンプレートで整形されsepで連結される
```
