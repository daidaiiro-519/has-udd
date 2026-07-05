# ブレインストーミング: spec由来の.featureを実際にbehave実行へ束縛する

**目的:** `uc-*.json`のTestScenariosから生成される`.feature`が、これまで`.waffle/specs/`配下に出力されるだけで**一度もbehaveに実行されていなかった**という発覚済みの欠陥を解消する。SP-4/SP-5（「シナリオは1回だけ定義し、ステップバインディングで観点を切り替える」）を実際に機能させる。
**モード:** 問題解決

---

## 問題の再定義

**`uc-scaffold-document.json`のTestScenariosと`waffle/features/scaffold.feature`（手書き）が、同じ振る舞いを別々の文言で二重管理しており、生成された方は誰にも実行されないため、ドリフトしても誰も気づけない。**

## 原因候補

- **表面的原因**: render engineの出力先が`.waffle/specs/`（behaveのデフォルト探索範囲外）になっている。
- **根本原因**: SP-4/SP-5の「シナリオ1回・ステップバインディングで観点切り替え」を実装する際、既存の手書き`.feature`をそのまま残し、生成`.feature`を実行系に接続する作業（「.feature→behave実行束縛」）が過去のセッションでバックログのまま放置されていた（`project-resume-point`メモリに記録済み）。

## 技術的に確認した前提

- `behave`（パス指定無し）は`features/`配下のネストしたサブフォルダの`.feature`も**自動的に再帰探索する**（実証済み）。よってrender先を`waffle/features/`配下に変えれば、追加設定なしで実行対象になる。
- ステップバインディングは`features/steps/`（または`features/<subdir>/steps/`）で読み込まれる規約。

---

## 論点 1: 生成先フォルダをどう構成するか（テストパターン別）

### AI 初期見解
**見解:** `waffle/features/generated/{acceptance,unit,e2e}/`という、SP-5のテストレベルに対応した3分類のサブフォルダに生成する。

**根拠:**
- SP-5は「Unit（aggregate.UnitTestScenarios）/ Acceptance（usecase.TestScenarios）/ E2E（同シナリオ×UI/APIステップバインディング）」の3レベルを既に定義済み。フォルダ名をこれに一致させれば、生成物の置き場所とテスト戦略の語彙が一致する。
- `acceptance/`には`uc-*.json`のTestScenarios由来、`unit/`には`agg-*.json`のUnitTestScenarios由来を生成する。`e2e/`は当面空（UI操作のステップバインディングを持つ対象がまだ無いため・将来PresentationSpecSchemaのComponentTestScenarios実行等で使う）。

### ユーザー見解
> 論点1は考えられるテストパターンは全て実現できるようにしましょう。generatedはいらないのでは？　→（さらに）コンポーネントテストはないの？

### AI 再考見解（ユーザー指摘「本当にこんなにカバーできるの？あくまでシナリオを元にできる範囲では」を受けて訂正）
**見解:** 「全パターンのフォルダを予約する」は過剰だった。**Gherkinシナリオとして自然に導出できるものだけ**をフォルダ化する。

SP-5の9軸を、シナリオ由来かどうかで再分類する:

| レベル | シナリオ由来か | 扱い |
|---|---|---|
| unit | ○ agg-*.UnitTestScenarios | フォルダを持つ |
| component | ○ scr-*.ComponentTestScenarios | フォルダを持つ |
| acceptance | ○ uc-*.TestScenarios | フォルダを持つ |
| e2e | △ 新規シナリオではなく**acceptanceと同じシナリオを別のステップバインディングで実行するだけ** | 別フォルダ・別コンテンツは不要 |
| uat | △ 同上（acceptanceの再利用） | 別フォルダ・別コンテンツは不要 |
| smoke | △ 新規シナリオではなく**既存シナリオのタグによる部分選択** | 別フォルダ不要（タグ機構の話） |
| regression | △ 新規シナリオではなく**既存シナリオの累積再実行** | 別フォルダ不要 |
| contract | ✗ API/メッセージ形状の一致検証で、Given/When/Thenの通常シナリオでは表現しにくい。今のspecスキーマに対応フィールドが無い | 見送り |
| performance | ✗ タイミング閾値等はspecの現行フィールドに無い | 見送り |
| security | ✗ 大半は既存の異常系シナリオ（事前条件違反・権限エラー等）で実質カバー済み。専用フォルダは不要 | 見送り |

**決定:** フォルダは`unit/`・`component/`・`acceptance/`の3つだけを用意する。

**根拠:**
- 「全パターン実現できるように」という私の解釈が行き過ぎだった。**具体的な生成元（spec側のどのブロックから作るか）が無いフォルダを予約することは、使われない空の受け皿を作るだけ**——feedback-library-selection（反射的に足さない）・YAGNIの原則に反する。
- e2e/uat/smoke/regressionは「同じシナリオの再利用（ステップバインディング差し替え・タグ選択・累積実行）」という**実行方法の違い**であり、新しいGherkinコンテンツの生成元では無い。フォルダを分ける意味が無い。
- contract/performance/securityは、今のspecスキーマにこれらを表現するフィールドが無い以上、シナリオとして自然に導出できない。無理にフォルダを用意しても中身を生成する当てが無い。

---

## 論点 2: 手書きの`waffle/features/*.feature`（scaffold.feature等）はどうするか

### AI 初期見解
**見解:** **廃止し、spec由来の生成`.feature`に一本化する。** ステップバインディング（`steps/*.py`）だけを手で書き、Gherkinの文言はspec（TestScenarios）を唯一の情報源にする。

**根拠:**
- SP-4/SP-5の原則そのもの:「シナリオは1回だけ定義し、ステップバインディングで観点を切り替える」。今の二重管理状態はこの原則への違反であり、今回発覚した本物のドリフトリスク。
- ただし文言が一致していないと既存のステップバインディングは動かない。**移行には、既存のステップバインディングを、spec側のGherkin文言に合わせて書き直す作業が要る**（逆に、spec側の文言を今のステップバインディングに合わせて書き換える、という順序もあるが、正本はspec側であるべきなのでステップバインディング側を合わせるのが筋）。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 3: render_engineのfeaturePath先の変更方法

### AI 初期見解
**見解:** `x-render-target.featurePath`のテンプレートを`.waffle/specs/...`から`waffle/features/generated/acceptance/...`（usecaseの場合）・`waffle/features/generated/unit/...`（aggregateの場合）に変更する。specKindごとの辞書という既存の仕組み（今回のcontextRef対応）をそのまま使い、`featurePath`の値だけを変える。

**根拠:** 既存の`x-source-target`/`x-render-target`のspecKindごとの辞書パターンをそのまま流用できるため、新しい仕組みは不要。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---
