# ブレインストーミング: ux-advisorの設計

**目的:** advisorエコシステム（Phase 3）の一環として、新設する`ux-advisor`（画面設計・体験設計担当）が
何を持つべきか、DDD・アーキテクチャ両方の観点から発散させ、守備範囲と原典知識の方針を決める。
**モード:** アイデア発散

---

## 前提: ddd-advisor・tech-lead-advisorへの事前相談

このブレストの前に、確立済みの両advisorへ実際に相談し、以下の回答を得た（詳細は本セッションの
やり取りを参照。要旨のみ記載）。

**ddd-advisorから:**
- UIは独立したサブドメインにならない。`subdomain.md`の原則（サブドメイン＝事業活動の単位）に
  UIは該当せず、各境界づけられたコンテキスト内の「プレゼンテーション層」という横断的な実装関心事
  として扱うのが原則に沿う。
- 「画面仕様と実装の境界」の具体的な引き方はDDDバックボーンの範囲外。レイヤー間の依存方向という
  一般原則までしか言えない。
- 既存メモリの「PresentationSpecSchemaは非DDDのSpec家族として別家族化すべき」という判断は妥当。
  DDDの戦略的設計（サブドメイン分類・境界づけられたコンテキスト）は事業活動の複雑さの区切り方を
  扱うものであり、「画面に何を表示するか」はこの枠組みが対象とする複雑さの種類ではないため。

**tech-lead-advisorから:**
- `architecture-layer-boundary.md`が明言する通り、DDD自体がプレゼンテーション層を積極的に
  モデル化しないため「必要な場合は別途明示的な設計判断が要る」——これがux-advisorの存在意義。
- 「画面の外部契約」対「画面の内部実装（コンポーネント構造・状態管理）」の分離は、
  `architecture-port-adapter.md`のPort/Adapter原則を類推適用すれば筋は通るが、
  バックボーンが直接答えを持つわけではない（プレゼンテーション層内部の分割はスコープ外）。
- tech-lead-advisorとux-advisorの境界線案：tech-lead-advisorは「プレゼンテーション層が
  アプリケーション層・ドメイン層に対して正しい依存方向を保っているか」というマクロな層配置を、
  ux-advisorは「プレゼンテーション層の内部（コンポーネント構造・状態管理・デザインシステム）」を
  担う、という切り分け。
- CodingSchemaの4区分（tech-stack/architecture/coding-standard/test-standard）をux-advisorに
  類推適用する提案は可能だが、これはtech-lead-advisorのバックボーン外の類推にすぎず、
  「デザイントークン／デザインシステムそのものへの理解」という4区分のどれにも自然に収まらない
  独自軸が要りそうだ、という所見も添えられた。

---

## アイデアダンプ

1. ux-advisorの原典知識を、単一の権威ある書籍ではなく複数の確立された公開知見（Nielsen Norman
   Groupのユーザビリティ原則、Material Design/Human Interface Guidelines等のデザインシステム、
   Atomic Designのコンポーネント分類）の総合として構築する（論点8の結論をそのまま適用）
2. 「画面仕様」を独立したSpec家族（PresentationSpecSchema）として正式に切り出し、ux-advisorは
   このSpecの記述を導く役割を持つ
3. tech-lead-advisorとux-advisorの境界を「外側との依存方向（tech-lead-advisor）」対
   「レイヤー内部の設計（ux-advisor）」で明確に線引きする
4. 「再現性のあるUI/UX提案」を実現するため、判断基準を決定木（DecisionCriteria）として
   KnowledgeSchemaに構造化する（tech-lead-advisorと同じ型）
5. デザインシステムの「トークン層（色・余白・タイポグラフィ等の原子的な決定）」を、
   CodingSchemaの4区分に収まらない独自の第5の観点として新設する
6. アクセシビリティ（WCAG等）を、UXの中核原則として最初から組み込む
9. 「画面の外部契約」をユースケース記述（UsecaseSpec）と接続し、業務エキスパートが自然言語で
   書いたユースケースから画面契約を導出する経路を用意する
10. ux-advisorの判断基準に「認知負荷」という軸を持たせ、tech-lead-advisorの
    `architecture-evidence-based-scope`（YAGNI）と対になる「機能追加は認知負荷とのトレードオフで
    判断する」という原則を立てる
11. コンポーネント設計パターン（Atomic Design・Container/Presentational分離等）を、
    tech-lead-advisorのport-adapterと同型の「判断基準」として構造化する
12. フロントエンドの状態管理パターン（ローカル状態／グローバル状態／サーバー状態の分離）を
    ux-advisorとtech-lead-advisorのどちらが持つべきか、境界を明確に決める

**絞り込み候補（実現性・インパクトで上位4つ・論点として深掘り）:**
- ux-advisorのDDD/アーキテクチャ上の位置づけ（PresentationSpecSchemaとの関係含む）→ 論点1
- tech-lead-advisorとux-advisorの境界線の正式合意 → 論点2
- ux-advisorの原典知識の調達方針（論点8の適用） → 論点3
- ux-advisorの「設計観点」の分解（CodingSchema類推が妥当か、独自軸が要るか） → 論点4

---

## 論点 1: ux-advisorはDDD/アーキテクチャの中でどう位置づけられ、PresentationSpecSchemaとどう接続するか？

### AI 初期見解

**見解:** ux-advisorはDDDのサブドメイン分類・境界づけられたコンテキストという枠組みの「外側」に
位置する新しい種類のadvisorとして位置づける。PresentationSpecSchemaは非DDDのSpec家族として
正式に新設し、ux-advisorはその記述内容（画面の外部契約）を導く役割を持つ。

**根拠:**
- ddd-advisorへの事前相談で「UIは独立したサブドメインにならない」「画面設計はDDDが対象とする
  複雑さの種類ではない」という回答を得ており、これはWaffleの既存の家族分け原則
  （DomainSpecSchema＝DDD3層、今後のPlatformSpecSchema＝非DDDの第3家族）と同型のパターンである。
  UIも同様に非DDDの独立したSpec家族として扱うのが一貫する。
- 既存メモリ「spec は DDD より広い上位概念」（`project-coding-schema-stage-k`）が既に
  「spec＝コードから導けないauthoredな意図のSSOT・DDDはその一家族」という位置づけを確立しており、
  画面設計の意図（何を見せ、何を受け付けるか）もこの「authoredな意図」に該当する。
- ただし画面は必ず何らかの業務領域（サブドメイン）に紐づくため、PresentationSpecSchemaは
  DomainSpecSchemaのaggregate/usecaseへの参照点を持つべきである（完全に無関係ではない）。

### ユーザー見解
> 私は古典DDDに一石を投じてしまうかもしれませんが、エンドユーザーに一番近いインターフェースである
> ユーザーインターフェースは立派な事業価値と業務価値に直結する。DDDにおいてこの部分は決して
> 切り離せないと思っている（DDDを踏襲したUDDだからこその発想）。業務エキスパートが最も知見を
> 持っているものはドメインそのものではなく、そのドメインを使った業務ユースケースであり、
> 価値はそこに生まれる。どれだけモデリングが素晴らしくても提供価値がおざなりでは事業領域として
> 無価値。presentation層をDDDの範囲外として捉えるのはいささか投げやり。
>
> ただしアーキテクチャの依存方向性は絶対に崩したくない。UIに複雑な業務ロジックを持たせるべきでは
> なく、presentationとしてのロジックに限る。presentationが内側（ユースケース）に依存するのは
> 問題ないが、中心が外側のpresentationに依存することは絶対にしない。presentationがユースケースを
> 参照するときはportを介して繋ぎに行く——DIされる領域だと考えている。

### AI 再考見解
**見解:** 「UIは独立したサブドメインにならない」という結論自体は維持するが、これは重要度の低さを
意味しないと明確化する。サブドメイン分類は「自社で作り込むべきか」という投資判断の道具であり、
UIの事業価値を測る道具ではない——道具のミスマッチであって価値の否定ではない。この区別を踏まえ、
PresentationSpecSchemaは非DDDのSpec家族のままだが「周辺的な付属品」ではなく
「UsecaseSpecと対になる第一級のSpec」として位置づけ直す。ux-advisorの役割も「画面の外部契約を
導く」だけでなく「業務ユースケースの価値をエンドユーザーに届ける設計を主導する」に格上げする。

**根拠:**
- `domain-expert.md`の原則（業務エキスパートの役割は「業務活動がどうあるべきか」を知っていること）
  は、抽象的なドメインモデルではなく具体的な業務ユースケースの積み重ねとして読める。ユーザーの
  主張はバックボーンと矛盾せず、むしろ自然な延長線上にある。
- has-uddの既存設計（UsecaseSpec/DomainModelSpecのみでSpec構成・論点5で確定した表で業務エキスパートの
  入口をUsecaseSpecに置く）は、既にこの発想を採用済みであり、新しい逸脱ではなく一貫した帰結。
- ただし`architecture-dependency-direction.md`の依存方向原則（プレゼンテーション層はビジネスロジック
  層に依存してよいが逆は不可）は絶対に譲らない。UIの重要性を認めることと、層の分離を捨てることは
  両立する——presentationはPrimary/Drivingアダプターとしてユースケースのポートを（DIで）呼び出す
  一方向のみであり、業務ロジックをpresentation側に持たせることはしない。

### 合意決定
**決定:** PresentationSpecSchemaは非DDDのSpec家族として新設するが、「UsecaseSpecと対になる
第一級のSpec」と位置づける。ux-advisorの役割は「画面の外部契約を導く」に留まらず、
「業務ユースケースの価値をエンドユーザーに届ける設計を主導する」まで含む。依存方向は
`architecture-dependency-direction.md`/`architecture-port-adapter.md`の原則に厳密に従い、
presentationは常にユースケースのポートをDIで呼び出す側であり、業務ロジックを内包しない。
**理由:** サブドメイン分類は投資判断の道具であり事業価値の重要度を測る道具ではない、という区別に
より「非DDDのSpec家族＝軽視」という誤読を避けつつ、UIが事業価値・業務価値に直結するという
ユーザーの指摘を正当に取り込める。同時に依存方向の原則を堅持することで、UIの重要性を認めることと
アーキテクチャの健全性を両立させる。
**次のアクション:** PresentationSpecSchemaのcontent設計（画面の外部契約をどう表現するか、
UsecaseSpecとの参照関係）は、ux-advisorのバックボーン設計（論点3・4）が固まった後に着手する。

---

## 実施記録: 論点1の議論から知識を3件補強（2026-07-09）

論点1の議論を通じて、両advisorのバックボーンに以下を追記した。

- **ddd-advisor `subdomain.md`**（直接Edit・KnowledgeSchema未移行のため）: 「サブドメイン分類は
  投資判断の道具であり、事業価値の重要度を測る道具ではない」という明確化を原則に追記。
- **tech-lead-advisor `architecture-port-adapter.md`**（source JSON→validate→render→手動コピー）:
  「プレゼンテーション層はPrimary/Drivingアダプターとして扱い、ユースケースのポートをDIで呼び出す。
  presentation自身がユースケースの実装や業務ルールを内包しない」を原則に追記。
- **tech-lead-advisor `architecture-dependency-direction.md`**（同上）: アンチパターンに
  「UI（プレゼンテーション）層に業務ルールの判定ロジックを持たせる」を追加。

**ユーザーからの追加決定**: ddd-advisorの19knowledgeファイルも、tech-lead-advisorと同様
document.json（KnowledgeSchema/v1）で管理すべき、との方針が示された。既存バックログ
「ddd-advisorのKnowledgeSchemaへのレトロフィット」の優先度を引き上げる（着手時期は本ブレスト
完了後に別途調整）。

---

## 論点 2: tech-lead-advisorとux-advisorの守備範囲の境界線をどこに引くか？

### AI 初期見解

**見解:** tech-lead-advisorが「レイヤー配置（プレゼンテーション層がアプリケーション層・
ドメイン層に対して正しい依存方向を保っているか）」を、ux-advisorが「レイヤー内部の設計
（コンポーネント構造・状態管理・デザインシステム）」を担う、という境界線で合意する。

**根拠:**
- tech-lead-advisorへの事前相談でこの切り分け案自体が既に提示されており、
  `architecture-layer-boundary.md`の「プレゼンテーション層はDDD自体がモデル化しないため
  別途明示的な設計判断が要る」という記述と整合する（tech-lead-advisorは層の"外側との関係"は
  語れるが"内側の設計"は持たない、という素直な役割分担）。
- 論点10（`brainstorm-platform-engineering-application`）で既に確立した「advisor同士は
  互いを名指しせず、情報はテキストベースの値として出所を問わず受け取る」という疎結合原則を
  そのまま踏襲できる。ux-advisorはtech-lead-advisorの判定結果（レイヤー配置が正しいか）を
  前提として受け取るだけでよく、tech-lead-advisor側の実装詳細を知る必要がない。
- 状態管理パターン（ローカル/グローバル/サーバー状態の分離）のような、DDDでいう「アプリケーション層
  とドメイン層の境界」に類似したフロントエンド固有の判断は、この境界線に従えばux-advisorが担う
  （tech-lead-advisorのバックボーンにフロントエンド固有の知識は無いため）。

### ユーザー見解
> 悪くないと思うね。

### AI 再考見解
**見解:** AI初期見解を維持しつつ、論点1の合意（presentationはportを介してユースケースに
一方向依存・DIされる領域）を踏まえて境界線をより具体的に確定する。「presentationがユースケースの
portを正しい依存方向で呼んでいるか」がtech-lead-advisorの判定軸、「そのport呼び出しの内側で
画面がどう構成されるか（コンポーネント構造・状態管理・デザインシステム）」がux-advisorの判定軸。

**根拠:**
- 両advisorが同じ対象（例：レビュー対象のコンポーネント）を見ても判断軸が重ならない——
  依存方向の適合は`architecture-dependency-direction.md`の決定木で、コンポーネント内部の設計は
  ux-advisor独自の決定木で、それぞれ独立して判定できる。
- 論点1の合意（presentation層の依存方向厳守）と論点2の境界線が同じ原則（依存方向の適合はtech-lead-advisor、
  内部構成はux-advisor）で一貫しており、二つの論点が矛盾なく組み合わさる。

### 合意決定
**決定:** tech-lead-advisorは「presentationがユースケースのportを正しい依存方向で呼んでいるか」を、
ux-advisorは「そのport呼び出しの内側で画面がどう構成されるか（コンポーネント構造・状態管理・
デザインシステム）」を担う、という境界線で確定する。
**理由:** DDDバックボーンがプレゼンテーション層の内部をモデル化しないという既存の制約と、
論点1で確定した依存方向の原則（presentationは常にユースケースのportをDIで呼ぶ一方向）の両方と
整合し、両advisorの判断軸が重複せず独立して機能する。
**次のアクション:** ux-advisorのSKILL.md設計時、Step1に「tech-lead-advisorの依存方向判定結果を
入力として受け取る（名指しせず、値として）」という手順を、論点10で確立した疎結合原則に沿って組み込む。

---

## 論点 3: ux-advisorの原典知識（backbone）は何を土台に構築すべきか？

### AI 初期見解

**見解:** 単一の権威ある書籍を調達するのではなく、tech-lead-advisorと同じアプローチ
（複数の確立された公開知見の交差点をAIが総合し、単一の権威ある出典として断定的に語らない）で構築する。

**根拠:**
- 論点8（`brainstorm-platform-engineering-application`）で既に「DDDだけが根幹であり書籍ベースの
  厳密さが要る。他の借用思想は公開知見の総合で足りる」という合意決定に至っており、UX/フロントエンド
  デザインはDDDと同じ「根幹」の地位を持たない（Waffleの中核業務ロジックではなく、実行層の関心事）
  ため、この合意がそのまま適用できる。
- ユーザーが求める「再現性のあるUI/UX提案」「デザインシステムへの理解」は、特定の一冊の書籍より、
  実務で広く採用されている複数の確立された知見（Nielsen Norman Groupのユーザビリティ原則、
  Material Design/Human Interface Guidelinesのようなデザインシステムの考え方、Atomic Designの
  コンポーネント分類論）を土台にした方が、実際の判断基準としての網羅性が高い。
- tech-lead-advisorが実際にこのアプローチで構築され、効果測定（Phase1）で独立エージェント同士の
  収束を確認できたという実績があり、再現性のある判断基準を作れることは既に実証済みの型である。

### ユーザー見解
> 方針はいい。ただしどの情報源を土台にするかが重要（この論点の本題は「何をknowledgeとして
> 持たせるか」）。信頼できるソースとしてClaude Code公式の`/frontend-design`skillがある。
> 全部が効果的とは限らないので、抽出して独自解釈のknowledgeにした方がいい。

### AI 再考見解
**見解:** 情報源の候補を「Anthropic公式`/frontend-design`（視覚的判断・紋切り型回避・制作プロセスの
規律）」「Nielsen Norman Group（ユーザビリティ原則）」「Material Design/Human Interface
Guidelines（デザインシステムの考え方）」「Atomic Design（コンポーネント分類論）」の4系統に確定する。
ただしddd-advisor/tech-lead-advisorと同じ規律で、これらを丸ごと転記せず、判断基準（決定木）として
抽出・独自解釈でknowledge化する。全カテゴリを先回りして作り込まず、実相談で優先度を確認してから
段階的に作る（Thinnest Viable Platform）。

**根拠:**
- ユーザーの指摘通り「全部が効果的とは限らない」——`/frontend-design`は美的判断・プロセス寄り、
  他3系統は使いやすさ・部品体系寄りで、性格が異なる。丸ごと採用ではなく判断の分岐点として使える
  部分だけ抽出するのが、ddd-advisor刷新時に確立した「原典の丸写しでなく判断基準として再構成する」
  規律と一致する。
- `/frontend-design`自体が既に「コードを書く前に設計プランを作る」というUDDの精神と同型のプロセスを
  持っており、この部分（計画→批評→実装→批評のサイクル）は特に抽出価値が高い。
- 4系統すべてを一度に知識化するのは論点6で確立した「机上で4本作ろうとしてユーザー指摘で
  優先順位確認に方針転換した」教訓（`architecture-evidence-based-scope`）に反する。
  最初のknowledge文書は、tech-lead-advisor同様、実際の相談を経てから決める。

### 合意決定
**決定:** ux-advisorの原典知識は、Anthropic公式`/frontend-design`・Nielsen Norman Group・
Material Design/HIG・Atomic Designの4系統を情報源候補としつつ、丸ごと転記せず判断基準として
抽出・独自解釈でknowledge化する。最初から4系統・全領域をフル装備せず、実際の相談を通じて
優先度を確認してから段階的にknowledgeを増やす。
**理由:** 情報源ごとに性格（美的判断／使いやすさ／デザインシステム／部品体系）が異なり丸ごと
採用は非効果的というユーザーの指摘は妥当であり、ddd-advisor刷新時に確立した「原典を判断基準として
再構成する」規律・tech-lead-advisorのThinnest Viable Platform運用実績とも一貫する。
**次のアクション:** ux-advisorのSKILL.md初版を作る際、最初のknowledge文書1〜2本をこの4系統から
選んで試作し、効果測定（Phase1と同型の動く最小サンプル）で検証してから本格展開する。

---

## 論点 4: ux-advisorの「設計観点」をどう分解するか？CodingSchemaの4区分は流用できるか？

### AI 初期見解

**見解:** CodingSchemaの4区分（tech-stack/architecture/coding-standard/test-standard）を
そのまま流用せず、UI固有の「デザイントークン／デザインシステム」を独自の軸として追加した
構成にする。

**根拠:**
- tech-lead-advisorへの事前相談で「4区分はバックエンド指向のCodingSchemaの類推にすぎず、
  デザイントークン／デザインシステムそのものへの理解という、どの区分にも自然に収まらない独自軸が
  要る」という所見が既に得られている。
- ユーザーが列挙した期待（再現性のあるUI/UX提案・デザインシステムへの理解・フロントエンドデザイン
  知識・画面仕様と実装の境界）を素直に区分にすると、「デザインシステム/トークン」
  「コンポーネント設計（画面契約と内部実装の分離）」「UI実装規約」「UIテスト方針」
  「フロントエンド技術選定」の5区分程度になり、CodingSchemaの4区分と1対1には対応しない。
- ただし区分を増やしすぎるとThinnest Viable Platform原則（tech-lead-advisorの
  `architecture-evidence-based-scope`）に反するため、最初から5区分をフル装備するのではなく、
  実際の相談（tech-lead-advisor構築時と同様、優先順位を確認してから作る）を経て段階的に
  knowledgeを増やす進め方にすべきである。

### ユーザー見解
> 最近は`design.md`というものがあるらしい。これはデザイントークンをまとめたもの。

### AI 再考見解
**見解:** 「デザインシステム/トークン」区分の成果物形式として`design.md`（カラーパレット・
タイポグラフィの役割・レイアウトコンセプト・シグネチャー要素をまとめたもの）を正式に採用する。
CodingSchemaのtech-stack.jsonが技術選定の記録フォーマットを定めているのと同じ位置づけで、
ux-advisorは`design.md`相当のフォーマットをデザイントークンの記録フォーマットとして持つ。

**根拠:**
- `/frontend-design`の設計プラン成果物（色・タイポグラフィ・レイアウト・シグネチャー要素）が
  実質的に`design.md`の内容と一致しており、既に公式skillが実証済みのフォーマットを流用できる。
- CodingSchemaが「規約集」（詳細設計でなく判断基準の記録）という位置づけなのと同様、
  `design.md`もコードそのものではなく「決定の記録」として機能させれば、Waffleの既存の
  spec/coding設計原則（意図はauthoredなSSOTとして残す）と一貫する。

### 合意決定
**決定:** ux-advisorの「設計観点」は5区分（デザインシステム/トークン・コンポーネント設計・
UI実装規約・UIテスト方針・フロントエンド技術選定）を候補とし、最初から全区分を作り込まず
段階的に増やす。「デザインシステム/トークン」区分の成果物形式には`design.md`
（`/frontend-design`の設計プラン相当：カラーパレット・タイポグラフィの役割・レイアウトコンセプト・
シグネチャー要素）を採用する。
**理由:** CodingSchemaの4区分をそのまま流用せず独自軸を追加すべきというtech-lead-advisorの所見と、
`design.md`という既に実証済みのフォーマットを使えるという実務上の裏付けが揃った。
Thinnest Viable Platform原則により全区分の先回り実装は避ける。
**次のアクション:** ux-advisorの最初のknowledge文書候補として「デザインシステム/トークン
（`design.md`フォーマット）」を優先的に試作する。

---

## 論点 5: advisorが書いたものを、自身・他advisorに批評させるプロセスを導入すべきか？どこに置くか？

### AI 初期見解

**見解:** 批評プロセスは導入すべきだが、置き場所はOrchestrator（waffle.json）のoperatingRulesでは
なく、**各advisor自身のSKILL.md内のプロセスステップ**として実装する。

**根拠:**
- `/frontend-design`が既にplan→critique→build→critiqueという自己批評サイクルをプロセスとして
  持っており、これをux-advisorのSKILL.mdのStepとして直接踏襲できる（新しい発明ではなく既存の
  実証済みプロセスの移植）。
- 「単一の判断だけで確定させず複数視点で検証する」という設計思想は既にhas-udd/Waffle側で確立済み
  である（Workflowの「Adversarial verify」パターン＝N人の独立した懐疑者に反証させる、
  brainstormスキル自体の「AI初期見解→ユーザー見解→AI再考見解」という往復構造も同種）。
- ただしOrchestrator document（`waffle/.waffle/documents/agent/waffle.json`）は、直近の意思決定
  （「ルートオーケストレーターは要らない。今はwaffleだけを考える」）によりwaffle scopeに限定されて
  いる。advisor全般（repo全体で使われる）に関わるこの批評ルールをwaffle.jsonのoperatingRulesに
  書くのはスコープ不一致であり、root Orchestrator documentが無い現状では、各advisor自身の
  SKILL.mdに持たせるのが唯一の実装可能な場所である。

**提案する具体的な仕組み:**
- **自己批評**: 同じadvisorが自分の初期出力を、`/frontend-design`の批評フェーズと同じ形で
  再検証するStepを追加する。
- **他advisor批評**: 論点2で確立した境界線（tech-lead-advisor＝依存方向の適合、ux-advisor＝
  レイヤー内部の設計、ddd-advisor＝ドメイン整合性）にそのまま従い、出力の性質に応じて関連する
  advisorへ批評を依頼する。論点10の疎結合原則（テキストベース・出所を問わない）を守り、
  批評を依頼する側は相手を名指しで結合させない。

### ユーザー見解
> これはwaffleスコープだからむしろ適切では？advisorはその都度呼ばれて批評を依頼されるのが
> 好ましいのでは？

### AI 再考見解

**見解:** 両方の指摘を取り入れて見解を修正する。(1) 置き場所はwaffle.jsonのoperatingRulesが
正しい——advisor（ddd-advisor/tech-lead-advisor/ux-advisor）自体がWaffleが所有・出荷する成果物
（source documentは全て`waffle/.waffle/documents/`配下）である以上、「advisorをどう使うか」という
利用ルールはWaffle自身のOrchestratorの管轄であり、root Orchestratorの不在とは無関係にwaffle scope
で正しい。(2) 批評の仕組みは各advisorのSKILL.md内部に埋め込むのではなく、**呼び出す側
（Orchestrator＝Claude Code自身）がその都度、批評を明示的に依頼する**という運用ルールにする。

**根拠:**
- 置き場所の訂正: 私は当初「advisorは repo全体で使われるからroot scope」と考えたが、これは
  「誰が使うか」と「誰が所有するか」を混同していた。既存メモリの「engine認識/Role/knowledge
  アーキテクチャ」原則（Orchestrator＝engine routingを担う唯一の存在・Role/Skillはengineを
  呼ばない）をadvisorに適用すると、複数のadvisorを組み合わせて使う判断（批評を依頼するかどうか・
  誰に依頼するか）はOrchestrator側の役割であり、個々のadvisorのSKILL.md内部に持たせるべきではない
  ——これは私の当初案（各SKILL.mdに批評Stepを埋め込む）が実は違反していた原則だった。
- 機構の訂正: 「advisorが他のadvisorを呼ぶ」という設計は、論点10で確立した疎結合原則
  （advisor同士は互いを名指しせず、情報はテキストベースの値として受け取る）にも実は緊張関係が
  あった——ある advisor の SKILL.md が「別のadvisorに批評を依頼する」という手順を内包すること自体、
  結合度を上げてしまう。呼び出す側（Orchestrator）が都度「このadvisorの出力を、別のadvisorに
  批評してもらう」と明示的に判断・実行する方が、疎結合原則と一貫する。

### 合意決定

**決定:** advisorの出力に対する批評プロセスは、`waffle.json`のoperatingRulesとして追加する
（advisor自体がWaffleの所有物であるため、waffle scopeが正しい置き場所）。仕組みは各advisorの
SKILL.md内部に埋め込まず、**Orchestrator（呼び出す側）がadvisorを使って何かを書かせた後、
その都度、自己批評（同じadvisorに批評フェーズとして再検証させる）または他advisor批評（論点2の
境界線に従い関連advisorに依頼する）を明示的に実行する**、という運用ルールにする。
**理由:** advisor（ddd-advisor/tech-lead-advisor/ux-advisor）はWaffleが所有・出荷する成果物であり、
その利用ルールはWaffle自身のOrchestratorの管轄として正しい。また「advisorが他のadvisorを呼ぶ」
のではなく「Orchestratorがadvisorの組み合わせを判断する」という役割分担は、既存の
engine認識/Role/knowledgeアーキテクチャ原則（Orchestratorのみがengine/advisorの組合せを判断する）
と、論点10の疎結合原則の両方と一貫する。
**次のアクション:** `waffle.json`のoperatingRulesに、批評プロセスの運用ルール（1項目）を
`scaffold fill`で追加し、validate/render/deployする。文言は「advisorを使って何かを書かせた後は、
自己批評または関連する他advisorへの批評依頼を経てから確定する」という趣旨で、具体的なタイミング・
判断基準は既存ルールと同じ粒度（検証可能な具体的指示）で書く。

---
