---
name: "harness-query-engine"
description: "document.json および通常ファイルに対してセマンティック操作によるクエリを実行し、{ prompt, value } 形式で結果を返す。AI がファイルを直接読んで解釈することなく、Python スクリプトが一切のデータアクセスを担う。"
---

# harness-query-engine

---

## 目的

document.json および通常ファイルに対してセマンティック操作によるクエリを実行し、{ prompt, value } 形式で結果を返す。AI がファイルを直接読んで解釈することなく、Python スクリプトが一切のデータアクセスを担う。

---

## 役割

- セマンティック操作（16本）を受け取り、対応する Python 処理にルーティングする
- content 各ブロックの blockType から schema の x-prompt-query を引いて index を動的算出するインデックス走査（ファイル / ディレクトリ横断）を実行する。index は document.json に保存しない
- document.json の blockKey 単位でコンテンツを取得する（ブロックレベル操作）
- 配列の取得・スライス・key/正規表現フィルタ・id 検索・ネスト展開・全階層の再帰検索を実行する
- schemaRef を持たない通常ファイルを raw フォールバックとして返す
- すべてのエラーを { error, message } 形式でラップして返す（例外を AI に素通りさせない）

---

## インターフェース

**入力は自然言語のテキスト（要望）**。受け取ったテキストから下表の **パラメータに成型** し、『呼び出し』の形に当てはめて呼ぶ（実行手順参照）。**出力は `{ prompt, value }`**（`value` はそのまま・`prompt` が読み方）。

### パラメータ（テキストから成型して埋める・呼び出しに渡す値）

| name | type | 必須 | 説明 | 例 |
|---|---|---|---|---|
| operation | string | ✓ | セマンティック操作名（一覧は Step 2 のオペレーション表で選ぶ） | get_block |
| path | string | ✓ | 対象ファイル（index_scan_dir はディレクトリ）。要望テキストで指定される | .waffle/documents/skills/harness-query-engine.json |
| blockKey | string | - | 取得するブロックのキー（Group 2・3 で必須） | interface |
| arrayField | string | - | 操作対象の配列フィールド名（Group 3 で必須） | input |
| field | string | - | 取得/フィルタ対象のフィールド名 | description |
| idField | string | - | ID として使うフィールド名 | stepId |
| idValue | string | - | ID の値 | step-1 |
| key | string | - | フィルタ条件のキー名（filter_items） | required |
| value | any | - | フィルタ条件の値（filter_items） | true |
| pattern | string | - | 正規表現（filter_pattern） | ^uc- |
| start | integer | - | スライス開始（get_items_slice） | 0 |
| end | integer | - | スライス終了（get_items_slice） | 5 |
| fieldName | string | - | 再帰検索するフィールド名（find_all） | stepId |
| nestedField | string | - | 展開するネスト配列（get_nested_items） | children |

### 出力（value の中身）

| name | type | 必須 | 説明 | 例 |
|---|---|---|---|---|
| prompt | string/null | ✓ | value の読み方の指針（blockType→x-prompt-query から動的算出）。Group 1・再帰では null | この engine の I/O 定義を持ちます… |
| value | any | ✓ | クエリ結果（そのまま返る・型は operation 依存） | [{ "name": "operation", … }] |
| type | string | - | raw フォールバック時のみ "raw" | raw |
| content | string | - | raw フォールバック時のみ。生テキスト | # README… |
| error | string | - | エラー時のみ。INVALID_OPERATION/INVALID_PATH/INVALID_JSON/INVALID_PATTERN/MISSING_PARAM/NOT_FOUND/NO_MATCH | MISSING_PARAM |
| message | string | - | エラー時のみ。人間可読の詳細 | blockKey は get_block で必須です |

---

## 呼び出し

選んだ operation と成型したパラメータを、下記の CLI / MCP の形に当てはめて呼ぶ（各パラメータの意味は『インターフェース』参照）。

### Skills（CLI）

```
uv run --project waffle waffle query --operation <operation> --path <path> [--<param> <value> ...]
```

例:

```
uv run --project waffle waffle query --operation get_block --path .waffle/documents/skills/harness-query-engine.json --blockKey interface
```

```
uv run --project waffle waffle query --operation filter_items --path .waffle/documents/skills/harness-query-engine.json --blockKey interface --arrayField input --key required --value true
```

### MCP

```
query_document({ "operation": "<operation>", "path": "<path>", ...<param>: <value> })
```

例:

```
query_document({"operation": "get_block", "path": ".waffle/documents/skills/harness-query-engine.json", "blockKey": "interface"})
```

MCP は uv run --project waffle waffle serve 起動後に利用可。

---

## 実行手順

### Step 1: 要望テキストから対象と取得内容を読み取る

対象 path（ファイル/フォルダ）は要望テキストに含まれている前提（呼び出し側＝Orchestrator が指定）。無ければ『対象の指定が必要』と返す。あわせて、どの意味的単位（ブロック/フィールド/条件）が欲しいかを読み取る。

### Step 2: オペレーションを選ぶ

欲しい結果から下表で operation を1つ選び、必須引数を『インターフェース』のパラメータ表で成型する。

- 構造を知りたい → `index_scan` / `index_scan_dir`
- 特定ブロック → `get_block` / `get_field`
- 配列を絞る → `filter_items` / `filter_pattern` / `get_by_id`
- 全階層検索 → `find_all`

| operation | 用途 | 必須引数 | 例 |
|---|---|---|---|
| `scan` | 生テキスト取得 | - | path=README.md |
| `get_meta` | メタ情報(documentId/status 等) | - | path=<doc> |
| `index_scan` | 1 doc の block 一覧を動的算出 | - | path=<doc> |
| `index_scan_dir` | ディレクトリ横断で index 集約 | path=ディレクトリ | path=.waffle/documents/skills |
| `get_block` | block を丸ごと取得 | blockKey | blockKey=interface |
| `get_field` | block の1フィールド取得 | blockKey, field | blockKey=purpose field=text |
| `get_items` | 配列要素を全取得 | blockKey, arrayField | blockKey=interface arrayField=input |
| `get_item_field` | 配列要素の特定フィールド | blockKey, arrayField, field | arrayField=input field=name |
| `get_items_slice` | 配列をスライス | blockKey, arrayField, start, end | start=0 end=5 |
| `filter_items` | key==value で配列フィルタ | blockKey, arrayField, key, value | key=required value=true |
| `filter_exists` | field を持つ要素を抽出 | blockKey, arrayField, field | field=example |
| `filter_pattern` | field が正規表現一致 | blockKey, arrayField, field, pattern | field=name pattern=^uc- |
| `get_by_id` | idField==idValue の要素 | blockKey, arrayField, idField, idValue | idField=stepId idValue=step-1 |
| `get_nested_items` | ネスト配列を展開 | blockKey, arrayField, nestedField | nestedField=children |
| `get_children` | 指定要素の children | blockKey, arrayField, idField, idValue | idField=stepId idValue=step-1 |
| `find_all` | 全階層を再帰検索 | fieldName | fieldName=stepId |

### Step 3: 選んだ operation と成型したパラメータで呼ぶ

Step 2 で選んだ operation と成型したパラメータを『呼び出し』の CLI / MCP の形に当てはめて実行する（『呼び出し』に具体例あり）。

### Step 4: 返り値 {prompt, value} を使う

`value` が結果（そのまま）、`prompt` が読み方の指針。AI はファイルを直接読まず value を根拠に判断する（Harness 原則）。

### Step 5: 不足・空一致・エラーに対処する

要望に対象が無ければ『対象の指定が必要』と返す。`NO_MATCH` は空一致（正常系・`value: []`）。`INVALID_*` / `MISSING_PARAM` は入力を見直す。

---

## ガードレール

- 対象（path）は呼び出し側が要望テキストで指定する。無ければ実行せず『対象の指定が必要』と返す
- 読み取り専用。ファイルへの書き込み・削除は行わない
- AI はファイルを直接読まず、engine が返す value を根拠にする（Harness 原則）
- 結果が空でも正常系として NO_MATCH（value: []）を返す
- 例外は握りつぶさず { error, message } で返す（エラーコードは『出力』の error を参照）

---

## 参照

- `.waffle/documents/`: クエリ対象となる全集約の document.json 配置ディレクトリ（skills / specs / knowledge / agents / coding）
- `docs/brainstorm/design-engine-query.md`: query engine 設計ブレスト（Q-1〜Q-5）
