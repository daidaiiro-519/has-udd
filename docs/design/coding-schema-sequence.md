# CodingSchema インスタンス作成と Spec の関係（時系列シーケンス）

CodingSchema の instance（tech-stack / code-template / 最小サンプル）が **いつ作られ**、
**Spec とどう関係し**、**変更時にどう保守されるか**をイベントごとに整理する。

鍵: tech-stack / code-template / サンプルは「スタック採用時に一度だけ」Spec と独立に作られ、
**実装時に初めて両者が合流**する。そして **UDD なので赤テストが先・実装が後**。

> 旧版からの修正: (a) .feature(赤) を実装より**前**に移動（UDD 準拠）/ (b) ステップバインディングを明示 / (c) 変更時シーケンスを追加。

---

## 図1: 作成フロー（初回・happy path）

```mermaid
sequenceDiagram
    autonumber
    actor Dev as 開発者 / Orchestrator
    participant SC as scaffold engine
    participant Coding as Coding docs (tech-stack / code-template)
    participant Smp as 最小サンプル (examples/)
    participant Spec as Spec docs
    participant Feat as .feature + ステップバインディング
    participant Code as ソースコード
    participant Val as validate / Hooks
    participant Rg as ripgrep + reconcile

    Note over Dev,Rg: (1) スタック採用時（プロジェクトに一度きり・Spec と独立）
    Dev->>Smp: 最小1スライスを作り動かす（規約を体現・緑テスト）
    Dev->>SC: tech-stack / code-template を scaffold
    SC-->>Dev: 空 document.json（prompt 付き）
    Dev->>Coding: 規約を蒸留して fill（言語/アーキ/library＋検証ルール）
    Note right of Coding: 具体規約はここ（instance）

    Note over Dev,Rg: (2) Spec 作成時（いつでも・技術非依存）
    Dev->>SC: Spec を scaffold
    SC-->>Dev: 空 Spec document.json
    Dev->>Spec: What を fill（ActorIntent / 受け入れ条件 / TestScenarios）

    Note over Dev,Rg: (3) 実装時（UDD: 赤テスト先 → 実装後）
    Dev->>Rg: 「この Spec の既存実装は?」逆引き
    Rg-->>Dev: 無し→新規 / 有り→修正（重複防止）
    Dev->>Spec: What を読む
    Dev->>Feat: TestScenarios から .feature を render（赤）
    Dev->>Feat: ステップバインディングを書く（.feature ↔ 実装IF）
    Note over Feat: ここで初めてテストが実行可能（まだ赤）
    Dev->>Coding: 規約・library を読む
    Dev->>Smp: 手本を読む（真似る）
    Dev->>Code: 赤に導かれ実装（@spec 決定的埋込・gen-gap 内に本体）
    Feat->>Code: テスト実行
    Code-->>Feat: 緑になるまで実装を直す

    Note over Dev,Rg: (4) 検証・逆引き時（commit / merge）
    Dev->>Val: validate（依存方向 / 命名 / ban / アンカー / テスト緑）
    Val-->>Dev: 違反なら BLOCK
```

---

## 図2: 変更フロー（陳腐化が実際に起きる場所）

初回作成より、こちらの変更ループの方が陳腐化リスクの本体。

```mermaid
sequenceDiagram
    autonumber
    actor Dev as 開発者 / Orchestrator
    participant Spec as Spec docs
    participant Coding as Coding docs (tech-stack / code-template)
    participant Smp as 最小サンプル
    participant Feat as .feature + ステップバインディング
    participant Code as ソースコード
    participant Val as validate / Hooks
    participant Rg as ripgrep + reconcile

    Note over Dev,Rg: ループA: Spec 変更（What が変わった）
    Dev->>Spec: Spec を変更
    Dev->>Rg: 影響する既存実装を逆引き（@spec で）
    Rg-->>Dev: 該当コード一覧
    Dev->>Feat: .feature を再 render（該当が赤に戻る）
    Dev->>Code: 赤に導かれ既存実装を修正（gen-gap 内・本体保護）
    Feat->>Code: テスト → 緑
    Dev->>Val: validate / Hooks

    Note over Dev,Rg: ループB: スタック/規約 変更（library更新・アーキ変更）
    Dev->>Coding: tech-stack / code-template を更新
    Dev->>Smp: サンプルを新規約に更新（手本を最新化）
    Dev->>Val: 全コードを新ルールで再 validate
    Val-->>Dev: 旧規約違反を surface
    Dev->>Code: 違反箇所を移行（AI が修正）

    Note over Dev,Rg: ループC: ドリフト検知（随時・CI）
    Rg->>Code: @spec / @stack を走査
    Rg->>Spec: 実在突合（reconcile）
    Rg-->>Dev: 未実装（Spec有×タグ無）/ orphan（タグ有×Spec無）を surface
```

---

## 時系列の関係（要点）

| イベント | 作られる Coding instance | Spec との関係 |
|---|---|---|
| (1) スタック採用時（一度きり） | tech-stack・code-template・最小サンプル | Spec とは無関係（技術側だけ先に整う） |
| (2) Spec 作成時（いつでも） | （作られない） | Spec は技術非依存で独立に増える |
| (3) 実装時 | （作られない・既存を読む） | ここで Spec × tech-stack × code-template が合流 |
| (4) 検証 / 逆引き | （作られない） | コードの @spec を Spec と reconcile |
| ループA Spec 変更 | （作られない） | Spec→影響コード逆引き→赤→修正→緑 |
| ループB スタック変更 | tech-stack/code-template/サンプルを**更新** | 全コードを新ルールで再 validate→移行 |

---

## 一番大事な3点

1. tech-stack / code-template / サンプル = スタックごとに「一度」作り、**スタック変更時に更新**（ループB）
2. Spec = 技術を知らずに「いつでも」作る（What 専用）。変更時はループA（赤テスト駆動）
3. 実装時に Spec(What) × tech-stack/code-template(How) が合流 → コード（@spec 決定的埋込）

---

## 未解決の設計論点（正直な明記・要ブレスト）

| # | リスク | 状態 |
|---|---|---|
| ④ | **固定サンプルの鮮度**: トークン効率で固定サンプルを手本にしたが、規約進化で腐る。ループBの「サンプル更新」をどうトリガ/保証するか未設計 | 未解決 |
| ⑤ | **②(規約 instance) と ③(サンプル) の drift**: 別々に作るので片方だけ更新されるとズレる。「サンプルが自分の検証ルールを通る」CI 機構が要る | 未解決 |
| — | ループB の「全コード再 validate→移行」の規模・自動化粒度（大規模時の現実性） | Phase 3 で検討 |
