# CodingSchema 設計ブレスト（CodingTemplate / TestTemplate）

## 目的

Coding 集約（CodingSchema）を設計する。**What（Spec）→ How（コード/テスト）の橋渡し**を担う。
最大の課題: **アーキテクチャ別 CodingTemplate の表現**と、**has-udd が守るべき構造的契約 vs エンドユーザーのアーキテクチャ自由の境界**を決めること。

---

## 前提（確定事項から）

- CodingSchema = Coding 集約。CodingTemplate（実装骨格）+ TestTemplate（ステップバインディング骨格）+ DocComment 注入仕様
- **`.feature`（中立）は SpecSchema 側**（TestScenarios + x-render「feature」）。CodingSchema は**言語/アーキテクチャ依存の部分のみ**
- 役割（SP-1 補論）: アーキテクチャ差（hexagonal/clean/onion）・言語差（TS/Python）を CodingTemplate/TestTemplate が吸収。Spec は純 What
- 「1 SSOT → コード骨格 + DocComment」の実体（template-engine・更新は DocComment in-place 注入で本体保護）
- AgentSchema は延期（手動 Orchestrator で動く）。今の焦点は CodingSchema

---

## 論点一覧

| # | 論点 | 状態 |
|---|---|---|
| CO-1 | has-udd の構造的契約 vs ユーザーのアーキテクチャ自由（境界） | ✅ CLOSED（実装テンプレ非提供に着地） |
| CO-2 | CodingSchema の構成（codingKind 3種・構造・render 形式） | ✅ CLOSED |
| CO-3 | CodingTemplate の所有・既定提供（実装テンプレ vs 動く最小サンプル） | ✅ CLOSED（テンプレ非提供・サンプルは example） |
| CO-4 | コード render の engine 扱い（機械生成はどこまでか） | ✅ CLOSED（機械生成=.feature だけ・コードは AI＋validation） |
| CO-5 | DocComment / @spec・@stack アンカー・トレーサビリティ・重複防止 | ✅ CLOSED |
| CO-x | 無駄量産への3層防御（UDD/サブドメイン較正/検証ゲート） | ✅ CLOSED |
| CO-6 | scaffoldable 規約・Steps/Guardrails | 未 |
| 残 | codingKind「code-template 痩せ」確認（規約+検証だけの器を残すか） | 未 |

## 合意事項（CO-1〜CO-5・記録 2026-06）

### 境界（has-udd vs ユーザ）
- has-udd 提供: CodingSchema(構造)・契約定数(@spec/@stack/gen-gap)・query逆引き/validate engine/Hooks(検証と強制の機構)・動く最小サンプル(example)
- ユーザ提供: tech-stack中身・code-templateの検証ルール(サンプルが叩き台)・実コード(AI)
- **has-udd 不提供: 実装テンプレ(コード生成器)・スターターの強制適用** ← 一切なし。理由: AIは言語を知っている・テンプレは腐る・中立でなくなる・組合せ爆発

### codingKind（案B）
- `tech-stack`（言語/アーキ/library束縛+非ドメイン能力レジストリ・1プロジェクト1つ）/ `code-template`（規約+検証ルール・薄い・お手本コード無し）/ `test-template`
- 全て document.json。契約定数 @spec/@stack/gen-gap は CodingSchema ルート共通

### render / 形式
- 3 document 自体 → HTML（人間レビュー・純機械）
- 真の成果物 → ソースコード（対象言語・tech-stack で動的）
- **機械が決定的に生成するのは `.feature` だけ**（純データ変換）。クラス/署名/port/本体は AI が idiom で書き、validation が強制。「署名まで機械生成」は撤回

### トレーサビリティ / 重複防止
- 欲しいのは細かいトレースでなく「重複防止＝アンカー→既存成果物の逆引き」粗くて十分
- アンカー: ドメイン→`@spec:{specId}` / 非ドメイン→`@stack:{capability}`
- **@アノテーションはソースコードの DocComment に持つ**（document.json に持たない＝腐る）。逆引きは query が走査で動的生成（_index 同原則）
- **作業中の Spec は既知文脈 → ソースファイル作成時に @spec/@stack/gen-gap が決定的に埋まる**。validation は backstop
- document.json は宛先の定義(Spec id・tech-stack capability)を持つだけ

### 動く最小サンプル（genesis seed）
- スタックごと1個・最小1スライス(Spec→コード→緑テスト)・規約を体現・`examples/`に分離（framework default にしない）・育てる種

### 無駄量産への3層防御
- ① UDD（存在の門番・主役）: Spec無ければ作らない＝AI過剰生産への一次防御（UDD は問題でなく答え）
- ② サブドメイン較正（厚みの門番）: 中核=厚い/補完=薄い/一般=@stackのみ
- ③ 検証ゲート（一貫性の門番・脇役）: 機械ルールを Hooks で強制
- **重大注意: 検証ゲートを一律にすると過剰設計を量産（本書アンチパターン3）。サブドメインで厚みを変える。has-udd 自身も最小から**

### DDD 裏打ち
- DDD は Spec↔コードを別管理しない（コード=モデル/ユビキタス言語・翻訳はアンチパターン）。振る舞いは Gherkin が仕様=テスト=トレース
- has-udd の Spec が許される条件: ユビキタス言語・実行可能(.feature)・命名と@specで繋ぐ・bounded context内に限定
- 技術(library/middleware)はユビキタス言語の外＝ドメインSpecで管理しないのが DDD 的に正解(@stack+規約)

---

## 論点 CO-1: has-udd の構造的契約 vs ユーザーのアーキテクチャ自由（境界）

### AI の立場（たたき台）

**has-udd はアーキテクチャを規定しない（ユーザーの領域）。だが「非陳腐化・トレーサビリティ・安全な再生成」を成立させる最小の構造的契約は固定する。**

#### has-udd が固定する構造的契約（アーキテクチャ非依存）

| # | 契約 | なぜ固定が要るか |
|---|---|---|
| 1 | **トレーサビリティ（@spec タグ）**: 生成される全コード成果物は DocComment に `@spec:{documentId}` を持つ | これが無いと乖離検知（SP-6）が成立しない |
| 2 | **再生成安全（generation-gap）**: 「生成領域（再 render で上書き可）」と「ユーザー本体（保護）」の境界が**機械検出可能**であること | これが無いと再 render がユーザー実装を壊す（SP-4 の更新が破綻） |
| 3 | **出力マッピング**: 各ドメイン概念型（aggregate / value-object / usecase / scenario）→ 出力パスパターンを宣言 | engine がどこに書くか機械的に決まる必要がある |
| 4 | **テスト連携**: `.feature`（中立）→ ステップバインディング → 実装、の連携。`.feature` がバインディング生成の入力 | テスト↔実装の整合（SP-4/SP-5）の前提 |

→ **これらは「メタ構造の契約」**。アーキテクチャの中身ではなく「追跡できる・安全に再生成できる・どこに出すか」を保証するだけ。

#### ユーザーが定義する（アーキテクチャ/言語依存・自由）

| 項目 | 例 |
|---|---|
| 各ドメイン概念のコード構造（テンプレート本体） | hexagonal: Entity + Repository Port + Adapter / clean: Entity + Gateway + Interactor |
| 言語 / フレームワーク | TS/Python/Java・pytest/Cucumber.js/JUnit |
| 命名規約・フォルダ規約 | — |

→ **CodingTemplate の本体（テンプレート文字列）はユーザーが書く**。has-udd は契約（1〜4）を課すだけ。

#### つまり境界

```
has-udd（固定・契約）:  @spec タグ / generation-gap 境界 / 出力パス宣言 / .feature→バインディング連携
        ──────────────────────────────────────
ユーザー（自由・本体）:  アーキテクチャ別コード構造・言語・命名（CodingTemplate の本体）
```

CodingSchema = 「契約フィールド（has-udd 必須）」 + 「テンプレート本体スロット（user 記入）」を定義する schema。

#### schema と engineSkills を分けて考える（あなたの指摘）

- **CodingSchema**（schema・OSS・パッケージ内）= テンプレートが満たすべき契約・スロットを定義
- **CodingTemplate**（document.json・ユーザープロジェクト）= ユーザーのアーキテクチャ別テンプレート本体（契約を満たす）
- **コード render engine**（CO-4）= Spec × CodingTemplate を読み、契約を守ってコード生成/更新（@spec 注入・gen-gap 尊重・パスへ書き込み）

→ クラス/インスタンス分離（CodingSchema=OSS / CodingTemplate=user）。engine は別途（CO-4）。

---

### ユーザー見解

---

## 合意事項

（論点解決後に記録）

---

## ブレスト完了（2026-06）

CO-1〜CO-5 ＋ 3層防御 ＋ 逆引き方式（A 案・PoC 技術検証済み）すべて CLOSED。
**CodingSchema/v1.json を `.has-udd/schemas/CodingSchema/v1.json` に作成済み**（SkillSchema 規約準拠: content=object・x-render={md,html} Jinja2・x-render-order/level・x-prompt-query/_index・直接フィールド+x-prompt-write）。

具体的な規約の在り場所（3層）:
- CodingSchema（class）= 器の形のみ
- code-template document.json（instance）= 具体規約そのもの（validate が読む SSOT）
- examples/ 最小サンプル（embodiment）= 規約を守って動く手本（AI が真似る）
- スタック採用時に サンプル→規約蒸留 で ②③ を一緒に作る

## 次のアクション（Phase 3）

- code-template / tech-stack / test-template の具体 document.json 作成（規約の中身）
- 動く最小サンプル（examples/）の作成
- 位置限定スキャン・reconcile の engine 具体定義（CO-6 相当）
- サブドメイン較正の具体ルールセット
