# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{summary.text}}` | この文脈/業務領域/集約/ユースケースが何かを1〜2文で（ドメイン語彙のみ・技術用語禁止）。 |
| `{{ubiquitousLanguage.items[1].term}}` | 用語（この文脈での呼び名）。 |
| `{{ubiquitousLanguage.items[1].definition}}` | その用語が指すものを1文で。 |
| `{{members.items[1].kind}}` | メンバーの種別。 |
| `{{members.items[1].members}}` | その種別の id を「 / 」区切りで。例: sd-order-management / sd-payment |
| `{{contextMap.items[1].counterpart}}` | 相手の bounded-context 名。 |
| `{{contextMap.items[1].relationship}}` | 関係種別（パートナー/顧客・供給/腐敗防止層 等）。 |
| `{{contextMap.items[1].content}}` | 何を連係するかを1文で。 |

---

# {{title.title}}

---

## 概要

{{summary.text}}

---

## ユビキタス言語

| 用語 | 定義 |
|---|---|
| `{{ubiquitousLanguage.items[1].term}}` | {{ubiquitousLanguage.items[1].definition}} |

---

## 構成要素

| 種別 | メンバー |
|---|---|
| {{members.items[1].kind}} | {{members.items[1].members}} |

---

## 文脈マップ

| 相手コンテキスト | 関係 | 内容 |
|---|---|---|
| {{contextMap.items[1].counterpart}} | {{contextMap.items[1].relationship}} | {{contextMap.items[1].content}} |
