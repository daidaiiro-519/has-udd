# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{naming.items[1].target}}` | 命名の対象。例: モジュール / ファイル |
| `{{naming.items[1].convention}}` | 命名規約。例: スネークケース |
| `{{style.items[1].rule}}` | 規約の内容。 |
| `{{docstring.style}}` | 言語標準の docstring スタイル。例: Google スタイル docstring |
| `{{docstring.summaryGuidance}}` | 要約行（1行目）の書き方の指針。「何をするか・いつ使うか」を検索と判断に効く語で、の旨。 |
| `{{docstring.target}}` | docstring を必須にする対象。例: 公開要素（module / 公開 class / 公開 function）。private は任意。 |
| `{{docstring.example}}` | このスタイルに従った docstring のコード例（要約行＋本文＋Args/Returns 等・カスタムタグなし）。 |
| `{{rules.items[1].rule}}` | 規約の内容。 |

---

# {{title.title}}

---

## 命名

| 対象 | 規約 |
|---|---|
| {{naming.items[1].target}} | {{naming.items[1].convention}} |

---

## スタイル

| 種別 | 規約 |
|---|---|
| 必須 | {{style.items[1].rule}} |

---

## docstring

- **スタイル**: {{docstring.style}}
- **対象**: {{docstring.target}}

### 要約行の書き方

{{docstring.summaryGuidance}}

```
{{docstring.example}}
```

---

## 決定ルール

| 種別 | 規約 |
|---|---|
| 必須 | {{rules.items[1].rule}} |
