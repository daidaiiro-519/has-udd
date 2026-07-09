# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{summary.text}}` | この画面/フローが何のために存在するかを1〜2文で（業務ユースケースの内容の繰り返しでなく、プロダクトとしての意図）。 |
| `{{usecaseSequence.participants[1].id}}` | 識別子。人間主体は名前（例: 顧客）、業務ユースケースは documentId（例: uc-place-order）。 |
| `{{usecaseSequence.participants[1].kind}}` | 人間の主体なら actor（棒人間アイコン）、業務ユースケース等のシステム側なら participant。 |
| `{{usecaseSequence.participants[1].label}}` | 表示名（任意・省略時はidそのまま）。 |
| `{{usecaseSequence.items[1].from}}` | この旅を辿る主体（participantsで宣言したid・例: 顧客）。 |
| `{{usecaseSequence.items[1].to}}` | 呼び出す業務ユースケースのdocumentId（participantsで宣言したid・例: uc-place-order）。 |
| `{{usecaseSequence.items[1].message}}` | この旅の中でこの業務ユースケースが果たす役割を1文で。 |
| `{{usecaseSequence.items[1].kind}}` | 常に command（業務ユースケースの実行呼び出し）。 |

---

# {{title.title}}

---

## 概要

{{summary.text}}

---

## 業務ユースケースの並び

```mermaid
sequenceDiagram
    {{usecaseSequence.participants[1].kind}} {{usecaseSequence.participants[1].id}} as {{usecaseSequence.participants[1].label}}
    {{usecaseSequence.items[1].from}}->>{{usecaseSequence.items[1].to}}: {{usecaseSequence.items[1].message}}
```

| 主体 | 業務ユースケース | この旅における役割 |
|---|---|---|
| {{usecaseSequence.items[1].from}} | `{{usecaseSequence.items[1].to}}` | {{usecaseSequence.items[1].message}} |
