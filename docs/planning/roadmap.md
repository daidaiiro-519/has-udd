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

**現在地:** コアループ（document を 作る→検証→render→配置）と engine/CLI/MCP/Spec/BDD は**実装完了**（✅）。HOW品質を守る層（規約強制・reconcile・投影・Hooks）と 知識/OKF/エージェント/マルチツール拡張は**設計止まり**（🟠）。テスト: pytest 10 / behave 63 緑。

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

## Stage S — SpecSchema/v2 再構成 🟠 設計中（Spec 層・Stage B の前提）

> `brainstorm-coding-schema-redefine`（Re-1〜5）で確定/議論中。Spec 層の関心事＝CodingSchema/Stage B とは別ステージ。

| 項目 | 状態 | メモ |
|---|---|---|
| 集約の realize＝宣言的（schema が不変条件・engine は executor） | ✅ 合意（Re-1） | コードに imperative 集約クラス不要＝bootstrap 正しい。OQ-3 は欠陥でない |
| specKind 階層 A案＝`bounded-context / subdomain / aggregate / usecase` | ✅ 合意（Re-5） | domain-model→aggregate リネーム＋subdomain 追加 |
| subdomain 分類（中核/一般/補完） | ✅ advisor 確定（Re-5） | harness-core=中核 / validation=一般 / rendering=補完 |
| ValueObjects の置き場＝aggregate 内 | ✅ 合意（Re-5） | 独立 kind にしない・VO は集約の構成要素・不変条件は宣言的 schema |
| status 遷移の宣言（遷移表）＋ spec への明示 | 🟠 議論中（Re-2/Re-4） | 値=VO(schema)／遷移=集約不変条件(guard) |
| SpecSchema/v2 実装（discriminator 改訂・dm-*→agg-*・sd-* 追加） | ⚪ 未着手 | scaffold/validate は schema 駆動なので新コード最小 |

---

## Stage B — HOW品質を守る層 🟠 設計止まり（最優先の穴）

> コードが「.feature は通るが悪いモデル」になるのを防ぐ多層。今日の議論の中心。

| 項目 | 状態 | 根拠ブレスト / メモ |
|---|---|---|
| CodingSchema 規約（tech-stack/code-template/test-template） | 🟡 schema＋instanceは在る・**強制が弱い** | `design-coding-schema` |
| code↔spec リンク＝**投影（reconcile）** | 🟠 結論あり・**OQ-1〜7 未解決** | `sim-code-spec-link-projection` |
| reconcile engine の帰属 | 🟠 **ES-3 未決** | `design-engine-set`（ES-3） |
| Hooks（検証ゲートの機械強制） | 🟠 H-1〜7 | `design-hooks` |
| 保守ループ（陳腐化対策・drift検知） | 🟠 ML-1〜6 | `design-maintenance-loop` |

**未 surface の検討漏れ（要昇格）:** OQ-1 アンカー多重度 / OQ-2 spec無しコードの規約 / **OQ-3 ドメインモデル整合ゲート（最重要）** / OQ-4 不変条件 unit test / OQ-5 重複防止の強制 / OQ-6 2グラフ混同 / OQ-7 supersede時アンカー寿命（→ `sim-code-spec-link-projection` 末尾）。

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

## Stage D — Orchestrator・エージェント・配布 ⚪ 未着手

| 項目 | 状態 | 根拠ブレスト / メモ |
|---|---|---|
| Agent(Role) Schema | ⚪ | `design-engine-knowledge`（roleKind）・concept |
| custom Skill Schema | ⚪ | `design-schema-and-engine-skills` |
| HarnessAgent（Orchestrator・engine routing） | ⚪ | `brainstorm-has-udd-concept`・engine-awareness |
| FeedbackReport | ⚪ | — |
| Multi-tool 互換（Skills/Hooks/Agents/rules） | 🟠 ブレスト | `brainstorm-multi-tool-compatibility` |

---

## 横断：未解決論点レジストリ

| ID | 論点 | 置き場 |
|---|---|---|
| OQ-1〜7 | code↔spec 投影の検討漏れ | `sim-code-spec-link-projection` |
| ES-3 | reconcile engine の帰属 | `design-engine-set` |
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

- **計画**: `implementation-plan`（P0-7詳細）・`sprint-plan`（S1-4）・`spec-id-map`（engine→spec id）
- **概念/設計**: `brainstorm-has-udd-concept`・`brainstorm-has-udd-design`
- **engine 群**: `design-engine-{set,query,render,scaffold,knowledge}`・`design-render-primitives`・`design-schema-and-engine-skills`
- **Schema**: `design-spec-schema`・`design-coding-schema`
- **品質/保守**: `sim-code-spec-link-projection`・`design-hooks`・`design-maintenance-loop`
- **OKF**: `brainstorm-okf-has-udd`・`design-engine-render-okf`・`brainstorm-okf-frontmatter-relations`・`design/okf-prototype.html`
- **配布**: `brainstorm-multi-tool-compatibility`
- **DDD知識**: `brainstorm-ddd-knowledge-skill`
