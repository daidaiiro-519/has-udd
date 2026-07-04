# ブレインストーミング: PresentationSpecSchemaの中身をゼロから設計する

**目的:** ブレスト無しで実装してしまった`PresentationSpecSchema/v1`（specKind=screen/flow）の中身を、ゼロベースで再検討する。DDD本の範囲外（本は画面設計の方法論を持たない）であることを踏まえ、「コードから導けないauthoredな意図」という spec の存在理由に立ち返って設計する。
**モード:** 問題解決

---

## 問題の再定義

**PresentationSpecSchemaの中身（specKind=screen/flow・Fields/Actions/VisualRef/Steps）が、確立された方法論にも既存合意にも基づかず、一言のメモリのヒントから私が即興で作った構造であり、正しさの検証を経ていない。**

## 原因候補

- 「この機会にPresentationSpecSchemaも実装する」という許可を得た直後に、既存のDomainSpecSchemaの調子（Title/Summary/各種ブロック）をそのまま踏襲して埋めてしまい、**中身の設計そのものが決定を要する論点だという認識が抜けていた**。

---

## 論点 1: screen と flow を分けるべきか（flowは本当に独立した"著述すべき意図"を持つか）

### AI 再考見解（実装を見直して気づいた問題点）
**見解:** 現状の`flow`（`Steps: [{fromScreen, toScreen, trigger, condition}]`）は、**各screenの遷移情報の単なる集計**でしかなく、独立したDocumentとして著述する価値を疑う。

**根拠:**
- has-uddの一貫した原則（SP-2「メンバー・所属文脈は保存しない→フォルダ由来」・query engineの`_index`動的計算）は、「**導出可能なものは保存しない**」。もし各screenが自分の遷移先を持てば、"flow"は複数screenの遷移をグラフとして辿るだけで再構成でき、わざわざ別のDocumentとして著述する理由がない。
- 現に、私が設計した`ActionsBlock`（screenの操作）には「この操作が別の画面に遷移する」という情報が無い（`usecaseRef`はあるが遷移先screenが無い）。つまり今の設計では**遷移情報がActionsにもFlowにも中途半端に散らばっている**。
- 逆に、flowを独立させる価値があるとしたら、それは「複数画面をまたぐ一連の体験に、個々の遷移の総和以上の意味（名前・目的・成功/失敗の基準）がある」場合（例:「チェックアウトフロー」という一つのまとまりとして語る価値）。今のflow設計はその価値を何も持っていない（ただの遷移表）。

**提案（2案）:**
- **(a) flowを廃止**し、screenのActions自体に「遷移先screen」を持たせる（Actionを「何をするか」と「次にどこへ行くか」の両方を表現できるようにする）。複数画面をまたぐ体験の可視化は、query engineが複数screenのActionsを辿って動的に組み立てるビュー（保存しない）とする。
- **(b) flowを残すが、中身をグラフ(Steps)でなく物語(名前・目的・成功条件)にする**。個々の遷移そのものはscreen側に持たせ、flowは「なぜこの一連の画面が1つの体験としてまとまっているか」という、screen単体からは読み取れない意図だけを持つ。

私の意見は(a)寄りです——今のuc-scan-source-codeやagg-documentで貫いてきた「保存しない・導出する」の原則に最も忠実だからです。

### ユーザー見解
> （その後の対話でSP-5のテスト戦略との接続を指摘。E2Eレベルは「同じシナリオ×UIステップバインディング」で既存usecaseのTestScenariosを再利用すればよく、flow固有に新しく持たせる価値は「複数usecaseを貫くE2E合成シナリオ」であるべき、という方向で収束）

### 合意決定
**決定:** (b)寄りに収束。ただし物語だけでなく、**flowの中身を「業務ユースケースの並び（UsecaseSequence）」という具体的な構造にする**——画面間の生の遷移グラフ（Steps）は廃止し、`Actions.leadsTo`（screen側）に一本化。flowは「この体験を構成する業務ユースケースの時系列」を持ち、E2Eテストはこの並びで各usecaseのTestScenariosをUIステップバインディングで繋いで実行する。
**理由:** SP-5「シナリオは1回だけ定義し、ステップバインディングで観点を切り替える」の原則をそのまま踏襲。flowの独立価値は「複数usecaseを貫くジャーニーとしての意味づけ」にあり、個々の画面遷移の集計ではない。
**次のアクション:** `ActionsBlock`に`leadsTo`（遷移先screenのdocumentId）を追加、`FlowContent`を`UsecaseSequenceBlock`（業務ユースケースの並び・sequence図で描画）に置き換える。

---

## 論点 2: screenの中身は「コードから導けない意図」を過不足なく捉えているか

### AI 再考見解
現在の`Fields`（項目）・`Actions`（操作）・`VisualRef`（外部参照）を、spec の存在理由（「コードから導けないauthoredな意図」）に照らして再検証する。

- **Fields**（項目の名前・種別・必須・説明）: 情報アーキテクチャそのもので、著述する価値がある（コードのフォーム実装からは"なぜこの項目が要るか"は読み取れない）。妥当。
- **Actions**（操作・契機・説明・対応usecase）: UIの操作を業務ロジック（usecase）に紐付ける役割は、DDD的にも意味がある（UIが独自の業務ルールを持たず、意図を業務層に委譲していることの裏付けになる）。ただし論点1の通り、遷移先情報が抜けている。
- **VisualRef**（Figma等への参照）: 既存合意（ビジュアルは外部参照のみ）に忠実。妥当。

**気づいた欠落:** どの画面がどの`usecase`の入り口・出口なのかは`Actions.usecaseRef`から辿れるが、**その画面が持つ「業務上の目的（何を達成するための画面か）」を1文で語る場所が無い**。DomainSpecSchemaのUsecaseはActorIntentを持つのに、Screenには対応するものが無い。

### ユーザー見解
> （AI時代のフロントエンド仕様管理についての議論で収束。usecaseの入力パラメータから導出できる項目はspecに書かず都度クエリで引く前提とし、Summaryが「業務上の目的」を担う形に整理）

### 合意決定
**決定:** `Fields`は**usecaseの入力パラメータから導出できない項目だけ**に限定する（例: パスワード確認欄・利用規約同意チェック）。usecaseの入力に対応する項目はspecに書かず、都度usecase側から引く前提とする。「画面が存在する理由」は`Summary`が担い、usecaseの内容の繰り返しでなくプロダクトとしての意図を書く場所として明記する。加えて`ComponentTestScenarios`（画面固有の振る舞いの受け入れ契約・aggregateのUnitTestScenariosと同格）を新設する。
**理由:** 「詳細設計=クエリ結果（導出物）／spec=導出不可能な意図」の原則をフロントエンドにも一貫適用。usecaseパラメータの複製はドリフトの温床になる。
**次のアクション:** `FieldsBlock`のスコープを限定する文言に修正、`ComponentTestScenariosBlock`（TestScenariosBlockと同形）を追加。

---

## 論点 3: そもそもscreen/flowという単位が正しいか（代替の切り方）

### AI 再考見解
**見解:** 「screen（画面）」という単位自体は妥当（UIの最小の著述単位として業界標準的にも自然）。ただし論点1の結論次第で「flow」は無くなる可能性がある。他に検討すべき単位（コンポーネント単位・状態単位等）は今のところ過剰設計と考える——現時点でUIを持つ具体的な対象（Waffle自身にはUIが無い）が無く、抽象的に単位を増やしすぎるのは実利が薄い。

### 合意決定
**決定:** screenは単位として維持。flowは論点1の通り「画面遷移の集計」から「業務ユースケースの旅」に再定義した上で維持。
**理由:** screen/flowの2 kind構成自体は妥当で、中身の再設計だけで課題は解消された。
**次のアクション:** なし（実装済み）。

---

## セッションまとめ

**合意事項:**
1. flowを画面遷移グラフから「業務ユースケースの並び（UsecaseSequence）」に再定義。画面遷移そのものは`Actions.leadsTo`に一本化。
2. `Fields`をusecase入力から導出不可能な項目のみに限定。`ComponentTestScenarios`を新設（画面固有の振る舞いの受け入れ契約）。
3. screen/flowの2 kind構成自体は維持。

**実施済み:**
- `PresentationSpecSchema/v1.json`を上記方針で全面改訂（ScreenContent: Title/Summary/Fields/Actions/ComponentTestScenarios/VisualRef、FlowContent: Title/Summary/UsecaseSequence）
- 用語を「usecase」→「業務ユースケース」に統一
- `UsecaseSequence`の描画をstatediagram→sequenceDiagramに変更（usecaseは「留まる状態」でなく「起きる出来事」のため）
- Mermaid記法の網羅調査（`.claude/skills/mermaid-guide/references/`全17パターンファイルを検証・修正）を経て、RenderMetaSchemaに`participantsFrom`/`pseudoStatesFrom`を追加。`part_renderer.py`の`_sequence`/`_statediagram`を拡張（actor宣言・loop/alt入れ子・activate/deactivate・疑似状態）
- サンプルデータで実際にscaffold→validate→render→mermaid.parse()検証まで実施。pytest 24/24・behave 70/70 全緑
