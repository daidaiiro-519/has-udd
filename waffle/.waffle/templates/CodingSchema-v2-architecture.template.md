# {{documentId}}

## プレースフォルダー定義票

| プレースフォルダー | 記入する内容 |
|---|---|
| `{{title.title}}` | documentId をそのまま設定してください。 |
| `{{layers.style}}` | 技術方式。例: ポートとアダプター（ヘキサゴナル） |
| `{{layers.items[1].layer}}` | レイヤー名。例: domain, application, ports, inbound adapter, outbound adapter |
| `{{layers.items[1].responsibility}}` | このレイヤーの責務を1文で。 |
| `{{layers.items[1].mayDependOn[1]}}` | 依存してよいレイヤー名を列挙（内向きのみ）。最内層は空。（配列。この形式の行を必要な数だけ繰り返す） |
| `{{layout.tree}}` | 正典ディレクトリツリー（テキスト）。`{package}` 等プレースホルダを使ってよい（プロダクト非依存）。 |
| `{{layout.compositionRoot}}` | 結線（DI）を置く場所を1文で。例: inbound adapter の起動点にのみ置く |
| `{{conceptPlacement.items[1].concept}}` | 固定語彙から選ぶ（spec の specKind と対応）。 |
| `{{conceptPlacement.items[1].placement}}` | layout のツリー上の配置（相対パス）。例: application/usecases |
| `{{conceptPlacement.items[1].pattern}}` | この概念の実現方針を決定レベルで（正確な署名は書かない）。例: エントリメソッド1つ・ドメインは port 経由で呼ぶ |
| `{{rules.items[1].rule}}` | ルールの内容。 |
| `{{thicknessBySubdomain.items[1].category}}` | subdomain spec の Category。 |
| `{{thicknessBySubdomain.items[1].thickness}}` | 実装の厚み方針を1文で。 |

---

# {{title.title}}

---

## レイヤーと依存方向

### 様式

{{layers.style}}

| レイヤー | 責務 | 依存してよい先 |
|---|---|---|
| {{layers.items[1].layer}} | {{layers.items[1].responsibility}} | {{layers.items[1].mayDependOn[1]}} |

---

## ディレクトリ構成

```
{{layout.tree}}
```

### 合成ルート（結線・DI）

{{layout.compositionRoot}}

---

## 概念 → 実現形

| 概念 | 配置 | 形（決定レベル） |
|---|---|---|
| `{{conceptPlacement.items[1].concept}}` | {{conceptPlacement.items[1].placement}} | {{conceptPlacement.items[1].pattern}} |

---

## 規約（守るべきルール）

| 種別 | 規約 |
|---|---|
| 必須 | {{rules.items[1].rule}} |

---

## サブドメイン別の厚み

| Category | 実装の厚み |
|---|---|
| {{thicknessBySubdomain.items[1].category}} | {{thicknessBySubdomain.items[1].thickness}} |
