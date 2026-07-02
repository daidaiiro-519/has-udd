# ブレインストーミング: CodingSchema 再定義（Stage K・一から）

**目的:** SpecSchema/v2 を踏まえ、CodingSchema を**仕様／詳細設計／実装の境界**から定義し直す。肝は **codingKind（何で構成され・spec とどう関連づくか）**。
**モード:** 設計判断（論点 → 見解 → 合意）

**確定前提（揺らさない）:** Stage S=不変条件は宣言的に schema・engine は executor（imperative 集約クラス不要）／subdomain 分類=harness-core(中核)/validation(一般)/rendering(補完)／実装テンプレ非提供・機械生成は .feature だけ／`@spec`・`@stack` は DocComment・descriptor 非保存・ripgrep 逆引き／Re-2=status 遷移は薄い guard。

> スコープ: まず **CS-1（境界）→ CS-2（codingKind の構成）→ CS-3（spec との関連づけ）** を固める。動く最小サンプル・効果測定・描画見直しは**この3つが確定してから**（後続ラウンド）。

---

| # | 論点 | 状態 |
|---|---|---|
| CS-1 | 仕様／詳細設計／実装 の境界をどう引くか | 議論中 |
| CS-2 | codingKind は何で構成されるか（一から） | ✅ 合意（水準1＝概念合意。構造は CS-3＋後続） |
| CS-3 | 各 codingKind は spec とどう関連づくか | 議論中 |

---

## 論点 CS-1: 仕様／詳細設計／実装 の境界

### AI 初期見解
**見解:** **3層に明確に切る。**

| 層 | 担い手 | 中身 | 性質 |
|---|---|---|---|
| **仕様（WHAT）** | SpecSchema/v2 | bc/subdomain/aggregate/usecase・不変条件（宣言的）・TestScenarios（受け入れ） | 宣言的・技術非依存 |
| **詳細設計（HOW の“写像”）** | **CodingSchema の code-template** | 「spec/DDD 概念 → コードの実現形（配置・命名・パターン・依存）」の宣言 | **宣言的・コードそのものではない** |
| **実装（HOW の“実体”）** | authored code | code-template に従い人/AI が書くコード。`@spec`/`@stack` で紐付く | 命令的・authored |

**根拠:**
- Stage S で「不変条件=schema宣言・engine=executor」が確定＝**仕様は宣言的 WHAT に純化**、技術・コード構造を含まない。
- HOW には2種ある：**宣言的な“写像”（詳細設計）** と **命令的な“実体”（実装）**。前者を CodingSchema が担い、後者は authored＝これが「詳細設計兼実装」の内部境界。
- 機械生成は `.feature` だけ＝実装は生成しない（design-coding-schema 合意）。詳細設計（写像）は宣言、実装は人/AI。

### ユーザー見解
> 「仕様と詳細設計兼実装」の2分割で考えたい。そもそも DDD に詳細設計は要る？ AI×チームではコードが人の共通言語になれないので仕様を用意した。

### AI 再考見解（層モデル訂正）
**見解:** **DDD に「詳細設計」層は無い**（設計はコードに宿る・変換のたびに情報欠落を避ける）。私の「詳細設計＝template」は誤り→撤回。正しくは **仕様＝ドメインモデル(SpecSchema) ｜ 実装＝コード(authored)** の2分割。**CodingSchema は“規約集”**（詳細設計でなく、コードの書き方ルール）。「template」＝規約/枠の意味。
**根拠:**
- AI×チームではコードが大量・高速で AI 製＝**人の共通理解の媒体になれない → 仕様が人とAIの共通言語**（[[project-why-spec-exists]]）。古典 DDD の「コードが全て」は team/長期で弱い（意図が規律頼みで腐る）。
- モデルが仕様＋コードの2箇所＝drift 火種 → **成立条件＝TestScenarios(実行可能)＋reconcile**。
- @spec リンクの第一目的＝**探索・重複防止・関連性の可視化**（drift 検知はボーナス）。

### 合意決定
**決定:** 層＝**仕様(ドメインモデル)｜実装(コード)**。**CodingSchema＝規約**（tech-stack/code-template/test-template）。**詳細設計層は無し**。
**理由:** DDD 忠実（設計はコードに宿る）＋AI 時代の共通言語として仕様を外部化。has-udd＝「コードが全て」の弱点を仕様＋リンク＋reconcile で埋めた DDD。

---

## 論点 CS-2: codingKind は何で構成されるか（一から）

### AI 初期見解
**見解:** 「1 spec を動くコードにするのに何を宣言する必要があるか」から逆算すると **3 codingKind** に落ちる（v1 の3種は妥当・ただし code-template の中身を組み替える）:

| codingKind | 何を宣言するか | スコープ |
|---|---|---|
| **tech-stack** | 技術選択：言語 / アーキ / library / 非ドメイン能力（logging 等） | 1プロジェクト（or bc） |
| **code-template** | **規約＝「spec/DDD 概念 → コード実現形」の写像**（詳細設計でなく書き方ルール）。(a)概念→配置/命名/パターン (b)アーキ制約（依存方向・lib経由・no-print）(c)アンカー/gen-gap 規約 (d)subdomainカテゴリ→厚み | 1 stack+arch |
| **test-template** | **TestScenarios(.feature) → 実行可能** の写像（テストFW・step binding 規約） | 1 stack |

**根拠:**
- 「何のスタックで／各概念をどう実現し／テストをどう実行するか」の3つで、spec→動くコードが閉じる。
- v1 の3 kind は枠として正しい。**変える肝は code-template**：現状「コード概念キー（port/usecase/adapter…）」→ **「spec/DDD 概念キー（aggregate/usecase/VO/…）」**へ組み替える（CS-3）。
- 新 kind は不要：projection/guard 等は「code-template に従った authored コード」or「reconcile の派生物」で、宣言物ではない。

### ユーザー見解（対話で表明された要点）
- **「template」はソースひな型に見える**（実装テンプレ非提供と矛盾）→ 規約は **standard** と呼ぶべき。
- **architecture は tech-stack から分離**してよい。
- 実装で重要なのは **フロントエンド UI / バックエンド API / インフラ** の3つで、これらを**技術スタック＋規約**として表現したい。インフラは AWS のサービス分類（Compute/Security/…）が参考になりそう。
- 「**層で分けるか**」が決め手に見える。UI=プレゼンテーション層、インフラは別。DDD では普段どう進める？
- ティア名 `platform` vs `infrastructure` は、他プロダクトが clean/onion を採ると **`infrastructure層`** と語彙が被る。
- 締める水準は **水準1（概念合意）**。構造（ブロック/フィールド）は後続でよい。

### AI 再考見解（初期見解の更新）
**見解:** 初期見解（3 codingKind・code-template 中心）を、ユーザー指摘で更新する。核心は **「実装前に準備すべきもの」から逆算**すること。

1. **命名を実体化**：`template`→`standard`（ひな型でなく規約）。has-udd はコードのひな型を生成しない（`.feature` のみ機械生成）＝「template」は誤誘導。
2. **codingKind を4種に**（architecture を独立）：技術方式（＝層構成・依存方向・概念配置）は「命名・スタイル」とは別 SSOT ＝**骨格と細部の分離**。DDD 第8章の「技術方式＝層をどう組むかの原則」に対応。
3. **組織化の軸は“層”でなく“スタック（ティア）”**：DDD の型では**層は技術方式が決める**もので、層ごとに別モデル化しない。物理的に別プロセス/デプロイで動くもの（フロント／バック／platform）＝**ティア**を別スタックにする。
4. **spec 駆動には濃淡がある**（DDD の「ドメイン中心・UI/infra は外周アダプター」の帰結）：ドメインは backend だけが realize。frontend は usecase を消費、platform は能力を提供。
5. **language は借りる**：platform が提供する能力は **generic subdomain**（買う領域）＝独自語彙でなく業界標準（AWS カテゴリ）を借りる（design-heuristics「一般/補完に手をかけない」）。
6. **ティア名は `platform`**：DDD は `infrastructure` を**“層”に予約**している（データアクセス層＝adapter が住む層）。デプロイ/払い出しを `infrastructure` と呼ぶと層用語の場違い流用＝衝突。`platform` はどの層名とも衝突しない。コード層の呼称は各プロダクトのアーキ語彙に委ねる（hexagonal=adapter／clean/onion=インフラ層）。

**根拠:** [[project-why-spec-exists]]（仕様＝人の共通言語）／DDD 第8章（技術方式・レイヤ vs ティア）・第10章（generic は標準を借りる）／Stage S（宣言的 realize）。

### 合意決定（水準1＝概念合意。構造は CS-3＋後続ラウンド）

**A. codingKind の集合と責務（責務レベルで確定）**

| codingKind | 責務（何を宣言するか） | スタックでの厚み |
|---|---|---|
| **tech-stack** | 技術選択＋非ドメイン能力（＝`@stack` の宛先・capability） | 全ティア |
| **architecture** | 技術方式・レイヤ構成・**依存方向**・**DDD 概念→配置**・境界 | backend 厚／generic 薄 |
| **coding-standard** | 命名・スタイル・**`@spec` 注釈規約**・gen-gap | backend 厚／generic 薄 |
| **test-standard** | テストFW・種別・**TestScenarios→binding** | 全ティア |

＋ **examples/ の動くサンプル**（codingKind でない・併走する手本。「規約は語るより見せる」）。
**全スタックが4種を持つが、generic なティアほど薄い**（subdomain 較正と一貫）。

**B. 組織化モデル（確定）**
- プロジェクト＝**1つ以上のスタック**。**層で spec も codingKind も割らない**（層は architecture が決める）。層（論理）とティア（物理）は別軸。
- **3ティア**：**frontend**（presentation）／**backend**（application・hexagon 丸ごと＝domain＋app＋inbound/outbound adapter）／**platform**（deployment・IaC/K8s/クラウド）。

**C. spec 駆動の濃淡・継ぎ目（確定）**
- backend：`@spec`（domain 実現）＋`@stack`＝フル駆動。
- frontend：domain spec 無し・呼ぶ usecase に `@spec`（**消費リンク**）・画面設計は別成果物。
- platform：domain spec 無し・`@stack` の能力を**提供する側**。
- 継ぎ目：**`@spec`**＝backend↔domain（探索/重複防止の投影）／**`@stack`**＝backend↔platform（要求能力↔払い出し）。

**D. 語彙（確定）**
- `template`→`standard`／ティア＝`platform`（`infrastructure` は“層”に予約）／capability 語彙＝AWS カテゴリ（ベンダー中立）。

**★水準1 の限界（明示）：** 上は**概念の合意**であり、**「各 codingKind を document.json でどんなブロック/フィールドで表すか」「多スタックをどう宣言するか」「x-render の見た目」は未定**。これらは構造レイヤ＝**CS-3（概念キー化）と不可分**なので、CS-3 とその後の専用ラウンドで詰める。

**理由:** 「codingKind の構成」という問い自体には答えが出た（4種＋examples・スタック単位・濃淡・語彙）。構造は当初スコープで後続に置いており、CS-3 の概念キー化を通してから schema 骨子に落とすのが筋。

---

## 論点 CS-3: 各 codingKind は spec とどう関連づくか

### AI 初期見解
**見解:** **code-template の conventions を DDD/spec 概念でキー化**し、specKind と1:1で対応させる（＝関連が自明になる）。

| spec 側 | code-template の写像（例・python-hexagonal） |
|---|---|
| `aggregate` | 集約クラスは作らない。不変条件は schema（宣言的）＋薄いサービス。status 遷移は guard（domain/services） |
| `usecase` | application service（`application/usecases`）・公開名 engine・`@spec` |
| value-object | 不変な値型 |
| domain-service | 跨る計算（`domain/services`・ステートレス） |
| `subdomain.category` | 厚み: 中核=厚い設計 / 一般=ライブラリを adapter で / 補完=薄い TS |

- **tech-stack** ↔ bc/プロジェクト（`@stack` の宛先）。
- **test-template** ↔ usecase/aggregate の TestScenarios（`.feature`→実行）。
- リンクの実体＝**双方向**: コード→spec/stack は `@spec`/`@stack` アンカー（ripgrep 逆引き＝CQRS 投影）／spec概念→実現形は code-template の概念キー。
- **subdomainRichness は撤去**し、厚みは **subdomain spec の Category を SSOT** に参照（二重管理＝drift を排除）。

**根拠:**
- Stage S で specKind が DDD 概念に揃った＝code-template も同じ概念キーにすれば **spec ↔ code が1:1で自明**。
- 「aggregate はどう実装するの？」に spec 自身から code-template を辿って答えられる＝詳細設計の入口が spec 概念。
- subdomain spec ができた今、CodingSchema 側の subdomainRichness は重複＝SSOT を1つに。

### AI 初期見解（更新：CS-2 の4種分割を反映）
**見解:** 上の初期見解の「概念キー化」は正しいが、**担い手が code-template から architecture へ移る**（CS-2 で分離したため）。4 codingKind × spec の関連を整理し直す。

| codingKind | spec との関連 | キー／継ぎ目 |
|---|---|---|
| **tech-stack** | bounded-context スコープに対応。**domain 概念とは直接結びつかない** | `@stack`（↔platform）・capability |
| **architecture** | ★**DDD 概念キー**：`aggregate`/`usecase`/value-object/entity/domain-service → 配置・パターン。`subdomain.category` → 厚み | **specKind と 1:1**・概念キー |
| **coding-standard** | **`@spec` 注釈規約**（＝リンクの書き方）を定義。概念に依らず**横断的** | `@spec` アンカー形式 |
| **test-standard** | spec の **TestScenarios**（→`.feature`）を実行に束ねる | TestScenarios ↔ binding |

**双方向リンク:**
- **code→spec**：`@spec` アンカーを ripgrep 逆引き（＝CQRS 投影）＝**探索・重複防止・関連可視化**（第一目的）。drift 検知はボーナス。
- **spec 概念→実現形**：**architecture の概念キー**を辿る（「この aggregate/usecase はどう実装？」に spec から答えられる）。

**subdomainRichness 撤去:** 厚みは **subdomain spec の Category を SSOT** に参照（二重管理＝drift を排除）。

**責務の再配置（初期見解からの主な更新）:**
- 「概念→配置/パターン」＝**architecture**（構造の骨格）。
- 「`@spec` の書き方・命名・gen-gap」＝**coding-standard**（横断の細部・リンク機構）。
- ＝**骨格（何をどこに）と細部（どう書く・どう結ぶ）を別 codingKind に分けた**のが CS-2 の帰結。

**根拠:** Stage S で specKind が DDD 概念に揃った＝architecture を同じ概念キーにすれば spec↔code が1:1で自明／リンク第一目的＝探索・重複防止（[[project-why-spec-exists]]）／subdomain spec 成立で subdomainRichness は重複。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

<!-- CS-1〜3 合意後: 動く最小サンプル / 効果測定 / content描画 を後続ラウンドで。各論点に AI再考・合意決定を追記。 -->
