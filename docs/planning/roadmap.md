# has-udd ロードマップ（進捗トラッカー）

このファイルが「**今どこにいるか**」の SSOT。Phase ごとの進捗と、根拠となるブレスト doc を紐づける。
詳細な手順は各 `docs/brainstorm/*` と `docs/planning/{implementation-plan,sprint-plan,spec-id-map}.md` を参照。

## 凡例

| 記号 | 意味 |
|---|---|
| ✅ | 設計＋実装 完了（テスト緑） |
| 🟡 | 設計完了・実装は部分 or 未 |
| 🟠 | 設計途中（ブレスト段階・実装なし） |
| ⚪ | 未着手 |

**現在地（2026-07-10）:** コアループ（document を 作る→検証→render→配置）と engine/CLI/MCP/Spec/BDD は**実装完了**（✅）。HOW品質を守る層（規約強制・reconcile・投影・Hooks）と OKF/マルチツール拡張は**設計止まり**（🟠）。**Stage D の一部（Agent/custom Skill/Orchestrator）は advisor エコシステムとして実装済み**（詳細＝`advisor-ecosystem-roadmap.md`）。テスト: waffle単体 pytest 188件 green。
**インフラ変更:** Waffle・LoomDB を `git subtree split` で独立 GitHub リポジトリへ分離済み（[waffle](https://github.com/daidaiiro-519/waffle)・[loomdb](https://github.com/daidaiiro-519/loomdb)、いずれも public）。has-udd 側の `waffle/`・`loomdb/` はそのまま通常のサブディレクトリとして残置（submodule化はしていない）。両リポジトリには本ロードマップのスナップショットを `docs/planning/` 配下に配置済み。

---

## Stage A — ブートストラップ：コアループ ✅ 完了

> 詳細＝`docs/planning/implementation-plan.md`（P0-P7）。Phase 1-4 完了＝bootstrap マイルストーン。

| Phase | 成果物 | 状態 | 根拠ブレスト |
|---|---|---|---|
| P0 | パッケージ骨格・shared(Result/tags)・ports・outbound | ✅ | `design-implementation-architecture` |
| P1 | validate engine | ✅ | `design-engine-set` |
| P2 | render engine → SKILL.md/HTML（一次実証） | ✅ | `design-engine-render`・`design-render-primitives`・`design-schema-and-engine-skills` |
| P3 | query engine ＋ 動的 _index（16操作） | ✅ | `design-engine-query` |
| P4 | scaffold engine（create/fill）＝コアループ完成 | ✅ | `design-engine-scaffold` |
| P5 | CLI（typer）＋ MCP（fastmcp）front-door | ✅ | `design-engine-query`（2モード） |
| P6 | SpecSchema/v1（bc/dm/uc・TestScenarios→.feature） | ✅ | `design-spec-schema` |
| P7 | dogfood：自己記述 Specツリー7本 | ✅(主) | — |

**確定済みの土台:** 宣言的 x-render（RenderMetaSchema 閉語彙＋sequence）／deploy=render内蔵copy／Schema は `domain/model/` 配下／uv+pytest+behave。

> ⚠️ **「✅完了」の意味＝bootstrap が動く・テスト緑。最終ゲートに対して凍結/適合済みではない。**
> **Stage A は Stage B で変わり得る（設計上の前提＝「移行＝validation/reconcile を on にするだけ」）。** 移行レディ仕込みは実在（コードに `@spec`×8・`@stack`×11・gen-gap×69）。Stage B のゲート点灯時、最初に当たるのが Stage A（dogfood）なので具体的に:
> - **reconcile 点灯 → orphan 検出**: 既に `part_renderer.py @spec:uc-render-parts` は spec 不在＝orphan（spec を書く or アンカー修正で Stage A 変動）。
> - **アンカー形式確定**（DocComment 規約: `@spec:` コロン記法→docstring/Javadoc 形式）で全アンカー書き換え。
> - **CodingSchema 規約の強制点灯**で既存コードの逸脱が surface し得る。
> - **OQ-3 は再フレーム済（`brainstorm-coding-schema-redefine` Re-1 で合意）**: Document は集約で正しく、has-udd は不変条件を **schema に宣言的にカプセル化**する（engine は executor）。よって**コードに imperative な集約クラスは不要＝集約導入リファクタは不要**・bootstrap は正しい realize。残る隙間は **status の“遷移”規則のみ**（JSON Schema 表現不能→薄い guard・Re-2）。整合ゲートは「不変条件が schema に在るか／engine が schema を迂回していないか／遷移 guard が在るか」を見る形に縮小。

---

## Stage S — SpecSchema/v2 再構成 ✅ 完了

> `brainstorm-coding-schema-redefine`(Re-1〜5) ＋ `design-spec-schema-v2`。has-udd 全10 Spec を v2 移行済み・v1 撤去・全緑（pytest15/behave66）。commit 5db48cc。

| 項目 | 状態 |
|---|---|
| specKind 階層 `bounded-context / subdomain / aggregate / usecase`＋discriminator | ✅ 実装 |
| 集約=宣言的不変条件(schema)＋status遷移(guard宣言)・VO は aggregate 内・Entities(isId) | ✅ |
| subdomain 分類（harness-core=中核 / validation=一般 / rendering=補完） | ✅ |
| render=MD 正本（HTML 撤去）＋描画整備（区切り/状態図/B4a/コマンド・イベント整形/守り方） | ✅ |
| has-udd 全 Spec 移行（bc / sd×3 / agg×2 / uc×4）・v1 撤去・.feature(UDDループ)維持 | ✅ |

---

## Stage K — CodingSchema 確定（code-template＋サンプル＋効果測定）🟠 ブレスト（★Stage B の前提）

> Stage S を先に完走したため**未着手**。reconcile/ゲート（Stage B）は「**conformant とは何か＝CodingSchema 規約＋動くサンプル**」が確定して初めて作れる。順序＝K→B。

| 項目 | 状態 | メモ |
|---|---|---|
| code-template 規約の確定（Re-1/Re-2/Re-4 決定） | ✅ 決定＋実装済 | `brainstorm-coding-schema-redefine`（Re-2/3/4 CLOSED）。宣言的realize=imperative集約クラス不要／status遷移=`domain/services/lifecycle_guard.py`が各schemaのx-lifecycle宣言を読む薄いguard（validate engineで判定のみ・副作用なし）。副産物: agg-documentのLifecycleがSpec系(CREATED→VALIDATED→RENDERED→SUPERSEDED)とCoding/Skill系(DRAFT→ACTIVE→DEPRECATED)の2系統に分かれていた drift を発見・agg-document.json修正＋SpecSchema/v2にmaturityLifecycleブロック追加で解消 |
| 動く最小サンプル（examples/・Spec→コード→緑テスト1本） | ⚪ | Re-3で合格基準4点確定（.feature緑/lint通過/別AI収束/違反検出）・実装未着手 |
| 効果測定（規約が意図した品質を生むか） | ⚪ | 合格基準は Re-3 で確定済み・実施は examples/ 完成後 |
| CodingSchema の content 構造・描画（Spec 同様の見直し） | ⚪ | 根拠=`design-coding-schema`・`brainstorm-coding-schema` |

> 決定済み（Stage S 副産物）: 宣言的realize・subdomain分類。未決＝上記。ブレスト=`brainstorm-coding-schema`（本ステージで実施）。

---

## Stage B — 3ゲートのループ（＝has-udd の核心）🟠 設計止まり（Stage K の後）

> **★2026-07 再定義（`brainstorm-ai-era-detail-design`・憲法級）**: Stage B＝「spec は嘘をつかない」を機械保証する **3ゲートのループ**。①シナリオ実行（BDD 緑・実装済✅）②構造照合（正典配置・依存方向・不変条件の所在）③言語検査（コード語彙↔spec 語彙）。**これが無ければ spec はただの文書**＝最優先。
> **supersede**: @spec/@stack アンカーは全廃（リンク＝正典配置＋計算）・`sim-code-spec-link-projection` の「@spec ripgrep 投影」結論は置換（投影の機械化は code_scan に継承）・OQ-1/OQ-7（アンカー系）は消滅・ES-3 は解決（code_scan=query engine 拡張・PoC 済）。

| 項目 | 状態 | 根拠ブレスト / メモ |
|---|---|---|
| CodingSchema 規約（Stage K で再定義中） | 🟠 architecture/tech-stack/coding-standard 確定・test-standard 残 | `brainstorm-coding-schema` |
| ゲート2: 構造照合（reconcile＝その場で再計算して比較） | 🟠 設計 | `brainstorm-ai-era-detail-design`（D-1/D-3） |
| ゲート3: 言語検査（新規） | ⚪ | 同上（帰結5） |
| code_scan（DocComment 動的インデックス・ES-3 の解） | 🟡 **PoC 検証済**・正式実装残 | 同上（D-4） |
| 投影（コード→API/DB/インフラ文書の機械生成） | ⚪ 新規機構 | 同上（D-2） |
| Hooks（ゲートの発火点） | 🟠 H-1〜7 | `design-hooks` |
| 保守ループ | 🟠 ML-1〜6 | `design-maintenance-loop` |

**残 OQ（再編後）:** OQ-2 spec無しコードの規約 / OQ-3 ドメインモデル整合ゲート（→ゲート2に統合） / OQ-4 不変条件 unit test / OQ-5 重複防止の強制（→意図カタログ検索＋ゲート） / OQ-6 2グラフ混同。**Stage B 設計の最初の一手＝保守シミュレーション実験**（spec 変更→AI 保守→対応計算の精度実測・D-1 合意）。

---

## Stage C — 知識 & OKF 🟠 ブレスト/PoC

| 項目 | 状態 | 根拠ブレスト / メモ |
|---|---|---|
| knowledge engine（2軸・knowledgeRefs） | 🟠 設計のみ・未実装 | `design-engine-knowledge` |
| OKF 適用 戦略 | ✅(合意) | `brainstorm-okf-has-udd`（CLOSED） |
| OKF render 設計（バンドル/frontmatter/cross-link） | 🟠 RO-1〜5 途中 | `design-engine-render-okf` |
| OKF frontmatter relations（tags vs relations 等） | 🟠 論点1合意/2-4途中 | `brainstorm-okf-frontmatter-relations` |
| graph viewer（自前・Cytoscape+marked+mermaid+CSS） | 🟡 PoC 動作確認済 | `docs/design/okf-prototype.html` |
| #31 frontmatter OKF整合 / #32 render okf / #33 viewer | ⚪ 本実装未 | 上記群 |

---

## Stage D — Orchestrator・エージェント・配布 🟡 一部実装済み（advisorエコシステムとして進行中）

> ★2026-07 更新: 当初想定していた「Agent(Role) Schema」「custom Skill Schema」「HarnessAgent
> （Orchestrator）」は、**advisorエコシステム**（`ddd-advisor`/`tech-lead-advisor`/`ux-advisor`/
> `platform-advisor`）という具体的な実装対象を得て前進した。詳細な進捗管理は
> `docs/planning/advisor-ecosystem-roadmap.md` に分離（Phase 1〜5構成、Phase 1・2・3完了）。

| 項目 | 状態 | 根拠ブレスト / メモ |
|---|---|---|
| Agent(Role) Schema（`AgentSchema/v1`・agentKind=orchestrator） | ✅ 実装 | `.has-udd/documents/agent/waffle.json` が実インスタンス。`OperatingRules`/`SubOrchestratorRefs`/`SkillFollowUp`ブロックまで実装・`CLAUDE.md`/`AGENTS.md`へrender |
| custom Skill Schema（`SkillSchema/v1`・skillKind=custom） | ✅ 実装 | ddd-advisor(knowledge19)/tech-lead-advisor(knowledge11)/ux-advisor(knowledge5)/platform-advisor(knowledge7)の4advisorが準拠。`docs/planning/advisor-ecosystem-roadmap.md` Phase 1〜3 |
| HarnessAgent（Orchestrator・engine routing） | ✅ 実装（Waffle自身のOrchestrator） | `waffle.json`がWaffle自身のCLAUDE.md/AGENTS.mdをrenderする実例として機能 |
| knowledge文書のKnowledgeSchema化 | ✅ 完了 | ddd-advisor 19件・tech-lead-advisor 11件・ux-advisor 5件・platform-advisor 7件、全てKnowledgeSchema/v1準拠のdocument.jsonへ統一。物理配置は`waffle/.waffle/skills/`・`waffle/.waffle/knowledge/`に一本化し、`.claude/skills/`側はsymlink化 |
| FeedbackReport | ⚪ | — |
| Multi-tool 互換（Skills/Hooks/Agents/rules） | 🟠 ブレスト | `brainstorm-multi-tool-compatibility` |
| Conformance Scorecard実装（Stage B本体） | ⚪ | `advisor-ecosystem-roadmap.md` Phase 4 |
| OKF/カタログ実装（graph viewer本実装） | ⚪ | `advisor-ecosystem-roadmap.md` Phase 5・Stage C参照 |

---

## 横断：未解決論点レジストリ

| ID | 論点 | 置き場 |
|---|---|---|
| OQ-1〜7 | code↔spec 投影の検討漏れ | `sim-code-spec-link-projection` |
| ES-3 | reconcile engine の帰属 | ✅ CLOSED（2026-07）: reconcileという独立engineは新設しない。`brainstorm-schema-aggregate-zerobase`（論点1）で結論——has-udd自身は不変条件が全てschemaに宣言的に存在し意味比較を要する業務ロジックコードが無いため、当初の「AI生成コード↔spec drift」という前提が消滅。汎用engineの正しさは既存BDD（.feature/behave）が担い、spec↔schemaの構造整合は必要時にvalidate engineへの小さな機械チェック追加で足りる |
| ML-1〜6 | 保守ループ | `design-maintenance-loop` |
| H-1〜7 | Hooks | `design-hooks` |
| RO-1〜5 | OKF render 設計 | `design-engine-render-okf` |

---

## 次の一手（Stage B の正しい順序・ユーザー指摘で確定）

> ⚠️ 整合ゲート（reconcile）は**後段**。先に「conformant とは何か」＝規約＋サンプル＋効果測定を固める。

1. **CodingSchema / code-template を確定**（HOW の実現方法＝Document 集約を持つか等のアーキ判断を含む。旧 C-1/C-2/C-3 を吸収） ← `design-coding-schema` を土台に
2. **動く最小サンプル構成**（tech-stack＋アーキの手本・配布 example。bootstrap の集約リファクタもここに接続＝Stage A 波及）
3. **効果測定**（規約が意図した品質のコードを生むか／サンプルが動くか）
4. **その後 reconcile/整合ゲート**（旧 C-4・強制＝reconcile×Hooks／ES-3 で帰属確定）→ `brainstorm-stage-b-conformance`（保留中・ここで再開）

別系統（並行可）: **OKF #31（frontmatter 整合）** ＝ Stage C・低コスト・PoC 実証済み。

---

## 関連 doc 索引

- **計画**: `implementation-plan`（P0-7詳細）・`sprint-plan`（S1-4）・`spec-id-map`（engine→spec id）・`advisor-ecosystem-roadmap`（Stage D詳細・Phase 1-5）
- **概念/設計**: `brainstorm-has-udd-concept`・`brainstorm-has-udd-design`
- **engine 群**: `design-engine-{set,query,render,scaffold,knowledge}`・`design-render-primitives`・`design-schema-and-engine-skills`
- **Schema**: `design-spec-schema`・`design-coding-schema`
- **品質/保守**: `sim-code-spec-link-projection`・`design-hooks`・`design-maintenance-loop`
- **OKF**: `brainstorm-okf-has-udd`・`design-engine-render-okf`・`brainstorm-okf-frontmatter-relations`・`design/okf-prototype.html`
- **配布**: `brainstorm-multi-tool-compatibility`
- **DDD知識**: `brainstorm-ddd-knowledge-skill`
