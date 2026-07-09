# 状態遷移図（stateDiagram-v2）

## 概要

オブジェクトの状態と、状態を変化させるイベント・条件を表現する図。集約の状態ライフサイクルを可視化するのに適している。

## 使いどころ

- 集約（注文・申請・チケット等）のライフサイクル
- 業務プロセスの状態管理
- ドメインイベントによる状態遷移

## 使わないケース

- 処理の順序（誰が何を呼ぶか） → `sequenceDiagram`
- 静的な構造 → `flowchart` or `classDiagram`

---

## 基本テンプレート

```mermaid
stateDiagram-v2
    [*] --> 初期状態
    初期状態 --> 次の状態 : イベント名
    次の状態 --> [*]
```

`[*]` は開始・終了を表す（矢印の始点なら開始状態、終点なら終了状態）。

---

## 状態の宣言

**単純な状態:**

```
stateDiagram-v2
    Still
```

**長い名前・スペースを含む名前へのエイリアス:**

```
stateDiagram-v2
    state "長い状態名（スペースを含む）" as id
    [*] --> id
    id --> OtherState
```

---

## 遷移（トランジション）

```
stateDiagram-v2
    Still --> Moving
    Moving --> Still
    Moving --> Crash
```

**ラベル（イベント名・条件）付き遷移:**

```
stateDiagram-v2
    Still --> Moving : イベント名
```

---

## 開始・終了状態

```
stateDiagram-v2
    [*] --> Still
    Crash --> [*]
```

---

## 複合状態（サブ状態・composite state）

```
stateDiagram-v2
    state Composite {
        InnerState1
        InnerState2
        InnerState1 --> InnerState2
    }
```

**入れ子（ネスト）:**

```
stateDiagram-v2
    state Level1 {
        state Level2 {
            NestedState
        }
    }
```

**複合状態同士の遷移:**

```
stateDiagram-v2
    state Comp1 {
        State1
    }
    state Comp2 {
        State2
    }
    Comp1 --> Comp2
```

---

## 並行状態（`--` ディバイダによるフォーク領域）

複合状態の中を `--` で区切ると、複数の領域が並行して動作することを表現できる。

```mermaid
stateDiagram-v2
    state Composite {
        State1
        --
        State2
    }
```

## fork / join（明示的な分岐・合流ノード）

```mermaid
stateDiagram-v2
    state fork_state <<fork>>
    state join_state <<join>>

    State1 --> fork_state
    fork_state --> State2
    fork_state --> State3
    State2 --> join_state
    State3 --> join_state
    join_state --> State4
```

## choice（選択疑似状態）

条件によって遷移先が分岐する場合に使う。

```mermaid
stateDiagram-v2
    state choice_state <<choice>>
    State1 --> choice_state
    choice_state --> State2 : 条件A
    choice_state --> State3 : 条件B
```

---

## 注釈（note）

```
note right of State1
    補足説明（複数行可）
end note

note left of State2 : 一行の注釈
```

---

## 方向（direction）

```
stateDiagram-v2
    direction LR
    State1 --> State2
```

選択肢: `TB`（既定）/ `LR` / `BT` / `RL`

---

## コメント

```
%% これはコメント
State1 --> State2 %% 行末コメント
```

---

## スタイリング

⚠️ **動作確認済みの注意点**: `class <状態名> <クラス名>` の対象に日本語等の非ASCII状態名を直接指定するとレクサエラーになる。`state "日本語名" as alias` でASCIIエイリアスを宣言し、`class alias クラス名`のようにエイリアス側を指定すること（遷移の矢印では日本語名を直接使ってよい）。

**classDefとclassによる適用:**

```
classDef movement font-style:italic;
classDef badEvent fill:#f00,color:white,font-weight:bold;

class State1 movement
class State2 badEvent
```

**ショートハンド（`:::`）:**

```
classDef highlight fill:#ff0
State1:::highlight --> State2
```

---

## 実例

### 例1: 注文のライフサイクル

```mermaid
stateDiagram-v2
    [*] --> 下書き : 注文作成

    下書き --> 確定済み : 注文確定
    下書き --> キャンセル済み : キャンセル

    確定済み --> 発送済み : 発送
    確定済み --> キャンセル済み : キャンセル

    発送済み --> 完了 : 受取確認

    完了 --> [*]
    キャンセル済み --> [*]
```

### 例2: 複合状態（サブ状態）+ 注釈

```mermaid
stateDiagram-v2
    [*] --> 処理中

    state 処理中 {
        [*] --> 検証中
        検証中 --> 承認待ち : 検証OK
        検証中 --> エラー : 検証NG
        承認待ち --> 承認済み : 承認
    }

    note right of 検証中 : 入力値の妥当性を確認する

    処理中 --> 完了 : 承認済み
    処理中 --> 差し戻し : エラー
```

### 例3: 並行状態（fork/join）+ choice

```mermaid
stateDiagram-v2
    [*] --> 並行処理

    state fork_state <<fork>>
    state join_state <<join>>

    並行処理 --> fork_state
    fork_state --> 在庫確認
    fork_state --> 決済処理

    在庫確認 --> join_state
    決済処理 --> join_state

    state 判定 <<choice>>
    join_state --> 判定
    判定 --> 完了 : 全て成功
    判定 --> 差し戻し : いずれか失敗
```
