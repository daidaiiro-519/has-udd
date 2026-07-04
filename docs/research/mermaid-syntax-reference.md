# Mermaid記法 調査メモ（sequenceDiagram / stateDiagram-v2）

**目的:** RenderMetaSchemaの`sequence`/`statediagram`パートが将来サポートしうる記法の全体像をWeb調査で確定する。特に「人間のアクターを表す専用の記法があるか」を確認する。
**出典:** [Mermaid公式ドキュメント](https://mermaid.js.org/syntax/sequenceDiagram.html)・[stateDiagram](https://mermaid.js.org/syntax/stateDiagram.html)（2026-07時点）

---

## sequenceDiagram

### 参加者の宣言

**`participant`（既定・矩形）:**
```
participant A
```

**`actor`（人間のアクター専用・棒人間アイコン）:**
```
actor A
```
→ **これが今回探していた「アクター用の表現」**。`顧客`のような人間の主体は`participant`でなく`actor`で宣言すれば、システム（usecase等）とは視覚的に区別される棒人間アイコンで描画される。

**特殊記号（boundary/control/entity/database/collections/queue）:**
```
participant B as boundary
participant C as control
participant E as entity
participant DB as database
participant Coll as collections
participant Q as queue
```

**別名（エイリアス）:**
```
participant A as Alice
actor B as Bob
```

**グループ化（box）:**
```
box Aqua Group Label
  participant A
  participant B
end
```

**生成/破棄:**
```
create participant B
A --> B: Hello
destroy A
B --> A: Goodbye
```

### メッセージの矢印種別

| 記法 | スタイル |
|---|---|
| `->` | 実線・矢頭なし |
| `-->` | 破線・矢頭なし |
| `->>` | 実線・矢頭あり |
| `-->>` | 破線・矢頭あり |
| `<<->>` | 実線・双方向 |
| `<<-->>` | 破線・双方向 |
| `-x` | 実線・×印（失敗/エラー表現） |
| `--x` | 破線・×印 |
| `-)` | 実線・非同期矢印 |
| `--)` | 破線・非同期矢印 |

### 活性化（activation）

```
activate A
deactivate A
```
省略形（矢印に`+`/`-`を付ける）:
```
A->>+B: 呼出
B-->-A: 応答
```

### ノート

```
Note right of A: 右側の注記
Note left of A: 左側の注記
Note over A: 単一参加者上の注記
Note over A,B: 複数参加者にまたがる注記
```

### 制御構造

**ループ:**
```
loop 説明
  A->>B: メッセージ
end
```

**条件分岐（alt/else）:**
```
alt 条件1
  A->>B: パス1
else 条件2
  A->>B: パス2
end
```

**任意ブロック（opt）:**
```
opt 任意の動作
  A->>B: メッセージ
end
```

**並行（par/and）:**
```
par アクション1
  A->>B: メッセージ1
and アクション2
  C->>D: メッセージ2
end
```

**中断（break）:**
```
break 例外発生
  A->>B: エラー処理
end
```

**背景ハイライト（rect）:**
```
rect rgb(0, 255, 0)
  A->>B: 緑背景のやりとり
end
```

### その他

- `autonumber`：メッセージへの自動連番
- `%% コメント`：行コメント
- `#35;`のようなHTMLエンティティコードで特殊文字をエスケープ

---

## stateDiagram-v2

### 基本

```
stateDiagram-v2
  state1
  state1 --> state2: 遷移ラベル
```

### 開始・終了状態

```
[*] --> InitialState
FinalState --> [*]
```
→ Waffleの`agg-document`Lifecycle（`CREATED→VALIDATED→RENDERED→SUPERSEDED`）は、この`[*]`開始記法をそのまま使っている。

### 複合（入れ子）状態

```
state Composite {
  InnerState1
  InnerState2
}
```

### 分岐（choice）

```
state Choice <<choice>>
State1 --> Choice
Choice --> State2
Choice --> State3
```

### 並行（fork/join）

```
State1 --> <<fork>>
fork --> State2
fork --> State3
State2 --> <<join>>
State3 --> <<join>>
```

### ノート

```
note right of State1
  注記の内容
end note
```

### 方向指定

```
direction LR
```
（`LR`/`TB`/`RL`/`BT`）

### スタイル（classDef）

```
classDef styleName property:value;
class State1,State2 styleName
```

---

## 現状のWaffle実装との対応

`part_renderer.py`の`_sequence`/`_statediagram`は、上記のごく一部（`->>`/`-->>`/`Note over`、`-->`のみ）しか実装していない。今回の`PresentationSpecSchema`の「業務ユースケースの並び」の課題は、**`actor`宣言を使えば解決可能**——`顧客`を`participant`でなく`actor`として宣言すれば、システム（usecase）とは視覚的に区別された人間のアイコンで描画され、「顧客が各usecaseを呼び出す」という関係を、違和感なく複数参加者間のやりとりとして表現できる。

**次のアクション:** `_sequence()`に`actor`宣言（`kind`とは別に、参加者ごとの役割区分）を追加することを、PresentationSpecSchemaのブレストに戻して検討する。
