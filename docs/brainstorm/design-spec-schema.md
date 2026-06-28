# SpecSchema 設計ブレスト（Spec family / specKind）

## 目的

has-udd の **Spec 集約（specKind family）** を設計する。これは has-udd の要（かなめ）= Usecase-Driven Development（UDD）の土台。
DDD と整合し、**陳腐化しない・疎結合な spec 駆動**を成立させる Spec 構造を確定する。

---

## 設計哲学（最重要・全論点の判断軸）

### spec 駆動の本質的ジレンマ

| 要請 | 内容 |
|---|---|
| **整合性** | ドキュメント管理なので、仕様と実装は必ず整合していないといけない（ズレた仕様は嘘） |
| **疎結合** | だが仕様/設計/実装が密結合だと保守コスト爆発・果ては「仕様＝コードの鏡」で無価値化 → 仕様無視 → 泥団子 |

### 解（has-udd が目指すもの）

1. **仕様は安定した「何を（What）」に抽象化**し、揮発する「どうやって（How）」から切り離す
2. **境界**: 仕様は**ドメイン（業務・安定）に結合**し、**実装（コード・揮発）には結合しない**（DDD の核 = ドメインモデルは安定核）
3. **整合の契約 = 実行可能な受け入れ条件 / テストシナリオ**。実装はテストが通る限り自由に変えてよい。仕様は**望む振る舞い/ドメインが変わったときだけ**変わる
4. **UDD の規律**: 「何をするか」が変われば Spec を先に更新（Spec が先・実装が後）。「どうするか」が変わっても Spec は変えない
5. **AI による陳腐化防止**: コード変更が「What（仕様）」に及ぶか「How（実装）」だけかを判定。テストで整合を確認・乖離を警告

### つまり「spec はヘキサゴナルの契約」

```
Spec（ドメイン契約・安定）  ← 受け入れ条件/テストで定義
     ↑ 満たす
実装（アダプター・揮発）     ← テストが通る限り自由に変えられる
```
実装（How）を変えてもドメイン契約（What）は不変 → 仕様は陳腐化しない・密結合しない。

---

## DDD の3層（specKind の構成根拠）

ユースケース単体では業務領域を構成できない。spec 駆動には3層が整理されている必要がある:

```
ビジネスドメイン
  └─ サブドメイン（中核/一般/補完）≈ 区切られた文脈（bounded context）
       ├─ ユビキタス言語
       ├─ ドメインモデル（集約・エンティティ・値オブジェクト・業務ルール・ドメインイベント）
       └─ ユースケース（1 Actor + 1 Intent + 1 Aggregate・ドメインモデルに対する操作）
```

| specKind（候補） | DDD 層 | 安定度 | 主な内容 |
|---|---|---|---|
| **bounded-context** | 戦略 | 最も安定 | サブドメイン分類・ユビキタス言語・境界・配下 spec へのリンク |
| **domain-model** | 戦術 | 安定 | 集約・エンティティ・値オブジェクト・業務ルール・ドメインイベント・ユニットテスト仕様 |
| **usecase** | アプリケーション | 振る舞い契約 | 1 Actor + 1 Intent + 1 Aggregate・受け入れ条件・テストシナリオ・参照集約 |

---

## 論点一覧

| # | 論点 | 状態 |
|---|---|---|
| SP-1 | spec/実装の境界・抽象化・疎結合・陳腐化防止の設計原則（哲学の具体化） | ✅ CLOSED |
| SP-2 | Spec family（specKind）の構成と3層の関係・相互参照 | ✅ CLOSED |
| SP-3 | 各 specKind の content 設計（blockType） | ✅ CLOSED |
| SP-4 | 整合性の契約（テスト）の表現・実行・更新 | ✅ CLOSED |
| SP-5 | **テスト戦略（シナリオ SSOT・多層テスト・観点分離・ピラミッド）** | ✅ CLOSED |
| SP-6 | status lifecycle・render 出力の整理・AI 乖離検知 | ✅ CLOSED |
| SP-7 | scaffoldable schema 規約準拠・x-render-target / x-prompt | ✅ CLOSED |

---

## 論点 SP-7: scaffoldable schema 規約準拠・x-render-target / x-prompt ✅ CLOSED

### 合意内容

#### ① scaffoldable schema 規約の準拠（SC-2）
- discriminator = **specKind**（bounded-context / domain-model / usecase）。if/then/else で content 分岐（SkillSchema の skillKind と同型）
- **値フィールドに oneOf/anyOf 禁止**（形が一意に決まらず scaffold が機械生成できない。バリエーションは discriminator 経由で表現）／ 開いた additionalProperties 禁止 ／ 有界再帰

#### ② x-render-target（specKind 別・SP-6 の出力①②を駆動）
| specKind | formats | 出力 |
|---|---|---|
| bounded-context | `["html"]` | HTML のみ |
| domain-model | `["html", "feature"]` | HTML + UnitTestScenarios の .feature |
| usecase | `["html", "feature"]` | HTML + TestScenarios の .feature |

```json
"x-render-target": {
  "formats": ["html", "feature"],
  "path": ".has-udd/specs/{context}/usecase/{documentId}.html",
  "featurePath": ".has-udd/specs/{context}/usecase/{documentId}.feature"
}
```
※ コード骨格③は CodingTemplate（Coding 集約・Phase 3）の x-render-target が別途駆動。SpecSchema は ①② まで。

#### ③ x-prompt 系
- `x-prompt-write`: scaffold の fillTemplate（値記入指示）
- `x-prompt-query`: query/knowledge の index 動的計算
- `x-frontmatter`: HTML 主体なので最小

#### ④ テスト/コード生成に必要な追加アノテーション
- `x-test-scenario`: TestScenarios/UnitTestScenarios ブロックを .feature に変換する目印（Gherkin 構造）
- `x-spec-tag`: DocComment に注入する `@spec:{documentId}` の生成元（乖離検知の安定タグ・SP-6）

### 根拠
- discriminator = specKind で SkillSchema/AgentSchema と一貫・scaffold SC-3 がそのまま効く
- x-render-target に featurePath を持たせ render-engine が汎用のまま .feature を出せる（出力先は schema 宣言の原則）
- x-test-scenario / x-spec-tag で「どのブロックがテストか・どのタグを注入するか」を schema 宣言 → engine 汎用維持

---

## 合意事項（全 SP CLOSED）

| # | 合意 |
|---|---|
| SP-1 | 仕様はドメイン+テスト（What）に結合・実装（How）に結合しない。補論: 詳細設計は CodingTemplate・1 SSOT→doc+コード骨格 |
| SP-2 | specKind=bounded-context/domain-model/usecase（DDD3層）。依存は Domain 核へ・保存参照は usecase.aggregateRef のみ・メンバーはフォルダ由来 |
| SP-3 | content（ドメイン語彙のみ）。AcceptanceCriteria=EARS / TestScenarios=Gherkin。ContextRelations 非保存（導出） |
| SP-4 | TestScenarios→TestTemplate で .feature に render。更新は .feature 再生成+DocComment 注入のみ・本体は赤テスト駆動 |
| SP-5 | シナリオ=What の SSOT。ステップバインディングで観点（受け入れ→結合→UIコンポーネント→E2E）切替。Unit と純ビジュアルだけ別 |
| SP-6 | render は全 specKind（HTML/.feature）。lifecycle CREATED→VALIDATED→RENDERED→SUPERSEDED。乖離検知は @spec タグ起点・Hooks 強制 |
| SP-7 | discriminator=specKind・oneOf 禁止・x-render-target に featurePath・x-test-scenario / x-spec-tag |

---

## 次のアクション

SpecSchema ブレスト完了 → Phase 3 で UsecaseSpecSchema / DomainModelSpecSchema / BoundedContextSpecSchema の具体定義。

---

## 論点 SP-6: status lifecycle・render 出力の整理・AI 乖離検知 ✅ CLOSED

### render 出力の整理（重要・混同回避）

**spec 配下の全 document.json が render 対象。** render が生成するもの3種:

| 出力 | 何から | implementation か |
|---|---|---|
| ① 人間向け HTML | 全 specKind の content | ❌ 純 What |
| ② .feature（Gherkin） | usecase.TestScenarios / domain-model.UnitTestScenarios | ❌ 純 What（仕様のシナリオを実行可能形に出すだけ） |
| ③ コード骨格 + DocComment | **Spec × CodingTemplate（Coding 集約・Phase 3・別機構）** | ❌ How は CodingTemplate 側・Spec に無い |

- **「usecase だけ render」は誤り**。全 specKind が HTML render 対象
- **③ は Spec 単体の render ではなく Spec × CodingTemplate**（2入力）。Spec は純 What のまま・密結合しない（SP-1 補論）
- ② Gherkin は仕様内 TestScenarios を `.feature` に書き出すだけ（実装でない）

### status lifecycle（全 specKind 共通）

`CREATED → VALIDATED → RENDERED → SUPERSEDED`
- CREATED: AI が値を埋めた直後（粗い）
- VALIDATED: 受け入れ条件・シナリオが揃い jsonschema 検証通過
- RENDERED: **①HTML（+ usecase/domain-model は ②.feature）生成済み**（コード生成③は別・Phase 3・status と無関係）
- SUPERSEDED: 後続版に置換
- status 書き換えは明示操作（validate は判定のみ・SC-4 と同じ）

### AI 乖離検知（陳腐化防止・SP-1 原則4 の具体化）

```
コード変更検知（Hooks）→ 変更ファイルの DocComment @spec:uc-id から影響 UsecaseSpec 特定
  → そのテスト（.feature）実行 → 緑=整合 / 赤=「振る舞い変化。Spec を先に更新（UDD 規律）」
```
責務: SpecSchema = 検知に必要なメタ（TestScenarios・@spec 紐付け）を持つだけ / Hooks（Phase 5）= 検知・実行・警告の機械強制 / AI = 影響 Spec 特定・What/How 判断

---

## 論点 SP-5: テスト戦略（シナリオ SSOT・多層テスト・観点分離） ✅ CLOSED

### 背景

UsecaseSpec.TestScenarios（Gherkin）= **受け入れテスト（実行可能な受け入れ条件・Specification by Example）**。UAT の自動化された形に通じる。
この「シナリオを SSOT として多層テストに展開する」戦略は has-udd の重要な価値なので独立論点とする。

### 合意内容

#### 核心: 振る舞い（シナリオ）を1回だけ定義し、観点（レベル）を ステップバインディング で切り替える

```gherkin
# UsecaseSpec.TestScenarios（1回書くだけ・SSOT・What）
Then 注文が "CONFIRMED" 状態で作成される
```
```typescript
// ステップバインディング A: 受け入れレベル（アプリサービスを叩く）
When('顧客が注文を確定する', () => { result = new CreateOrder(repo).execute(cart); });
// ステップバインディング B: E2E レベル（実 API を叩く）
When('顧客が注文を確定する', async () => { result = await fetch('/api/orders', {...}); });
```

- **シナリオ（What）は共通・ステップバインディング（観点/How）だけ差し替え** → テストの意図を各レベルで書き直さない（重複ゼロ）
- Spec を変えれば全レベルが同じシナリオを参照 → レベル間でズレない（整合が自動）
- **これが「楽になる」本質 = テスト意図を1か所（Spec）に集約**

#### テストレベル（ピラミッド）と has-udd の対応

```
UAT          ユーザー承認（手動/自動）
E2E          実システム丸ごと（同シナリオ × UI/API ステップバインディング）
Acceptance   業務言語の振る舞い ← UsecaseSpec.TestScenarios（受け入れ）
Component/Integration  （ステップバインディング で実現可）
Unit         集約の不変条件 ← DomainModelSpec.UnitTestScenarios（別シナリオ）
```

別軸（目的別・直交）: Contract / Smoke / Regression / Performance / Security 等。

#### 観点（レベル）の分離 ── 正直な線引き

| 楽になる | 実作業が残る |
|---|---|
| 振る舞い定義（シナリオ）は1回・全レベル共通・SSOT | **ステップバインディング は各レベルで書く**（E2E は実 API/DB 繋ぎで重い） |
| レベル間整合は Spec が保証 | — |

- **Unit は別シナリオ**（集約の不変条件・DomainModelSpec）。「シナリオ使い回し」は主に受け入れ↔E2E
- **全シナリオを全レベルでやらない**（ピラミッド）。E2E は重要パスのみ・受け入れで大半カバー

### 補足A: Unit シナリオは集約ルールと1:1（別シナリオ）

ユニット = 集約の業務ルール/不変条件を**1ルール1シナリオ**で・ドメインオブジェクト直接（new Order / order.cancel()）。ユースケースの流れとは別物（DomainModelSpec.BusinessRules ↔ UnitTestScenarios が対応）。
- 別シナリオな理由: 1集約に多数のルール・ユースケースは一部の道筋しか通らない・ルールは複数ユースケースに跨る・単独検証で失敗箇所が一発で分かる

### 補足B: UI も scenario 駆動（コンポーネント〜E2E）。「UI は別」はアーキテクチャの依存方向だけ

| 観点 | UI |
|---|---|
| アーキテクチャ（hexagonal） | UI = Primary Adapter・ドメインコアの外（依存はコアへ）← 「別」の唯一の意味 |
| テスト | **同じユースケースシナリオが UI を直接駆動**（コンポーネント / E2E）・別ではない |

**同じシナリオが層を貫く（ステップバインディング を実物にしていくと層が上がる）:**
```
Unit            集約ルール              ← DomainModelSpec.UnitTestScenarios（別シナリオ）
Acceptance      usecase（スタブ）        ← UsecaseSpec.TestScenarios
Integration     usecase × 実DB          ← 同シナリオ × 実依存 ステップバインディング
UI Component    UI単独描画 × stub backend ← 同シナリオ × コンポーネント ステップバインディング（振る舞い）
E2E             実UI × 実backend         ← 同シナリオ × ブラウザ ステップバインディング
─────────────────────────────
Visual/Cosmetic ピクセル・色・レスポンシブ ← 唯一の非シナリオ（ビジュアル回帰・装飾）
```
- UI 振る舞い（コンポーネント & E2E）= 同じユースケースシナリオで駆動（ステップバインディング が違うだけ）。**E2E 偏重ではない・コンポーネント中心 + E2E は重要パスのみ**
- 純粋な見た目（装飾）だけが scenario 外（スナップショット/ビジュアル回帰ツール）
- Component テストの語: バックエンド=モジュール単独 / フロント=UI コンポーネント単独。文脈依存

### 補足C: 結合テストは「同シナリオ × 実依存 ステップバインディング」
結合は別シナリオでなく ステップバインディング の繋ぎ方（受け入れ=スタブ / 結合=実DB等 / E2E=全実物）。純インフラ結合（マイグレーション・キュー）は別。

### 合意

シナリオ（Spec）= What の SSOT・1回定義。ステップバインディング = 観点/レベル（受け入れ→結合→UIコンポーネント→E2E）。Unit と純ビジュアルだけが別。ピラミッドで「全部を全レベルでやらない」。意図の重複が消え整合が自動で取れるのが has-udd の価値。

---

## 論点 SP-4: 整合性の契約（テスト）の表現・実行・更新 ✅ CLOSED

### 合意内容

**仕様のテストシナリオが TestTemplate 経由で実テストコードに render され、テストが緑 = 仕様↔実装が振る舞いレベルで整合。** SP-1 補論「CodingTemplate で構造を render」のテスト版。

```
UsecaseSpec.TestScenarios（Gherkin・What の SSOT）
  ├─ render → 人間向け HTML（読める仕様）
  └─ render × TestTemplate → 実テストコード（pytest/jest 等・DocComment で Spec にリンク）
実装コード ← このテストを満たすよう書かれる（緑 = 整合）
```

- テストの SSOT = UsecaseSpec.TestScenarios（Gherkin）/ DomainModelSpec.UnitTestScenarios（2層: ユースケース振る舞い / 集約不変条件）
- TestTemplate は Coding 集約（Phase 3）。技術スタック差（pytest/jest/JUnit）を吸収

### ⭐ 更新の扱い（既存コードへの再 render）

**実装本体は要件から導出不可能 → render は本体を更新しない。** render が触るのは:

| 部分 | 更新方法 | render が触る |
|---|---|---|
| DocComment（Spec リンク・説明） | AST で該当シンボルの DocComment だけ再注入（安定タグ `@spec:uc-id`・冪等） | ✅ |
| テストコード | TestTemplate で再 render（generation-gap・シナリオ生成部は上書き可/カスタムは別領域） | ✅ |
| 実装本体（ロジック） | 触らない（開発者/AI が所有） | ❌ |

**本体の更新は「失敗するテスト」が駆動（UDD=TDD）:**
```
① Spec の振る舞いが変わる → テスト再 render（赤）→ 赤テストに導かれ本体を人/AI が修正
② 実装だけ変わる（How）→ テストは Spec 由来で不変・緑なら整合（Spec も render も触らない）
③ コード変更検知（Hooks・Phase 5）→ 影響 UsecaseSpec 特定 → テスト実行 → 赤なら「振る舞いが変わった→Spec を先に更新」を促す
```

### 難所（正直な評価）
- **既存コードへの DocComment in-place 注入（本体保護）が template-engine のコア難所**。AST 解析 + 安定タグ（`@spec:uc-id`）で該当 DocComment だけ冪等更新。言語別（JavaDoc/JSDoc/docstring）は TestTemplate/CodingTemplate が吸収
- feasible だが非自明 → Phase 3（Coding 集約）で詰める
- 乖離の機械的検知・強制は Hooks（Phase 5）

---

## 論点 SP-3: 各 specKind の content 設計（blockType） ✅ CLOSED

### 合意内容（全 content はドメイン語彙のみ・インフラ語彙禁止）

**BoundedContextSpec（戦略）:**
| blockType | 内容 |
|---|---|
| Purpose | この文脈の目的・責務範囲 |
| SubdomainClassification | 中核 / 一般 / 補完 + 判定理由 |
| UbiquitousLanguage | 用語 → 定義（この文脈の語彙・AI grounding） |

※ **ContextRelations は保存 block にしない**（高結合で陳腐化リスク）。文脈の地図はドメインイベントフロー + 参照から**導出するビュー**（将来・多文脈時に render で生成）。連係パターン（ACL 等）が要るときのみ最小限注釈。

**DomainModelSpec（戦術・1集約）:**
| blockType | 内容 |
|---|---|
| Purpose | この集約が表す業務概念 |
| AggregateRoot | ルートエンティティ・識別子・不変条件 |
| Entities | 集約内エンティティ |
| ValueObjects | 値オブジェクト（属性・等価性） |
| BusinessRules | 業務ルール（不変条件・制約） |
| DomainEvents | この集約が発行するドメインイベント |
| UnitTestScenarios | 集約の不変条件を検証するユニットテスト仕様 |

**UsecaseSpec（アプリケーション・1操作）:**
| blockType | 内容 | 記法 |
|---|---|---|
| Purpose | Actor が何を達成するか | — |
| ActorIntent | Actor（誰が）+ Intent（何を達成したいか） | — |
| AggregateRef | 操作対象の集約（SP-2 の唯一の保存参照） | ID 参照 |
| Preconditions | 事前条件 | — |
| AcceptanceCriteria | 受け入れ条件 | **EARS**（When/While/If…shall…・一意な要件文） |
| TestScenarios | 振る舞い契約（仕様↔実装の整合点・SP-1 原則2） | **Gherkin**（Given/When/Then・実行可能） |
| DomainEvents | このユースケースで発生するドメインイベント | — |

### 記法と可読性の両立
- source = EARS / Gherkin（精密・AI/機械向け）／ rendered = 整形 + 凡例 + 自然文補足（人間向け）
- render-engine が人間向けに整形 → 記法の馴染みの無さは render で解決（store は精密・render で可読）

### 2層テスト
- UnitTestScenarios（domain-model）= 集約の不変条件 / TestScenarios（usecase）= ユースケースの振る舞い

---

## 論点 SP-2: Spec family（specKind）の構成と3層の関係 ✅ CLOSED

### 合意内容

**3つの specKind を DDD の3層に対応させる。依存は最内核（Domain）へ向く。メンバー・所属はフォルダ由来（動的・保存しない）。**

| specKind | DDD 層 | 1ファイルの単位 | content（SP-3 で詳細） |
|---|---|---|---|
| **bounded-context** | 戦略 | 1区切られた文脈（≈サブドメイン） | サブドメイン分類（中核/一般/補完）・ユビキタス言語・（将来）文脈地図 |
| **domain-model** | 戦術 | 1集約 | 集約・エンティティ・値オブジェクト・業務ルール・ドメインイベント・ユニットテスト仕様 |
| **usecase** | アプリケーション | 1 Actor + 1 Intent + 1 Aggregate | 受け入れ条件・テストシナリオ・aggregateRef |

### bounded-context を入れる根拠
- **ユビキタス言語 = AI の grounding に必須**（この文脈で語が何を指すか）
- サブドメイン分類（戦略的優先度）・文脈地図の置き場
- **フォルダ境界 = 文脈境界**（正しいフォルダ管理）

### フォルダ構成
```
.has-udd/documents/specs/
  {bounded-context}/
    context.json              ← BoundedContextSpec
    domain-model/{id}.json
    usecase/{id}.json
```

### 参照の向き（DDD 依存規則: 最内核へ向く）
- **保存する参照は usecase.aggregateRef → domain-model の1つだけ**（アプリケーション→ドメインの依存・「この usecase はこの集約を操作する」というドメインの事実・フォルダから導出不可）
- domain-model は上位参照を持たない（安定核・何にも依存しない）
- **メンバー一覧・所属文脈は保存しない → フォルダ由来（query index_scan_dir で動的取得）**。`_index` 動的化と同じ原則・陳腐化回避
- domain-model → usecase / context への参照は**依存逆転違反なので禁止**

### アーキテクチャパターンとの整合
ヘキサゴナル/クリーン/オニオンのいずれでも依存は Domain（最内核）へ向く。Spec は「Domain 中心」という共通項だけ持ち、表現差（Port/Adapter vs Gateway/Interactor）は CodingTemplate が吸収（SP-1 補論）。

---

## 論点 SP-1: spec/実装の境界・疎結合・陳腐化防止の設計原則

### AI の立場（たたき台）

**仕様はドメイン（What）に結合し実装（How）に結合しない。整合は「実行可能なテストシナリオ」を契約として担保する。これにより疎結合と非陳腐化を両立する。**

#### 原則1: 仕様が参照してよいもの／いけないもの

| 仕様が結合してよい（安定） | 仕様が結合してはいけない（揮発） |
|---|---|
| ドメイン概念（集約・値オブジェクト・業務ルール・ドメインイベント） | クラス名・メソッド名・ファイルパス・DB スキーマ |
| 振る舞い（Actor が Intent を達成する・受け入れ条件） | アルゴリズム・データ構造・フレームワーク |
| ユビキタス言語の語彙 | 実装技術・ライブラリ |

→ Spec の content は**ドメイン語彙のみ**で書く（インフラ語彙禁止・Custom Skill の Steps と同じ規律）。

#### 原則2: 整合の契約 = テストシナリオ

- UsecaseSpec の**受け入れ条件をテストシナリオ（実行可能）**として持つ
- 実装はテストを満たす限り自由。テストが通る = 仕様と実装が**振る舞いレベルで整合**（実装詳細に依存しない）
- 「仕様 ↔ 実装」の結合点は**テスト（What）だけ**。実装構造には結合しない

#### 原則3: 変更の向き（UDD 規律）

```
「何を」が変わる → Spec を先に更新 → テスト更新 → 実装（TDD と同型）
「どうやって」が変わる → Spec 不変・テスト不変・実装だけ変わる（テストが整合を保証）
```

#### 原則4: AI による陳腐化防止

- コード変更時: 影響を受ける UsecaseSpec を AI が特定（refs / ドメイン概念のマッピング）
- テスト実行で整合確認。テストが通れば仕様は有効（更新不要）
- 振る舞いが変わった（テストが意図的に変わる）場合のみ仕様更新を促す
- Hooks（Phase 5）で機械的に「実装変更時に紐づく Spec / テストを確認」を強制

#### これが「泥団子化」を防ぐ理屈

仕様が実装構造に結合すると、実装を変えるたびに仕様も直す＝二重保守＝やがて放置＝泥団子。
仕様を**ドメイン + テスト（What）に結合**すれば、実装（How）の変更は仕様に波及しない。仕様は「コードの鏡」ではなく「ドメイン契約」として価値を保ち、陳腐化しない。

---

### 合意（SP-1 CLOSED）

仕様はドメイン（What）に結合し実装（How）に結合しない。整合は受け入れ条件/テストシナリオ（実行可能契約）が担保する。実装はテストが通る限り自由。

### 補論: 詳細設計・アーキテクチャ・SSOT 分裂の解（重要）

**詳細設計（技術スタック・アーキテクチャパターン）は Spec に入れない（密結合＝陳腐化を招く）。Coding 集約の CodingTemplate が持つ。**

| 集約 | 持つもの |
|---|---|
| Spec（What・安定） | bounded-context / domain-model / usecase（ドメイン語彙のみ） |
| Coding（How・揮発吸収） | CodingTemplate = ドメイン概念 → コード構造の写像（アーキテクチャ別・全 usecase で再利用＝DRY） |

- **アーキテクチャ差分の吸収**: アーキテクチャパターンはプロジェクトで1つ選ぶ定数。同じ Spec × CodingTemplate 差し替えで hexagonal/clean/onion 別のコード構造になる（差分は CodingTemplate が吸収）
- **「1 SSOT から document もコードも」= 構造は実現可能**: Spec（What の SSOT）→ render → 人間 document ＋（CodingTemplate 経由で）コード骨格 + DocComment 注入。1 か所直せば両方更新。**これが template-engine（DocComment 注入・Phase 3）の正体**
- **限界**: コード本体（ロジック）は要件から導出不可能 → render できない。だが構造の陳腐化は render で解決・振る舞いの整合はテストが検知
- **SSOT の割れ方**: What-SSOT（Spec）→ render で doc/コード骨格 / How-SSOT（コード本体）。境界 = 受け入れテスト。実装変更してもテストが通れば Spec 不変 → 泥団子化しない
- → CodingTemplate と template-engine（DocComment 注入）が「魔法の仕組み」の実体。Phase 3 Coding 集約設計の意義

---

## 合意事項

（論点解決後に記録）

---

## 次のアクション

SP-1〜SP-6 解決後 → UsecaseSpecSchema / DomainModelSpecSchema / BoundedContextSpecSchema の具体定義
