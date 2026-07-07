# ブレインストーミング: Agent Schema / HarnessAgent 設計

**目的:** has-uddのOrchestrator/Roleをdocument型として表現する設計を固める。特に「Agent集約を独立集約にするか、Knowledgeと同様にDocumentの一documentTypeへ統合すべきか」という既存決定との整合性を検証する。
**モード:** 問題解決

---

## 問題の再定義

`brainstorm-has-udd-design.md`で合意した「集約はdocumentTypeごとに分ける（Skill/Spec/Knowledge/Agent/Codingの5集約）」という決定が、今日Knowledgeで下した「独立集約ではなくDocument集約の一documentType」という結論と矛盾しているせいで、Agent Schemaの設計をそのまま進めると同じ矛盾を持ち込んでしまう。

## 原因候補

- 表面的原因: 「5集約」ブレストはKnowledgeSchema/AgentSchemaの中身を検討する前の初期段階の整理であり、後続のDocument/Schema集約統一（本セッションで実証済み）を知らずに書かれた
- 根本原因候補: 「documentTypeごとに集約を分ける」という直感（型が違う＝別集約）が、実際には「型の違いはdiscriminatorで表現でき、一貫性境界（集約）はDocumentという1つの実行時ライフサイクルに閉じる」という後知恵の設計原則と食い違っていた

---

## 論点 1: AgentSchemaは独立集約か、Documentの一documentTypeとして統合すべきか

### AI 初期見解
**見解:** Knowledgeと同様、Documentの一documentTypeとして統合すべき。AgentSchema用の独立集約は不要。
**根拠:**
- Agent document（Orchestrator/Role定義）も、他のDocumentと同じライフサイクル（create→validate→render→supersede、またはCoding/Skill系のDRAFT→ACTIVE→DEPRECATED）に従う限り、Document集約の一貫性境界の中に自然に収まる。差別化された不変条件（Agent固有のstatus遷移ルール等）が無い限り、別集約にする理由がない
- 「documentTypeごとに集約を分ける」という直感は、Knowledge検証で既に反証済み（実際にはquery/render/validate/scaffoldが無改修で動いた=集約が同じだから動いた）
- 集約を増やすことは境界が増えることを意味し、境界には維持コストが伴う（合成のたびにagg-agent.jsonのようなSpec文書を新設・保守する必要が生じる）。AgentSchemaがDocumentの一documentTypeで表現しきれるなら、既存のagg-document.jsonのDocumentType/DiscriminatorValueの記述を拡張するだけで済む

### ユーザー見解
> document集約でしょうねぇ。

### AI 再考見解
**見解:** 合意。AgentSchemaはDocument集約の一documentTypeとして統合する。
**根拠:**
- Knowledgeで実証した「集約は増やさずdiscriminatorで型を表現する」パターンをそのまま適用できる
- 別集約にする積極的理由（Agent固有の不変条件でDocument共通のライフサイクルでは表現できないもの）が見当たらない

### 合意決定
**決定:** AgentSchemaはDocumentの一documentTypeとして統合する（独立集約にしない）。
**理由:** Knowledgeと同じ理屈（集約を増やさない・discriminatorで型を表現）が成立し、別集約にする積極的理由が無い。
**次のアクション:** agg-document.jsonのDocumentType/DiscriminatorValue説明にAgentも反映する（実装フェーズで対応）。

---

## 論点 2: Orchestrator/Role(subagent)の1document内2レンズ（agentKind分岐）は今も妥当か

### AI 初期見解
**見解:** 妥当。既存のdiscriminator機構（specKind/codingKind/skillKind等）でagentKindも同型に表現できる。
**根拠:**
- Waffleの既存discriminator機構（`schema_discriminator.discriminator_key`・allOf if/then/else）は「1つのdocumentTypeの中で、値によってcontentの形が変わる」ケースを既に汎用的に扱えている（Coding/Skill/DomainSpec全てで実証済み）。agentKind: "orchestrator"|"subagent" もこの同じ仕組みで表現できるはずで、新しい判別の仕組みを発明する必要が無い
- 「1 Role = 1 subagent document（Role集約は作らない）」という決定も、Knowledgeと同じ発想（概念ごとに集約を増やさない）と一貫している

### ユーザー見解
> ちなみにオーケストレーターはCLAUDE.mdとかAGENTS.mdとかやと思った方がいいかもしれないですね！ロールはアドバイザーみたいなもんな気がしますよね！waffleだとロールって名前があんまりしっくりこないですね。has-uddの時はスクラムのなかと役割って感じでしたからねぇ

### AI 再考見解（誤り訂正）
**見解:** 前回の「Orchestratorはdocument化しない」は誤り。**ユーザー訂正**: Orchestratorは他のschema家族と全く同じくDocument（document.json・schemaRef検証済み）であり、CLAUDE.md/AGENTS.mdは「ツールが認識する場所」＝**render結果のdeploy先**にすぎない。これはSkillSchemaが`SKILL.md`にrenderして`.claude/skills/{id}/SKILL.md`にdeployする既存パターンと完全に同型。「1document内2レンズ（agentKind: orchestrator|subagent）」という構造自体は維持され、Orchestratorのcontentが機械的にCLAUDE.md/AGENTS.md形式へrenderされるだけ。「Role→Advisor」の改称は維持する（こちらは訂正が入っていない）。
**根拠:**
- SkillSchemaの`x-render-target`は`{"formats": ["md"], "path": ".waffle/skills/{documentId}/SKILL.md", "deploy": [".claude/skills/{documentId}/SKILL.md"]}`という構造を持つ。Orchestratorも同様に`x-render-target.deploy`に`CLAUDE.md`（Claude Code向け）・`AGENTS.md`（他ツール向け）を複数指定すれば、**1つのdocument.jsonから複数ツール向けに同じ内容を配信**できる。フォーマットはWaffle側が決める（既存のRenderMetaSchema閉語彙で表現）
- これは既存の「source=document.json（SSOT）・rendered=成果物（キャッシュ・手で維持しない）」という本セッション全体の原則とも完全に一致する。CLAUDE.mdを直接手で書き続けるのでなく、Orchestrator documentから機械的に再生成されるべきという話になる
- Phase 6「Multi-tool互換設計」も同じ理屈で解決する：deploy配列に複数ツールの起動ファイルパスを追加するだけで済み、新しい仕組みは要らない

### 合意決定
**決定:** OrchestratorはDocumentの一documentType（AgentSchemaのagentKind分岐の一方）として維持する。x-render-targetのdeployでCLAUDE.md/AGENTS.md等の各ツール起動ファイルへ配信する（SkillSchemaのSKILL.mdデプロイと同型）。「Role」は「Advisor」に改称する（この点は変更なし）。
**理由:** SkillSchemaの既存render+deployパターンをOrchestratorにもそのまま適用でき、「手書きのCLAUDE.mdが陳腐化する」問題も本セッションの原則（source=キャッシュしない・rendered=キャッシュ）で解決できる。
**次のアクション:** 論点3(Advisorのknowledge解決)・論点5(スコープ)を、Orchestratorも引き続きdocument化される前提で再確認する。

---

## 論点 3: skillRefs/knowledgeRefsは実際にどう解決されるか（Advisor前提に組み替え）

### AI 初期見解
**見解:** Waffleの既存query engine（`get_field`/`filter_items`等）で解決できる。新しい機構は不要。ただし呼び手はOrchestrator（CLAUDE.md/AGENTS.mdを読んでいるAI自身）であり、Advisor（ddd-advisor/tech-lead-advisorのような既存Skill）がknowledgeRefsで宣言した参照先を、Skill自身のSTEPの中で`waffle query`を呼んで解決する、という構図になる。
**根拠:**
- skillRefs/knowledgeRefsは「他documentのdocumentIdの配列」であり、Advisor Skillがそれを読んで対象documentへ`get_block`/`get_field`を呼べば済む。今日のKnowledge検証で確認した「既存engineがそのまま動く」という結論と同じ理屈
- knowledgeRefsが指すKnowledge documentの取得は、`query --operation get_field --blockKey <block> --field <field>`の組み合わせ、または複数document一括なら`index_scan_dir`（今回tags対応も追加済み）で十分まかなえる
- 実際、ddd-advisor Skillは既に「Step 2: 対応するknowledgeファイルを特定して必ず読む」という手順を持っており、これは将来的にknowledgeRefs（AgentSchema/AdvisorSchemaのフィールド）から機械的に導出できる可能性がある（現状は手書きの対応表）

### ユーザー見解
> 論点3はようやくクエリエンジンの日の目を浴びましたな

### AI 再考見解
**見解:** 合意。（既存query engineの`get_field`/`index_scan_dir`をAdvisorのknowledgeRefs解決にそのまま使う）

### 合意決定
**決定:** skillRefs/knowledgeRefsの解決は既存query engineで完結する。新しい機構は作らない。
**理由:** Knowledgeの検証と同じ理屈（既存engineが無改修で動く）が成立するため。
**次のアクション:** なし（実装フェーズでAdvisor Skillのstepsに反映）。

---

## 論点 4: Job(作業実行)を独立集約にしない・document.json statusをSSOTにするという既存決定は今も妥当か

### AI 初期見解
**見解:** 妥当。この決定は今日のセッションの原則（陳腐化しない・二重管理しない）とも一貫している。
**根拠:**
- Jobを独立集約にすると「実行状態」という揮発性の高い情報を別途モデル化・維持することになり、document.jsonのstatusとの二重管理・ドリフトの温床になる。今日のセッション全体で扱ってきた「reconcile対象を増やさない」方向性と整合する
- audit-engineトレース（リードモデル）に実行の可観測性を委ねる設計は、CQRS的に「書き込みの一貫性境界（集約）」と「読み取り専用の観測」を分離しており、DDD原則にも沿う

### ユーザー見解
> いらないってことかな？

### AI 再考見解
**見解:** 「Job概念自体が丸ごと不要」ではなく、「Jobのために専用の集約/document型を新設する必要は無い」という意味。実行状態はdocument.jsonのstatusが正として持ち、実行履歴・可観測性が将来必要になったときは別の読み取り専用の仕組み（audit-engine・現状未着手）に任せる、今のAgent Schema設計には影響しない範囲の話。
**根拠:**
- 「Job」という語彙・概念は消えない（作業実行を指す言葉として引き続き使う）が、それをDDDの意味での「集約」として独立させる必要が無いだけ
- audit-engineは今回のAgent Schemaのスコープ外（別途必要になったときに設計する）

### 合意決定
**決定:** Job用の独立集約/document型は作らない。実行状態はdocument.jsonのstatusで表現し続ける。
**理由:** 二重管理・ドリフトの温床を増やさないため。audit-engineは別スコープとして保留。
**次のアクション:** なし（Agent Schema設計には影響しない）。

---

## 論点 5: これはWaffle自身のスコープか、has-udd本体のスコープか（論点2の訂正を踏まえ再確認）

### AI 初期見解
**見解:** AgentSchema（Orchestrator/Advisor共通の型）自体はWaffleのdomain/modelに置く（Knowledgeと同じ理屈）。ただしOrchestrator documentのx-render-target.deployは「どのリポジトリ・どのツール向けか」に応じて可変になるため、**deploy先パスの決定はhas-udd運用側（各リポジトリ）の関心事**であり、Waffle自身はdeploy機構（render engineのdeploy処理）を提供するだけでよい。
**根拠:**
- 既存のSkillSchemaも同じ切り分けになっている：SkillSchema（型）はWaffle資産・deploy処理（render engineのdeployロジック）もWaffle資産だが、「どのdocumentIdをどこにdeployするか」というx-source-target/x-render-targetの具体的なパス値はdocument.json側（利用者が書く値）が決める
- Orchestrator documentの実体（1つのAgent document・agentKind=orchestrator）は、has-udd本体（repo root）に1つ、Waffle（waffle/配下）に1つ、という具合に**リポジトリごとに複数存在しうる**（今まさにwaffle/CLAUDE.mdとrepo root CLAUDE.mdが両方存在するのと同じ構図）。これはWaffle/has-uddという「どちらか一方」の話ではなく、「型はWaffle資産・インスタンスは各リポジトリに複数」という通常のDocument運用そのもの

### ユーザー見解
> 悩ましい。ツールの都合上常にルートが参照されるので、has-uddがwaffleを使うときにwaffleのルートにあるCLAUDE.mdやAGENTS.mdは死ぬんだよなぁ

（続けて、ルート参照の解決策として「参照フィールド」案を出したところ）
> 参照フィールドというかサブオーケストレーター的な感じだよなぁ

### AI 再考見解
**見解:** Orchestratorはリポジトリ内で1つとは限らず、**階層構造（ルートOrchestrator＋各サブディレクトリのSubOrchestrator）**を持つ。ツールは常にルートのCLAUDE.md/AGENTS.mdしか自動発見しないため、ルートOrchestratorが「作業対象がwaffle/配下だと判断したら、waffle/CLAUDE.mdというSubOrchestratorへ処理を委譲する」という、**Advisorへの委譲と同型の委譲構造**をOrchestrator間でも持つ。実際、has-udd root CLAUDE.mdに既にある「waffle/で作業する場合は必ずwaffle/CLAUDE.mdを先に読むこと」という一文は、この委譲を手書きproseで実現している先行実装。
**根拠:**
- 「参照フィールドを1つ持たせる」という単純な発想だと、ルートOrchestratorが単に子の存在を知っているだけ（静的なリスト）になるが、ユーザーの言う「サブオーケストレーター」はそれ以上の意味を持つ——**子もまた同じagentKind=orchestratorの完全なOrchestratorであり、ルートはその子へ実行委譲する**という、Advisor委譲パターンの入れ子版
- これは既存のsubdomain/bounded-context階層（bc-waffle-engines配下にsubdomainがネストする構造）とも似た形であり、Orchestrator自体も「区切られた文脈（bounded context）ごとに1つ存在し、親子関係を持つ」というモデルとして自然
- 現状のhas-udd root CLAUDE.mdの手書き委譲prose（「waffle/で作業する場合は...」）は、この階層構造が正式にモデル化されれば、render engineが機械的に生成できるようになる（scopeRef/childOrchestratorRefsのようなフィールドから委譲節を自動生成）

### 合意決定
**決定:** Orchestrator documentは`scope`（自身が管轄するディレクトリ。ルートなら`"."`、waffle用なら`"waffle/"`）と`subOrchestratorRefs`（子Orchestrator documentへの参照配列）を持つ。ルートOrchestratorをCLAUDE.md/AGENTS.mdへrenderする際、`subOrchestratorRefs`を機械的に辿って「〜で作業する場合は先に〜を読め」という委譲節を自動生成する。SubOrchestratorは特別な型でなく、scopeの値が違うだけの普通のOrchestrator document。
**理由:** ツールがルートのCLAUDE.md/AGENTS.mdしか自動発見しないという制約は無視できない。既存のhas-udd root CLAUDE.mdの手書き委譲prose（「waffle/で作業する場合は必ずwaffle/CLAUDE.mdを先に読むこと」）はこのパターンの先行実装であり、これをdocument.jsonから機械生成できるようにすることは、本セッション全体の原則（source=document.json・rendered=キャッシュ・手で維持しない）とも一致する。DomainSpecSchemaのbounded-context/subdomainネスト構造（contextRef）と同型のパターンを流用でき、新しい仕組みを発明する必要が無い。
**次のアクション:** AgentSchema実装フェーズで、Orchestratorのcontent構造に`scope`・`subOrchestratorRefs`フィールドを追加し、render時の委譲節自動生成ロジックを設計する。

---

## 論点 6: AgentSchemaはOrchestratorだけでよいか、Advisor（旧Role）も含むか

**背景:** 論点2でRole→Advisorに改称したが、Advisor自体を新しいAgentSchemaでモデル化すべきか、既存SkillSchema(skillKind=custom)のままでよいかが未確定だった。

### ユーザー見解
> アドバイザーはサブエージェントではない？

（Claude Codeの実際の仕組みを`claude-code-guide`で調査した結果を提示）
Claude Codeには **Skill**（`.claude/skills/{name}/SKILL.md`・同一コンテキストへの指示注入・分離なし）と
**カスタムSubagent**（`.claude/agents/{name}.md`・独立コンテキスト・tools/model/permissionMode等の
本格的なfrontmatter）という完全に別の2つの仕組みが存在することが判明。has-udd元々の設計
「Role=subagent」はSubagent（独立コンテキスト）を指していたが、今セッションで実際に使った
ddd-advisor/tech-lead-advisorはSkill（コンテキスト分離なし）として呼ばれていた。

### AI 見解
**見解:** Advisorは**Skillのまま**とし、Subagent化しない。AgentSchemaはOrchestrator専用に絞る。
**根拠:**
- 今セッション全体で実際に起きていたのは、ddd-advisorをSkillとして呼び、その場で知識を読み・推論し・
  ユーザーに答え・ユーザーが反論すれば即座に再考する、という**継続的な対話**だった（例: 論点1〜5の
  やり取り自体がこのパターン）。Subagentだと独立コンテキストで処理し要約だけ返す形になり、この
  行き来の速さ・透明性が失われる
- has-udd元々の「Role=subagent」という設計は、マルチエージェントScrum運営（PO/SM/Dev/QAが並行して
  自律的に動く）という別のユースケースを前提にしていた。それぞれの職種が互いにコンテキストを
  汚染し合わないことに価値がある場面であり、「会話の途中で専門家のレンズを借りる」という
  Waffleでの実際の使われ方とは前提が異なる
- 「Role」という名前がWaffleにしっくりこなかったのと同じ理由（has-uddのScrum文脈に紐づいた設計が
  Waffleの実態と合わない）が、実装の仕組み（subagent前提）自体にも当てはまっていた可能性が高い

### 合意決定
**決定:** AdvisorはSkill（既存SkillSchema・skillKind=custom）のままとし、Subagent化しない。AgentSchemaは
Orchestrator専用として設計する（agentKind分岐を持たない・KnowledgeSchemaと同様に単一の形）。
Advisorのknowledge参照は、既存の`ReferencesBlock`（`{path, description}`）で表現できるため
SkillSchema自体の変更も不要（ddd-advisor.json等が現状これを活用せずSteps内に手書きproseで書いて
いるのはコンテンツ側の改善余地であり、スキーマの不足ではない）。
**理由:** 今セッションを通じて観察された実際の使われ方（継続対話・同一コンテキスト）に基づく判断。
将来「対話の継続が不要・一発判定で十分」という使われ方が増えればSubagent化を再検討する。
**次のアクション:** AgentSchema/v1.jsonをOrchestrator専用（scope・subOrchestratorRefs・
x-render-targetのdeployでCLAUDE.md/AGENTS.mdへ配信）として実装する。

---

## ★後日談: Subagent（`agentKind: subagent`）を追加

論点6合意直後、ユーザーから重要な指摘:
> waffleでサブエージェントを作成できる機構を設けなくても大丈夫？

一度は「今の実際の使われ方（Skillとしての継続対話）に基づく判断」として見送りを提案したが、
ユーザーから明確な訂正:
> 前にも言ったけど、waffleで提供すべき要素は必要なんです。今使用されてないとかは関係ないです。
> なぜならこれはOSSですから

**この指摘は正しい。** MigrationEngineのYAGNI教訓（実際に使われた実績が無い機能は作らない）を
誤って適用していた。MigrationEngineは「has-udd/Waffle自身が今後も使わないと判断した機能」だったが、
Subagent作成機構は「Waffle自身が今使っていなくても、OSSとして他の利用者が必要とする一般的な
Claude Code機能」であり、性質が異なる。「今使っていない」を理由に機構自体を設けないのは、
OSSツールとしての完全性を欠く判断だった。

**実装した内容:**
- AgentSchemaに`agentKind: "orchestrator" | "subagent"`のif/then/else判別を追加
- Subagent側content: Description（委譲判断）・ToolAccess（tools/disallowedTools・任意）・
  RuntimeConfig（model/permissionMode・任意）・SkillPreloads（プリロードSkill・任意）・
  SystemPrompt（本文プロンプト）
- render engineをdiscriminatorごとのx-render-target（path/deploy/pathVars）・x-frontmatter
  出し分けに対応拡張（Orchestrator=CLAUDE.md/AGENTS.mdへdeploy・frontmatterなし、
  Subagent=`.claude/agents/{documentId}.md`へdeploy・frontmatterあり）
- 任意ブロック省略時・値が空のときはfrontmatterフィールドごと省略する規約を追加
  （`tools: []`=ツール禁止、フィールド省略=継承、という意味の違いを保つため）

**教訓:** YAGNI判断は「誰にとって不要か」を明確にしてから適用する。自分たち（開発者）が
使わないことと、OSSの利用者一般が必要とする標準機能を提供しないことは全く別の問題。

---
