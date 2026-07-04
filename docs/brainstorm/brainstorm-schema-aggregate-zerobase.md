# ブレインストーミング: Schema集約のゼロベース再検討

**目的:** 「Document集約」「Schema集約」という既存合意（[[project-spec-schema-enrichment]]）を鵜呑みにせず、DDD本第6章（値オブジェクト・エンティティ・集約・業務サービス）に厳密に照らしてゼロから検証する。特に「kind（specKind/codingKind/skillKind）ごとに不変なブロック構造を持つ」という、実装（JSON Schemaファイル）には存在するがSchema集約のspecには一切表現されていない構造をどう扱うべきかを決める。
**モード:** 問題解決

---

## 問題の再定義

**agg-schema.json（Schema集約のspec）が、実装（SpecSchema/v2.json等）に既に存在する「kindごとの不変なブロック構造」という本質的な構造を一切モデル化していないため、仕様が実装の実態を反映しておらず、仕様→実装という順序が保てていない。**

## 原因候補

- **表面的原因**: agg-schema.jsonのValueObjects/Entitiesに"kind"という概念がそもそも無い。
- **根本原因（要検証）**: 「kindはDocumentの型(discriminator)であり集約に昇格すべきでない」という過去の合意を、「だからSchema集約のVO/Entity構造の中でkindを扱う必要もない」という**別の主張にすり替えてしまった**（非論理的な飛躍）。「kindを別集約にしない」ことと「Schema集約の内部構造としてkindを表現しない」ことは別の話のはず。

---

## 論点 1: reconcile（および関連する初期設計）は今も必要な機構か、プロジェクト初期の残骸か

### AI 初期見解
**見解:** reconcileの必要性そのものは今回のセッションで実例をもって証明された（否定材料ではなく肯定材料が出た）。ただし「何と何を照合するか」という対象の設定は当初の想定（コード↔spec）からズレて再定義が必要。

**根拠:**
- 今回のセッションで実際に「agg-document.jsonが宣言するLifecycleと、CodingSchema/SkillSchemaの実際のstatus enumが食い違っていた」というdriftを発見し、修正した。これはまさにreconcileが検出すべき種類の不整合であり、「起きた」という事実がreconcileの必要性を裏付ける。
- 一方、reconcileの当初設計（`@spec`/`@stack`アンカーをコードのDocCommentに埋め込みripgrepで逆引き）は既にroadmap.md上で「全廃」と明記されており（"supersede"注記）、これは確かに初期設計の残骸として正しく葬られている。今も生きているのは「構造照合・言語検査」という再定義後のゲート概念（Stage B）。
- 今回見つかったdriftの性質は「コード↔spec」ではなく「**spec↔spec（agg-document ↔ 各schemaファイルの実際のenum）**」または「**spec↔schema（agg-schema ↔ 各schemaファイルの実際の構造）**」という、当初のreconcile設計が想定していなかった対象。reconcileの対象を「コードとspecの差」だけに限定した理解は狭すぎ、「spec同士・spec-schema間」も含めて再定義すべき。

### ユーザー見解
> reconcileはいらない。「機械的にできるのか」「そもそもspec↔code の意味比較という当初の懸念は、has-udd自身の業務ロジックが宣言的schemaに解消されたことで無くなったのでは」という指摘の末、「reconcileという独立機構は不要」という結論。

### 合意決定
**決定:** reconcileという独立した機構・エンジンは新設しない。代わりに (1) 汎用engineの正しさは既存のBDD（.feature/behave）がそのまま担う、(2) spec↔schemaの構造整合（今回発見した種類の不整合）は、必要になれば validate engine への小さな機械チェック追加で足りる（新規の大きな機構は不要）。
**理由:** has-udd自身のドメイン（Document/Schema）はStage Sの結論により不変条件が全てschemaに宣言的に存在し、意味比較を要する業務ロジックコードが実質存在しない。当初reconcileが解決しようとした「AIが書いたコードとspecの意味的drift」という問題そのものが、has-udd自身に関しては前提から消えている。
**次のアクション:** roadmap.md / 関連タスク（ES-3: reconcile engine 帰属の決定）を「reconcile不要・クローズ」に更新する。

---

## 論点 2: Schema集約のVO/Entity構造は「kindごとに不変なブロック形状を持つ」という事実を正しくモデル化できているか

### AI 初期見解
**見解:** できていない。これは実装漏れではなく**モデリング漏れ**であり、修正すべき本物のギャップ。「kindを別集約にしない」という結論と、「Schema集約のVO/Entity構造の中でkind依存の形状を表現しない」という結論は**別の主張であり、後者は前者から論理的に導けない**。前回の私の回答はこの2つを混同していた。

**根拠（DDD本 domain-model.md 準拠）:**
- **値オブジェクトの定義**（6.2.2.1）: 「フィールド値の組み合わせによって識別する（明示的なIDフィールドは不要）・不変・変更する場合は新しいインスタンスを生成して返す」。kindごとのブロック形状（例: `specKind=usecase`なら`title/summary/actorIntent/mainFlow/postconditions/acceptanceCriteria/testScenarios`が必須）は、まさにこの定義に一致する: kind名＋必須ブロック集合という値の組み合わせで一意に決まり、変更（＝規約が変わる）は新しいSchema版（`publishVersion`）を発行することでしか起きない＝既存のインスタンスを書き換えることはない。**これは教科書的な値オブジェクトそのもの**であり、値オブジェクトとして書けるものをモデルに書かないのは、本書アンチパターン1「基本データ型への執着」と同型の欠落（このケースでは「プリミティブ」ではなく「暗黙知への執着」）。
- **集約の定義**（6.2.2.3）: 「集約は内部と外部の間に明確な境界を定義することでデータの一貫性を強制する」。Schema集約が保証すべきデータの一貫性の中核は、まさに「このkindの Documentは、このブロック集合を持たねばならない」という制約そのもの。この制約をモデルから除外すると、Schema集約は「schemaId/version/statusを持つだけの空箱」になり、**集約が実際に守っている一番重要な不変条件が仕様から消える**。
- 現状のagg-schema.jsonのEntities.Schema.attributesには既に`blocks`という属性がある（`{"name": "blocks", "type": "構造定義（各ブロックのproperties・x-render・x-prompt）"}`）が、この属性の型記述は曖昧なプローズに留まり、「kindによって内容が変わる」という事実がどこにも書かれていない。**属性の置き場所は既にあった。表現が抜けていただけ。**

**前回の私の回答の誤りの構造:** 「型を集約に昇格させるのはアンチパターン（何でも集約）」という正しい合意を、「型に関する構造をモデルに書くこと自体もアンチパターン」に拡大解釈してしまった。前者は「Skill/Spec/Agent/Codingをそれぞれ別集約にするな」という主張であり、後者（今回問題にしている話）は「唯一のSchema集約の中で、kindという概念を正しくモデル化する」という別次元の話。両立する。

### ユーザー見解
> kindごとの値オブジェクトのスキーマ定義は実際に（JSON Schemaファイルに）存在するのだから、これはspecに書かれているべき。

### 合意決定
**決定:** kindごとの不変なブロック構造を値オブジェクトとして`agg-schema.json`に明記する。
**理由:** 実装（JSON Schemaの`$defs`＋discriminator）に既に存在し、実際に本番で機能している不変条件を、spec側が反映していないのは単純な記録漏れ。「kindを別集約にしない」こととは独立した論点であり、両立する。
**次のアクション:** 論点3の実装案の通り、`agg-schema.json`にValueObject「KindProfile」を追加し実装へ反映する。

---

## 論点 3: kindごとの構造は「値オブジェクト」か「子エンティティ」か

### AI 初期見解
**見解:** **値オブジェクト**（例: `KindProfile` または `ContentShape`）としてモデル化すべきで、子エンティティではない。

**根拠（domain-model.md 準拠）:**
- **エンティティの定義**（6.2.2.2）: 「個々のインスタンスを特定するための識別情報（ID）が必要・不変ではない（状態が変化する）」。ある1つのkind（例: `usecase`）のブロック形状定義は、**そのSchemaバージョンが存在する限り独立して状態変化しない**——「usecaseのブロック形状を変えたい」となった時は、Schema全体を新しいバージョンとして`publishVersion`するのであって、既存バージョン内で`usecase`のプロファイルだけをその場で書き換えることは無い（後方互換の不変条件がそれを禁止している：agg-schema.jsonの既存Invariants「公開済みの版は遡って構造を変えない」）。つまり子エンティティの要件（独立した識別子＋独立した状態変化）を満たさない。
- 逆に値オブジェクトの判断基準（domain-model.md 381-387行「業務ロジックがそのエンティティの状態に強い一貫性を必要とするか」）に照らしても、kindのプロファイルはSchema全体と常に一体（Schemaのバージョンが変われば全プロファイルも丸ごと差し替わる）であり、独立したライフサイクルを持たない。
- 従って: `Schema`（集約ルート・エンティティ）が `kindProfiles: KindProfile[]`（値オブジェクトのリスト）という属性を持つ、という形が最も素直。`KindProfile`自体は `{kind: string, requiredBlocks: string[], blockShapes: {...}}` のような値の組み合わせ。

**具体的な反映案（合意された場合の次アクション）:**
- `agg-schema.json`のValueObjectsに`KindProfile`（またはより適切な名前）を追加: 「represents: kindごとの必須ブロック集合とその形状。behavior: 不変。kind名と必須ブロック集合の組で識別。版が変わらない限り不変。」
- `agg-schema.json`のEntities.Schema.attributesの`blocks`を、`kindProfiles: KindProfile[]`に修正（または`blocks`の型記述を「kindごとに異なる（KindProfileのリスト）」に書き直す）。
- Invariantsに「各 kind の KindProfile は固定であり、同一版の中で変更されない」を追加。

### ユーザー見解
> （論点2の決定に統合。値オブジェクトとして書くことに合意）

### 合意決定
**決定:** `KindProfile`（kind名＋必須ブロック集合の組）を値オブジェクトとしてモデル化する。子エンティティにはしない。
**理由:** 独立した識別子・独立したライフサイクルを持たず（版が変われば丸ごと差し替え）、値オブジェクトの定義（フィールド値の組み合わせで識別・不変）に一致する。
**次のアクション:**
- `agg-schema.json`のValueObjectsに`KindProfile`を追加（represents: kindごとの必須ブロック集合と形状／behavior: 不変・kind名と必須ブロック集合の組で識別・版が変わらない限り不変）
- `agg-schema.json`のEntities.Schema.attributesの`blocks`属性を`kindProfiles: KindProfile[]`に修正
- Invariantsに「各kindのKindProfileは同一版内で不変」を追加
- 実データとして、SpecSchema/CodingSchema/SkillSchemaの実際のkind一覧をKindProfileとして書き出す（例示・全網羅は任意）

---

## 論点 4: Schema集約の境界自体は正しいか（実装では複数schemaファイルに分かれている）

### ユーザー見解
> 集約自体はどうなってますか？実装ではスキーマがいくつか分かれてると思いますが？

### AI 再考見解（ゼロベース確認）
実装を確認したところ、5つのschemaファイル（当時のSpecSchema/CodingSchema/SkillSchema/RenderMetaSchema/DocstringSchema）は均質ではなく、2つの異なる性質のものが混在していた。

- **Document型定義のschema**（当時のSpecSchema・CodingSchema・SkillSchema）: 実際にDocumentの`schemaRef`から指されており、kindごとにcontentブロック形状が変わる（if/then discriminator）。
- **派生構造を検証するmeta schema**（RenderMetaSchema・DocstringSchema）: どのDocumentの`schemaRef`からも指されていない。前者はx-render部品定義の束、後者はcode_scan出力（派生データ）の検証用で、そもそもDocumentという形（documentId/status/content）を持たない。DocstringSchemaは"kind"フィールドを持つが、これはcontentブロック形状を変えず抽出ルールのみを変える、別種の"kind"。

### 合意決定
**決定:** Schema集約の境界を「Documentのschemaが指しうる型」だけに narrowing する。RenderMetaSchema・DocstringSchemaはSchema集約の対象外（別概念＝共有語彙／派生構造のバリデータ）。
**理由:** Schema集約の存在理由は`agg-document`の不変条件「schemaRefは常に存在する」を支えること＝Documentのschema参照先が常に一貫した型であることの保証。RenderMetaSchema/DocstringSchemaはこの一貫性要求の対象外（誰にも参照されない）。
**次のアクション:** `agg-schema.json`のsummary/invariantsに対象範囲を明記する。

---

## 論点 5: Spec自体の名称是正（この機会に実施）

### ユーザー見解
> この機会にもう一度再定義する。SpecSchemaという語彙のままでいいのか？以前DomainSpecSchema/PresentationSpecSchemaに分ける合意をしたはず（`project-coding-schema-stage-k`メモリで確認・実施は保留されていた）。

### 合意決定
**決定:** この機会に改名を実施する。`SpecSchema`→`DomainSpecSchema`（documentType const `Spec`→`DomainSpec`）。新規に`PresentationSpecSchema/v1`（specKind=screen/flow・documentType const `PresentationSpec`）を実装する。
**理由:** 保留の理由だった「作業量に対して実利が薄い」は、今回Schema集約の境界を確定させる作業と一体で行うことで正当化される（同じタイミングでagg-schemaのスコープ定義もやり直す必要があるため）。
**次のアクション:** ファイル改名・全12 spec instanceのschemaRef更新・PresentationSpecSchema/v1新設・scaffold/validate/renderで一次実証・agg-schema.jsonへのKindProfile反映・regression確認後コミット。

---

## セッションまとめ

**合意事項:**
1. reconcileという独立機構は新設しない（既存BDD＋必要時のvalidate拡張で足りる）。ES-3タスクはクローズ。
2. Schema集約に`KindProfile`値オブジェクトを追加し、kindごとの不変なブロック構造をspecに反映する。
3. Schema集約の境界を「Documentのschemaが指しうる型」に narrowing する（RenderMetaSchema/DocstringSchemaは対象外）。
4. `SpecSchema`→`DomainSpecSchema`に改名し、`PresentationSpecSchema/v1`（specKind=screen/flow）を新設する（保留していた命名是正を実施）。

**実施済み:**
- `SpecSchema`→`DomainSpecSchema`の全面改名（schema本体・全12 spec instance・features/tests）
- `PresentationSpecSchema/v1`新設（screen/flow・scaffold→validate→renderの一次実証済み）
- `agg-schema.json`にKindProfile VO追加・対象範囲を4schema（DomainSpecSchema/PresentationSpecSchema/CodingSchema/SkillSchema）に明記
- pytest 19/19・behave 70/70 全緑
