# `OperationGuarantees` / `GuaranteeScenarios` 設計ドラフト（レビュー用・schema未実装）

`docs/brainstorm/brainstorm-waffle-next-evolution.md` 論点6・7の合意を、既存の`AcceptanceCriteria`/`Errors`/`TestScenarios`と同じ構造規約に揃えて具体化する。

---

## 1. `$defs`案（DomainSpecSchema/v2.json に追加する想定）

```json
"OperationGuaranteesBlock": {
  "type": "object",
  "required": ["blockType", "title", "items"],
  "x-render-order": 10,
  "x-render-level": 2,
  "x-prompt-query": "このusecaseがデータアクセス層（依存する資源）に対して呼び出し元に約束する、機能・非機能上の保証(べき等性・一貫性・提供チャネル等)を持ちます。",
  "x-render": [
    { "as": "list", "from": "items" }
  ],
  "properties": {
    "blockType": { "type": "string", "const": "OperationGuarantees" },
    "title":     { "type": "string", "const": "操作保証" },
    "items": {
      "type": "array",
      "x-prompt-write": "保証をEARSで列挙（When/While/If … shall …）。書いてよいのは『何を保証するか』だけ（べき等性・一貫性・提供チャネルの一貫性等）— 『どう実現するか』（具体的なDB技法・ロック機構・実装技術）は絶対に書かない（CodingSchemaのcode-templateの領分）。\n\n判定基準（このusecaseに書くべきか、集約のinvariantsに書くべきか）: 同じ資源を操作する複数usecaseの間で、この保証内容が重複するか？ 重複するなら、ここではなく集約（agg-*.json）のinvariants/unitTestScenariosに書く（例: パストラバーサル拒否は全usecase共通の資源の性質なので集約側）。このusecase固有の業務理由でのみ必要な保証だけをここに書く。\n\n性能の生数値（応答時間・スループット等）は書かない——CodingSchemaのtestTypes(non-functional-performance)の領分。",
      "items": { "type": "string" }
    }
  }
},

"GuaranteeScenariosBlock": {
  "type": "object",
  "required": ["blockType", "title", "scenarios"],
  "x-render-order": 11,
  "x-render-level": 2,
  "x-test-scenario": true,
  "x-prompt-query": "OperationGuaranteesの各保証を検証するテストシナリオ(分類・観点・Gherkin)を持ちます。MD＋.featureにrenderされ、TestScenariosと同じ機構でネイティブテスト化されます。",
  "x-render": [
    { "as": "paragraph", "heading": "背景", "from": "background" },
    { "as": "section", "from": "scenarios", "titleFrom": "name", "each": [
      { "as": "kvtable", "columns": [
        { "field": "category", "header": "分類" },
        { "field": "viewpoint", "header": "観点" }
      ]},
      { "as": "code", "from": "gherkin", "lang": "gherkin" }
    ]}
  ],
  "properties": {
    "blockType": { "type": "string", "const": "GuaranteeScenarios" },
    "title":     { "type": "string", "const": "操作保証シナリオ" },
    "background": {
      "type": "string",
      "x-prompt-write": "複数シナリオ共通の前提。無ければ空文字。"
    },
    "scenarios": {
      "type": "array",
      "minItems": 1,
      "x-prompt-write": "1シナリオ1要素。OperationGuaranteesの各項目を最低1つのシナリオで検証する。呼び出し経路（直接engine呼び出し／CLI／MCP）をまたいで同じ保証が成立するかを確認したい場合は、経路ごとにシナリオを分けてよい（同じ保証・異なるバインディング＝SP-4/SP-5の原則）。",
      "items": {
        "type": "object",
        "required": ["name", "category", "viewpoint", "gherkin"],
        "properties": {
          "name":      { "type": "string", "x-prompt-write": "シナリオ名（概要）。" },
          "category":  { "type": "string", "x-prompt-write": "分類: 正常系 / 異常系 / 境界値。" },
          "viewpoint": { "type": "string", "x-prompt-write": "観点: べき等性/一貫性/提供チャネルの一貫性 等＋検証の狙い。" },
          "gherkin":   { "type": "string", "x-prompt-write": "Given/When/Then。ドメイン語彙で書き、実装詳細は書かない。" },
          "covers":    { "type": "string", "x-prompt-write": "対応するOperationGuaranteesの項目への参照。" }
        }
      }
    }
  }
}
```

---

## 2. 実例①: `uc-scaffold-document` に適用（usecase固有の保証）

createが同じdocumentIdに対してべき等かどうかは、他usecase（query/render/validate）と重複しない、`uc-scaffold-document`固有の業務理由（AI/人が同じcreate要求を誤って複数回送る可能性がある）による保証。

```json
"operationGuarantees": {
  "blockType": "OperationGuarantees",
  "title": "操作保証",
  "items": [
    "When 同じ documentId で create を複数回実行したとき、engine はべき等に振る舞う shall（2回目以降は既存の骨格を上書きしない、または同一結果を返す）。"
  ]
},
"guaranteeScenarios": {
  "blockType": "GuaranteeScenarios",
  "title": "操作保証シナリオ",
  "background": "",
  "scenarios": [
    {
      "name": "同じdocumentIdでcreateを2回実行してもべき等",
      "category": "境界値",
      "viewpoint": "べき等性：同一documentIdへのcreate再実行は安全",
      "gherkin": "Scenario: 同じdocumentIdでcreateを2回実行してもべき等\n  Given 既に作成済みのdocumentId\n  When 同じdocumentIdでcreateを再実行する\n  Then 既存の骨格は上書きされず、同一の結果が返る",
      "covers": "操作保証: createはべき等"
    }
  ]
}
```

---

## 3. 実例②: `uc-query-document` に適用（CLI/MCPチャネル一貫性・論点7）

「INVALID_OPERATIONを返す」という保証がCLI/MCP経由でも同じであることを検証するシナリオ（同じ保証・異なるバインディング）。

```json
"operationGuarantees": {
  "blockType": "OperationGuarantees",
  "title": "操作保証",
  "items": [
    "While 未知のoperationが与えられたとき、engineはCLI・MCPいずれの提供チャネル経由でも同一のINVALID_OPERATIONエラーを返す shall。"
  ]
},
"guaranteeScenarios": {
  "blockType": "GuaranteeScenarios",
  "title": "操作保証シナリオ",
  "background": "",
  "scenarios": [
    {
      "name": "CLI経由でも未知operationはINVALID_OPERATION",
      "category": "異常系",
      "viewpoint": "提供チャネルの一貫性：CLI呼び出しでも直接呼び出しと同じ保証が成立する",
      "gherkin": "Scenario: CLI経由でも未知operationはINVALID_OPERATION\n  Given waffle query コマンド\n  When 未知のoperationを指定して実行する\n  Then INVALID_OPERATIONエラーがJSON出力される",
      "covers": "操作保証: CLI/MCPチャネルの一貫性"
    },
    {
      "name": "MCP経由でも未知operationはINVALID_OPERATION",
      "category": "異常系",
      "viewpoint": "提供チャネルの一貫性：MCPツール呼び出しでも直接呼び出しと同じ保証が成立する",
      "gherkin": "Scenario: MCP経由でも未知operationはINVALID_OPERATION\n  Given query_document MCPツール\n  When 未知のoperationを指定して呼び出す\n  Then INVALID_OPERATIONエラーが返る",
      "covers": "操作保証: CLI/MCPチャネルの一貫性"
    }
  ]
}
```

---

## 4. 確認したい点（レビューポイント）

1. `x-render-order`(10/11)は既存ブロック（Errors=8, TestScenarios=9）の後に続く想定で問題ないか。
2. `GuaranteeScenarios`の`covers`は文字列参照（`TestScenarios`と同じ緩い形）でよいか、それとも構造化した参照にすべきか。
3. 実例②のCLI/MCPシナリオは、既存の`cli.feature`/`mcp.feature`（手書き・spec由来ではない）と内容が重複しないか——重複するなら、こちらへ一本化し`cli.feature`側の該当シナリオを削除する対象になる。
4. usecaseのcontent全体における`required`配列に`operationGuarantees`/`guaranteeScenarios`を必須で加えるか、任意（該当する保証が無いusecaseもある）にするか。
