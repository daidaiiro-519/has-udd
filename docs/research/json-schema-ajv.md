# JSON Schema（Draft 2020-12）調査メモ

**調査日:** 2026-06-21
**目的:** has-udd の ContractSkills（document.json の Schema 設計）に使える JSON Schema の機能把握

---

## 選定結果 ✅

**採用: `jsonschema`（Python）**

has-udd の実装言語が Python に確定したため、`jsonschema` を採用する。
Schema ファイル（`.has-udd/schemas/*.json`）は JSON Schema Draft 2020-12 形式で記述し、`jsonschema` が読み込んで validate する。

---

## 参考: 言語別の代表ライブラリ

JSON Schema 自体は言語非依存の仕様。参考として主要言語の実装を示す。

| 言語 | 代表ライブラリ |
|---|---|
| **Python（採用）** | **jsonschema** |
| TypeScript | Ajv |
| Go | go-jsonschema / qri-io/jsonschema |
| Rust | jsonschema-rs |

---

## has-udd 設計に直結する機能

### ① $defs + $ref — 共通定義の再利用

共通エンベロープを `$defs` に一度定義し、各 Schema から `$ref` で参照できる。
DRY を保ちながら全 documentType 共通のフィールドを管理できる。

```json
{
  "$defs": {
    "DocumentEnvelope": {
      "type": "object",
      "required": ["documentId", "documentType", "schemaRef", "status"],
      "properties": {
        "documentId":   { "type": "string" },
        "documentType": { "type": "string", "enum": ["Agent", "Skill", "Spec", "Coding"] },
        "schemaRef":    { "type": "string" },
        "status":       { "type": "string", "enum": ["CREATED", "VALIDATED", "RENDERED", "SUPERSEDED"] },
        "refs":         { "type": "object" },
        "tags":         { "type": "array", "items": { "type": "string" } },
        "createdAt":    { "type": "string", "format": "date-time" },
        "updatedAt":    { "type": "string", "format": "date-time" }
      }
    }
  }
}
```

---

### ② oneOf + discriminator — documentType による分岐バリデーション

Ajv の `discriminator` キーワードを使うと、`documentType` の値によって
どの subSchema を適用するかを高速に切り替えられる。

```json
{
  "discriminator": { "propertyName": "documentType" },
  "oneOf": [
    { "$ref": "#/$defs/AgentDocument" },
    { "$ref": "#/$defs/SkillDocument" },
    { "$ref": "#/$defs/SpecDocument" },
    { "$ref": "#/$defs/CodingDocument" }
  ]
}
```

- バリデーションが O(1) になる（全 oneOf を評価しない）
- エラーメッセージが明確になる

---

### ③ allOf — 継承構成（基底 + 拡張）

基底エンベロープ + content Schema を合成できる。OOP の継承に相当。

```json
{
  "$defs": {
    "SpecDocument": {
      "allOf": [
        { "$ref": "#/$defs/DocumentEnvelope" },
        {
          "type": "object",
          "properties": {
            "content": { "$ref": "#/$defs/SpecContent" }
          }
        }
      ]
    }
  }
}
```

---

### ④ if / then / else — 条件付き必須フィールド

2〜3分岐の条件バリデーションに使う。oneOf より読みやすい。

```json
{
  "if":   { "properties": { "templateKind": { "const": "Implementation" } } },
  "then": { "required": ["docCommentFields"] },
  "else": { "required": ["testScenarioRefs"] }
}
```

- 2分岐 → `if/then/else`
- 3分岐以上 → `oneOf` の方が適切

---

### ⑤ TypeScript 型の自動生成（JSONSchemaType）

Ajv の `JSONSchemaType<T>` を使うと Schema から TypeScript 型を生成・同期できる。
ContractSkills（Schema）を書けばランタイムの型定義は不要になる。

```typescript
import Ajv, { JSONSchemaType } from "ajv"
const ajv = new Ajv()

interface UsecaseSpec {
  documentId: string
  documentType: "Spec"
  status: "CREATED" | "VALIDATED" | "RENDERED" | "SUPERSEDED"
}

const schema: JSONSchemaType<UsecaseSpec> = {
  type: "object",
  properties: {
    documentId:   { type: "string" },
    documentType: { type: "string", const: "Spec" },
    status:       { type: "string", enum: ["CREATED", "VALIDATED", "RENDERED", "SUPERSEDED"] }
  },
  required: ["documentId", "documentType", "status"]
}

const validate = ajv.compile(schema)
// validate は TypeScript の型ガードとして機能する
```

---

## has-udd 設計への示唆

**重要前提:** `oneOf` + `discriminator` 等の機能は **Schema ファイル側のバリデーションルール** として使う。document.json 自体の構造には登場しない。

| 設計上の課題 | 使う機能 | 使いどころ |
|---|---|---|
| 全 documentType 共通のエンベロープ定義 | `$defs` + `$ref` | `DocumentEnvelope` を `$defs` に定義し全 Schema から `$ref` |
| documentType → 具体 Schema へのルーティング | `oneOf` + `discriminator` | メタ Schema で `documentType` を discriminator にして Spec / Coding / Job に分岐 |
| schemaRef → 個別 Schema へのルーティング | `oneOf` + `discriminator` | Spec 基底 Schema で `schemaRef` を discriminator に UsecaseSpec / PBISpec 等に分岐 |
| content ブロックの blockType 別構造検証 | `oneOf` + `discriminator` | `content.additionalProperties` で `blockType` を discriminator に Overview / AcceptanceCriteria 等に分岐 |
| エンベロープ + documentType 固有フィールドの合成 | `allOf` | `[DocumentEnvelope, SpecFields]` を合成 |
| CodingTemplate の templateKind 別必須フィールド | `if/then/else` | `templateKind == "Implementation"` → `docCommentFields` 必須 等 |

---

## 参考リンク

- [Ajv 公式](https://ajv.js.org/)
- [TypeScript サポート](https://ajv.js.org/guide/typescript.html)
- [JSON Schema 2020-12](https://json-schema.org/draft/2020-12)
- [条件バリデーション](https://json-schema.org/understanding-json-schema/reference/conditionals)
- [oneOf / allOf / anyOf 合成](https://jsonic.io/guides/json-schema-composition-guide)
