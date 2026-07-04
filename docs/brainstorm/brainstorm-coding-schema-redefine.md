# ブレインストーミング: CodingSchema 再定義（実装方法の確定 → code-template → 最小サンプル → 効果測定）

**目的:** Stage B の土台＝「conformant とは何か」を確定する。**先に has-udd の実装方法を DDD で決め**、それに沿って code-template 規約を再定義し、動く最小サンプルと効果測定の要件まで固める。整合ゲート（reconcile）はこの後段。
**モード:** 設計判断（論点 → 見解 → 合意）

> ⚠️ **なぜ再定義か:** 既存合意（`design-coding-schema`・CLOSED）と SpecSchema enrich の「dm-document＝集約」が衝突して見えた。**Re-1 で解決済**: Document は集約で正しく、has-udd は集約の不変条件を **schema に宣言的にカプセル化**する（engine は executor）。よって**コードに imperative な集約クラスは不要＝bootstrap が正しい realize**。衝突の真因は「集約を imperative で実装すると思い込んだ」こと。

> 🔀 **スコープ注記（重要）:** 本 doc は途中から **SpecSchema/v2 の再構成**（Re-1=集約の宣言的 realize／Re-2=遷移 guard の宣言／Re-4=spec への明示／Re-5=specKind 階層＋subdomain＋VO）が中心になった。これらは **Spec 層の関心事＝独立ステージ「SpecSchema/v2 再構成」**として扱う（roadmap 参照）。**CodingSchema/Stage B 固有に残るのは Re-3（サンプル＋効果測定）＋ 遷移 guard の“置き場”だけ**。両者は結合する（gate が spec と code-template を突き合わせる）が、層が違うので分離する。

---

## 現状の CodingSchema（再定義の対象）

**codingKind 3種**: `tech-stack`（言語/アーキ/library束縛＋非ドメイン能力レジストリ）／`code-template`（規約＋検証ルール・薄い・お手本コード無し）／`test-template`。

**現 code-template（`python-hexagonal.json`）の中身:**
- conventions（概念→フォルダ→命名）: `port` / `usecase`(=engine) / `domain-service` / `outbound-adapter` / `inbound-adapter` / `shared`。**aggregate / value-object / entity は無い**。
- validationRules: anchor-required / dep-inward（依存方向）/ lib-via-adapter / no-print / engine-run / gen-gap。
- subdomainRichness: core=「**domain-service + application（汎用engine）**」/ supporting=transaction-script / generic=library-integration。← **core すら集約でなくサービス指向**。

**維持する合意（揺らさない）:** 実装テンプレ非提供／機械生成は `.feature` だけ／アンカーは DocComment・descriptor 非保存・ripgrep 逆引き／3層防御（UDD・サブドメイン較正・検証ゲート）／動く最小サンプルは `examples/`。

---

| # | 論点 | 状態 |
|---|---|---|
| Re-1 | has-udd の実装方法は何か＝集約を持つか持たないか | ✅ 合意（集約は正しい・realize は宣言的 schema） |
| Re-2 | schema が守れない「status 遷移」をどこで守るか | ✅ CLOSED（Re-1 の帰結をそのまま適用・実質的異論なし） |
| Re-3 | 動く最小サンプルの要件と「効果測定」の定義 | ✅ CLOSED（合格基準4点を確定・実装は未着手） |
| Re-4 | dm-document spec の「集約」framing をどうするか（Re-1 の波及） | ✅ CLOSED（Re-1 の帰結をそのまま適用） |
| Re-5 | spec 階層の切り直し（A案）＋ has-udd の subdomain 分類（中核/一般/補完） | ✅ advisor 判断で確定 |

> ★CLOSED の経緯（2026-07）: Re-2/3/4 は「AI初期見解」に対しユーザーから実質的な異論が出ず、
> Stage S の既存結論（不変条件=schema宣言的カプセル化・engine=executor・imperative集約クラス不要）
> をそのまま適用しただけと判明したため、追加のユーザー見解収集ラウンドを経ずに CLOSED とした。
> **決定はここで確定したが、実装（Re-2の遷移guard関数・Re-3のexamples/最小サンプル）はまだ無い**
> ＝roadmap.md Stage K の残タスクとして引き継ぐ。
>
> ⚠️ **本 doc「現状の CodingSchema」節は古い**: codingKind は 3種（tech-stack/code-template/
> test-template）でなく、CS-2/CS-3（`brainstorm-coding-schema.md`）で確定した **4種
> （tech-stack/architecture/coding-standard/test-standard）** に置き換わっている。
> `code-template` は `architecture`（ConceptPlacement等）＋`coding-standard`（命名/スタイル/docstring）
> に分割済み。

---

## 論点 Re-1: has-udd の実装方法は何か（集約を持つべきか）★核心

### AI 初期見解（昨日の C-1 から自己訂正あり）
**見解:** **has-udd の中核ロジックは“単純”（検証＋機械的変換が主）＝トランザクションスクリプト/アプリケーションサービスが DDD 的に正しい。ドメインモデル/集約をデフォルトにしない。** 昨日の私の C-1「集約が要る(YES)」は**撤回**する。

**根拠（DDD 図E-1 ＋ 逆転トリック）:**
- **逆転トリック**（実装方法→カテゴリー逆算）: bootstrap で直感的に選んだ実装＝engine（load→transform→Result）＝**トランザクションスクリプト**。逆算するとカテゴリー＝**補完・一般寄り**。
- 図E-1: 「データ構造は複雑か？」— JSON/schema は構造は入れ子だが、**ロジックは検証＋変換が中心**（入り組んだ業務ルール/不変条件ではない）→ トランザクションスクリプト（〜アクティブレコード）。中核の「金額/監査」も無い。
- **アンチパターン4（補完をドメインモデルで実装＝不必要な複雑さ）／アンチパターン1（何でも集約＝貧血ドメインモデル）** に該当する危険。dm-document を「集約（commands/invariants）」と呼ぶのは、振る舞いの薄い Document を名詞だから集約にした**“何でも集約”の疑い**。
- だから **bootstrap（サービス＋データ）は DDD 的に正しく、OQ-3 の「集約を実現していない＝欠陥」は誤った期待**だった。本書「できるだけ単純なツールを・複雑な手段は最後の手段」。

**含意:** Stage A の「集約導入リファクタ」は**不要になる可能性が高い**（むしろ入れると過剰設計）。

### ユーザー見解
> Document の検証や status 遷移は Document 自身が守るもの。engine は検証のトリガーを引くだけで、スキーマレベルの検証観点や status の enum 値はその Document と Schema が持つ。だから集約は合っていると思っていた。

### AI 再考見解（初期見解を撤回）
**見解:** **ユーザーが正しい。Document 集約は正しい。** 私の初期見解（単純ロジック＝transaction-script・集約不要）は、**has-udd が不変条件を「imperative メソッド」でなく「宣言的 schema」にカプセル化している**ことを見落としていた。engine はルールの所有者でなく executor で、ルールは schema（モデル）に集約されている＝**貧血の逆**。
**根拠:**
- DDD の集約の本質＝不変条件のカプセル化＝一貫性境界。has-udd はそれを **schema で宣言的に実現**（＝harness 原則）。ルールがスクリプトに散る transaction-script とは逆で、むしろ domain-model 寄り（ルールがモデル＝schema に集約）。
- ゆえに**コードに imperative な集約クラスは不要**＝**bootstrap（engine＋schema）が正しい realize**。**OQ-3「コードに集約が無い＝欠陥」も誤り**（集約は schema 層に在る）。
- 例外: status の**遷移**規則（前進のみ）は JSON Schema で表現不能（→ Re-2）。

### 合意決定
**決定:** **Document は集約（正しい）。realize は「schema＝不変条件の宣言的所在 / engine＝executor」。コードに imperative な集約クラスは不要**。
**理由:** harness 原則＝構造/ルールは schema・engine は機械。集約の不変条件も同原則で schema に置く＝宣言的カプセル化。これで「集約正しい」「bootstrap 正しい」「OQ-3 は欠陥でない」が両立。
**次のアクション:** Re-2 で「schema が守れる不変条件（静的）」と「守れない遷移（動的）」を切り分け、後者の置き場を決める。

---

## 論点 Re-2: schema が守れない「status 遷移」をどこで守るか

### AI 初期見解
**見解:** 不変条件を**2種に切り分ける**。**(静的)** status の enum 値・必須フィールド・構造制約 → **schema が宣言的に守る**（validate で enforce・既に機能）。**(動的)** status の**遷移**規則（CREATED→VALIDATED→RENDERED→SUPERSEDED・後退/飛ばし禁止）→ **JSON Schema では表現不能**（単一スナップショット検証のため）ので、**宣言的な遷移表（dm-document の Lifecycle に既にある）＋それを読む薄い guard** で守る。imperative な集約クラスは不要、guard は最小の1点。
**根拠:**
- JSON Schema は構造/値の検証であって状態遷移を見られない＝遷移は別の所で守るしかない。
- dm-document に Lifecycle（遷移）は**仕様としては在る**が**実行時 enforce が無い**＝ここが唯一の本物の隙間。
- harness 原則に沿うなら、遷移表も**宣言的（schema/spec のデータ）**にして、guard はそれを読む決定的コードにする（ロジックを imperative に散らさない）。
- code-template には「**遷移 guard の置き場**（shared か domain か）」を1つ足すだけで足りる。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 Re-3: 動く最小サンプルの要件と「効果測定」の定義

### AI 初期見解
**見解:** 最小サンプル＝**1スタックにつき「Spec → コード → 緑テスト」の最小1スライス**を `examples/` に置き、規約を体現する。**効果測定の合格基準**を先に定義する: (1) サンプルが `.feature` 緑、(2) validationRules（依存方向・アンカー・no-print 等）を**実際に lint で通る**、(3) **同じ Spec から別の AI セッションが書いても規約に収束する**（再現性）、(4) 規約違反をわざと入れると検出される（ゲートの空振りでない）。
**根拠:**
- 「効果測定ができてから次へ」（ユーザー方針）＝測定の合格基準を先に決めないと測れない。
- サンプルは「育てる種（genesis seed）」＝最小1スライスで十分（合意済み）。
- (3)(4) は規約が“飾り”でなく実効的かを検証＝Stage B ゲートを作る前の前提確認。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 Re-4: dm-document spec の「集約」framing をどうするか

### AI 初期見解（Re-1 合意を受けて方向転換）
**見解:** **dm-document の「集約」framing は弱めない（正しい）。** 代わりに、**「不変条件は schema が宣言的に持ち、engine は executor」という realize を spec 側に明示**する。具体的には domain-model spec に「invariants の所在＝schema（宣言的）」「Lifecycle 遷移＝guard で enforce」を区別して書けるようにする。
**根拠:**
- Re-1 で「Document＝集約・realize は宣言的 schema」が確定したので、framing を弱める必要は無い（むしろ弱めると harness 原則の良さが消える）。
- 衝突の真因は「集約か否か」ではなく「**集約を imperative で realize すると思い込んだ**」こと。realize の仕方（宣言的 schema）を spec/規約に明記すれば、code-template（schema＋executor）と整合する。
- 「同じ言葉」は死守。invariants を schema に置いても語彙の一貫性は保つ。
- 注意点（要検討）: 単純ドメインで「何でも集約」に走らないガードは別途必要（DDD アンチパターン1）。「集約か否か」の判定基準を SpecSchema 側に持つか。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 Re-5: spec 階層の切り直し（A案）＋ subdomain 分類

### 確定（advisor 判断・ユーザーが DDD 分類のリードを委任）
**specKind（A案）:** `bounded-context / subdomain / aggregate / usecase`（domain-model→aggregate リネーム＋subdomain 追加）。
- bounded-context: 境界・同じ言葉・構成要素一覧
- subdomain（新）: カテゴリー（中核/一般/補完）＋所属 usecase＋実装の厚み（CodingSchema subdomainRichness と接続）
- aggregate（旧 domain-model）: 一貫性の単位・不変条件（宣言的＝schemaRef）・lifecycle・commands・**ValueObjects（中に含める）**
- usecase: 既存のまま

**ValueObjects の置き場（advisor の call）:** VO は識別子を持たない＝集約の構成要素なので**独立 specKind にしない。aggregate spec 内の `ValueObjects` ブロックに記述**する。has-udd の VO: `status`(enum)/`documentId`(ID も VO)/`schemaRef`/`tags`/`discriminator` → **agg-document**、`version`/検証ルール構造 → **agg-schema**。不変条件は宣言的に schema。複数集約が真に共有する VO が出たら bounded-context spec の「共有モデル/同じ言葉」へ上げる（今は不要）。⚠️ `status` は**値=VO（schema が守る）／遷移=集約の不変条件（guard・Re-2）**で分ける。

**has-udd の subdomain 分類（advisor の call）:**

| subdomain | 分類 | 根拠 | usecase |
|---|---|---|---|
| sd-harness-core | **中核** | 差別化＝「engine が構造を持つ／陳腐化させない UDD ループ」。複雑・進化・社内開発必須 | scaffold / query / reconcile(将来) |
| sd-validation | **一般** | 検証能力は jsonschema＝既製品。差別化しない（使われる場所≠差別化） | validate |
| sd-rendering | **補完** | 機械変換・単純・差別化しない。自前 x-render は中核の制約に従うためで競争優位でない | render / deploy(将来) |

**集約（共有 substrate）:** agg-document / agg-schema（不変条件は宣言的＝薄い）。

```
bc-has-udd-engines
├ sd-harness-core (中核) → scaffold / query / reconcile
├ sd-validation   (一般) → validate（jsonschema を @stack）
├ sd-rendering    (補完) → render / deploy
├ agg-document, agg-schema（集約・共有）
```

**含意:** ① SpecSchema/v2 で specKind 改訂（subdomain 追加・domain-model→aggregate）。② subdomain が実装の厚みを駆動＝CodingSchema subdomainRichness を spec 側へ接続。③ 中核（harness-core）に設計投資を集中、一般/補完は薄く。

### ユーザー見解
> ✏️ _（分類への反論・補足があれば）_

---

<!-- 合意後に各論点へ「AI 再考見解」「合意決定」を追記。Re-1 が全体の起点。 -->
