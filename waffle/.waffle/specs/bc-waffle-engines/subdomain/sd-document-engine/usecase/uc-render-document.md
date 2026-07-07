# uc-render-document

---

## 概要

検証済みの Document を schema の x-render に従って人間可読な成果物（SKILL.md / HTML）に描画し、配置先へ反映する。

---

## 主アクターと意図

- **主アクター**: Orchestrator（HarnessAgent）
- **意図**: 対象 Document を成果物に描画し、canonical と deploy 先へ反映する

---

## 事前条件

- 対象 Document が存在し、schemaRef を持つ

---

## 基本フロー

```mermaid
sequenceDiagram
    actor Orchestrator
    Orchestrator->>Document: render する
    Document->>Document: x-render から成果物を生成（frontmatter＋部品）
    Note over Document: DocumentRendered
    Document-->>Orchestrator: 生成パス一覧を返す
```

---

## 事後条件

- Document が RENDERED 状態になる
- DocumentRendered が発行される
- 成果物が canonical に書かれ、deploy 先へ verbatim コピーされる
- deploy 先は固定の配列だけでなく、discriminator（specKind等）ごとに異なる配列としても宣言できる（canonicalのpath選択と同じ仕組み）
- パステンプレートの変数は、documentId等の既定変数に加え、schemaがx-render-target.pathVarsでcontentのドットパスを宣言すれば、そのcontent値も変数として使える（x-frontmatterと同型の宣言的解決）
- pathVarsもpath/deployと同様、discriminatorごとに異なる宣言（kindごとの{変数名: ドットパス}の組）としても書ける（discriminatorの分岐によってcontentの形が変わり、参照できるドットパスも変わるため）
- x-frontmatterも同様に、discriminatorの値ごとに異なるフィールド宣言（kindごとの{フィールド名: ドットパス}の組）として書ける（discriminatorによってcontentの形が変わり、frontmatterに出すべきフィールド自体も変わるため。frontmatterを持たないdiscriminator値は宣言しなければ生成されない）

---

## 受け入れ基準

- When 対象 Document が与えられたとき、engine は x-render に従い成果物を生成する shall。
- When deploy が有効なとき、engine は canonical と deploy 先の両方へ書き込む shall。
- When deploy 先が discriminator ごとの配列として宣言されているとき、engine は対象 Document の discriminator 値に対応する配列だけへ書き込む shall。
- When schemaがx-render-target.pathVarsでcontentのドットパスを宣言しているとき、engineはそのcontent値をパステンプレートの変数として使う shall。
- When pathVarsがdiscriminatorごとの宣言（kindごとの変数マップ）であるとき、engineは対象Documentのdiscriminator値に対応する変数マップだけを解決する shall。
- When x-frontmatterがdiscriminatorごとの宣言（kindごとのフィールドマップ）であるとき、engineは対象Documentのdiscriminator値に対応するフィールドマップだけからfrontmatterを生成する shall。
- If schemaRef が無いとき、engine は MISSING_SCHEMA_REF を返し描画しない shall。

---

## 操作保証

- When 同じ Document を複数回 render したとき、engine は常に同一の成果物を生成する shall（決定的：入力が同じなら出力も同じ）。
- When x-render が RenderMetaSchema の各部品種別（paragraph/list/table/keyvalue/code/section/kvtable/sequence/statediagram/architecture/flowchart）を宣言したとき、engine はその種別ごとの整形規則に従って決定的に描画する shall。
- When 対象パスが存在しないとき、engine は INVALID_PATH エラーを返す shall（対象を特定し取得する解決プロセス自体の契約であり、複数のusecaseに共通する）。
- When 対象のschemaRefを解決できないとき、engine は INVALID_SCHEMA_REF エラーを返す shall（schemaを特定し取得する解決プロセス自体の契約であり、複数のusecaseに共通する）。

---

## エラー

---

## 受け入れシナリオ

### 検証済み Document を成果物に描画する

| 分類 | 観点 |
|---|---|
| 正常系 | 描画：x-render に従い成果物と生成パスを返す |

```gherkin
Scenario: 検証済み Document を成果物に描画する
  Given 描画対象の Document
  When render する
  Then 成果物が生成され、生成パス一覧が返る
```

### schemaRef を持たない Document は描画しない

| 分類 | 観点 |
|---|---|
| 異常系 | エラー：schemaRef 欠如は MISSING_SCHEMA_REF |

```gherkin
Scenario: schemaRef を持たない Document は描画しない
  Given schemaRef の無い Document
  When render する
  Then MISSING_SCHEMA_REF エラーが返る
```

### deploy すると canonical と deploy 先の両方に書く

| 分類 | 観点 |
|---|---|
| 正常系 | 受け入れ基準：deploy が有効なとき canonical と deploy 先の両方へ書き込む |

```gherkin
Scenario: deploy すると canonical と deploy 先の両方に書く
  Given deploy 先を持つ Document
  When deploy を有効にして render する
  Then canonical と deploy 先の両方に成果物が書かれる
```

### SkillSchemaをMarkdownにレンダリングする

| 分類 | 観点 |
|---|---|
| 正常系 | schema種別横断：SkillSchemaのDocumentが見出し・パラメータ表・呼び出し例まで正しく描画される |

```gherkin
Scenario: SkillSchemaをMarkdownにレンダリングする
  Given SkillSchemaのDocument
  When renderする
  Then 見出し・目的・パラメータ表・オペレーション選択・呼び出し例が全て出力に含まれる
```

### frontmatterはx_frontmatterのドットパスを解決して生成する

| 分類 | 観点 |
|---|---|
| 正常系 | frontmatter：schemaのx-frontmatter宣言(フィールド→ドットパス)を解決してYAML frontmatterを生成する |

```gherkin
Scenario: frontmatterはx_frontmatterのドットパスを解決して生成する
  Given x-frontmatterを宣言するSchemaのDocument
  When renderする
  Then 出力冒頭にname/description等を含むYAML frontmatterが生成される
```

### CodingSchemaはMarkdownとして描画できる

| 分類 | 観点 |
|---|---|
| 正常系 | schema種別横断：CodingSchemaのDocumentも同一engineで描画できる(schema固有ロジックを持たない汎用性) |

```gherkin
Scenario: CodingSchemaはMarkdownとして描画できる
  Given CodingSchemaのDocument
  When renderする
  Then Markdown形式で見出しを含む出力が生成される
```

### usecase_Specは基本フローをシーケンス図に受け入れシナリオをMarkdownに出す

| 分類 | 観点 |
|---|---|
| 正常系 | schema種別横断：usecase Specは MainFlow をMermaidシーケンス図に、TestScenariosをMarkdownに出す |

```gherkin
Scenario: usecase_Specは基本フローをシーケンス図に受け入れシナリオをMarkdownに出す
  Given usecase SpecのDocument
  When renderする
  Then 出力にmermaidのsequenceDiagramとテストシナリオ節が含まれる
```

### aggregate_Specは集約の構造とライフサイクルをMarkdownに出す

| 分類 | 観点 |
|---|---|
| 正常系 | schema種別横断：aggregate Specはコマンド・ドメインイベント・ライフサイクルをMermaidのstateDiagramと表で出す |

```gherkin
Scenario: aggregate_Specは集約の構造とライフサイクルをMarkdownに出す
  Given aggregate SpecのDocument
  When renderする
  Then 出力にコマンド節・ドメインイベント名・mermaidのstateDiagram-v2が含まれる
```

### 不正なJSONはINVALID_JSON

| 分類 | 観点 |
|---|---|
| 異常系 | エラー：対象ファイルがJSONとして解釈できないときはINVALID_JSON |

```gherkin
Scenario: 不正なJSONはINVALID_JSON
  Given 不正なJSONの対象ファイル
  When renderする
  Then INVALID_JSONエラーが返る
```

### discriminatorごとに異なるdeploy先へ書き分ける

| 分類 | 観点 |
|---|---|
| 正常系 | 受け入れ基準：deploy先がdiscriminatorごとの配列で宣言されているとき対応する配列だけへ書く |

```gherkin
Scenario: discriminatorごとに異なるdeploy先へ書き分ける
  Given deploy先がdiscriminatorの値ごとに異なる配列として宣言されたschemaのDocument
  When deployを有効にしてrenderする
  Then そのDocumentのdiscriminator値に対応する配列のdeploy先だけに書かれる
```

### pathVarsで宣言したcontent値をパステンプレートの変数として使う

| 分類 | 観点 |
|---|---|
| 正常系 | 受け入れ基準：x-render-target.pathVarsが宣言するcontentドットパスの値がパステンプレートに反映される |

```gherkin
Scenario: pathVarsで宣言したcontent値をパステンプレートの変数として使う
  Given x-render-target.pathVarsでcontentのドットパスを宣言したschemaのDocument
  When renderする
  Then そのcontent値がパステンプレートの変数として解決され、対応するパスに書かれる
```

### discriminatorごとに異なるpathVarsを解決する

| 分類 | 観点 |
|---|---|
| 正常系 | 受け入れ基準：discriminatorの分岐によって参照できるcontentドットパスが変わるため、pathVars自体もdiscriminatorごとに宣言できる |

```gherkin
Scenario: discriminatorごとに異なるpathVarsを解決する
  Given discriminatorの値ごとに異なるpathVars宣言（kindごとの変数マップ）を持つschemaのDocument
  When renderする
  Then そのDocumentのdiscriminator値に対応する変数マップだけが解決され、パステンプレートに反映される
```

### discriminatorごとに異なるx_frontmatterを生成する

| 分類 | 観点 |
|---|---|
| 正常系 | 受け入れ基準：discriminatorの分岐によってfrontmatterに出すべきフィールド自体が変わる（frontmatter無しの分岐も許容） |

```gherkin
Scenario: discriminatorごとに異なるx_frontmatterを生成する
  Given discriminatorの値ごとに異なるx-frontmatter宣言（kindごとのフィールドマップ）を持つschemaのDocument
  When renderする
  Then そのDocumentのdiscriminator値に対応するフィールドマップだけからfrontmatterが生成される
```

---

## 操作保証シナリオ

### 同じDocumentを2回renderしても同一の成果物になる

| 分類 | 観点 |
|---|---|
| 境界値 | 決定性：入力が変わらなければ出力も変わらない |

```gherkin
Scenario: 同じDocumentを2回renderしても同一の成果物になる
  Given 変更されていないDocument
  When 同じDocumentを2回renderする
  Then 1回目と2回目の成果物は同一である
```

### x_render宣言どおりに決定的に描画する

| 分類 | 観点 |
|---|---|
| 正常系 | 描画：x-renderがtable部品を宣言したとき、その宣言どおりの構造でMarkdownテーブルとして整形される |

```gherkin
Scenario: x-render宣言どおりに決定的に描画する
  Given interfaceブロック(x-render宣言=table)を持つDocument
  When renderする
  Then schemaのx-render宣言どおりに整形されたMarkdownテーブルが出力に含まれる
```

### 存在しないパスはINVALID_PATH

| 分類 | 観点 |
|---|---|
| 異常系 | 解決契約：対象パスが実在しないとき、パスの解決に失敗しINVALID_PATHになる |

```gherkin
Scenario: 存在しないパスはINVALID_PATH
  Given 実在しない対象パス
  When 本usecaseを実行する
  Then INVALID_PATHエラーが返る
```

### 解決できないschemaRefはINVALID_SCHEMA_REF

| 分類 | 観点 |
|---|---|
| 異常系 | 解決契約：schemaRefを解決できないとき、schemaの解決に失敗しINVALID_SCHEMA_REFになる |

```gherkin
Scenario: 解決できないschemaRefはINVALID_SCHEMA_REF
  Given 解決できないschemaRef
  When 本usecaseを実行する
  Then INVALID_SCHEMA_REFエラーが返る
```
