# has-udd 実装スプリント計画

詳細な依存グラフ・フェーズ定義は `docs/planning/implementation-plan.md` を参照。
本書はそれを **スプリント（反復）単位の開発バックログ**に落としたもの。

方針: **bootstrap（手作りコア4engine）→ dogfood**。
各スプリントは「動く増分」を出し、効果測定する。

---

## Definition of Done（全スプリント共通）

各スプリントの全成果物が以下を満たして初めて「完了」とする。
**移行レディ（後で has-udd 管理下へ無痛 snap-in）を DoD に焼き込む。**

- [ ] 受け入れテストが緑（振る舞いを固定 = 将来の UsecaseSpec の TestScenarios になる）
- [ ] コードは `@spec:` / `@stack:` アンカーを DocComment に持つ（Spec 未作成なら placeholder id）
- [ ] 本体は `has-udd:impl-start/end`（gen-gap）で囲う
- [ ] code-template 規約（配置 / 命名 / 依存方向=内向き）に従う
- [ ] 該当 engine は `.has-udd/skills/*.json`（Skill 文書）と対応が取れている
- [ ] 「engine → 将来 spec id」対応表（`docs/planning/spec-id-map.md`）を更新
- [ ] スプリントゴールの効果測定が記録されている

---

## Sprint 1 — コア思想の一次実証（検証して描画する）

> **ゴール: 「schema が構造を持つ・engine は汎用・AI は値だけ」を実証する。既存の Skill document.json を SKILL.md にレンダリングし、Claude Code の skill として動かす。**

| バックログ | タスク | 完了条件 |
|---|---|---|
| Phase 0: パッケージ骨格 | #21 | `src/has_udd/` ヘキサゴナル骨格・`shared`(Result/契約定数)・`domain/ports`・`adapters/outbound`(fs/jsonschema/schema_repository) が import 通る／schema をロードできる |
| Phase 1: validate engine | #22 | 既存 `.has-udd/skills/*.json` が SkillSchema/v1 で **PASS** |
| Phase 2: render engine | #23 | `harness-query-engine.json → SKILL.md` を生成し、**Claude Code が skill として認識・動作** |

- **依存**: なし（最初のスプリント）
- **デモ/効果測定**: 生成 SKILL.md を実際に skill として呼び出し、期待どおり動くか
- **得られる確信**: コアの Harness 原則が実際に回る

---

## Sprint 2 — コアループ完成（読んで・作る・叩く）

> **ゴール: scaffold→fill→validate→render の一周を完成させ、CLI で全 engine を叩けるようにする（bootstrap 終了）。**

| バックログ | タスク | 完了条件 |
|---|---|---|
| Phase 3: query engine + 動的_index | #24 | doc から動的 _index（x-prompt-query×blockType）取得・block を id で取得 |
| Phase 4: scaffold engine | #25 | 空 SkillScaffold → fill → validate → render が**一周**する |
| Phase 5: CLI inbound | #26 | `has-udd validate\|render\|scaffold\|query` が叩ける |

- **依存**: Sprint 1（P0,P1,P2）
- **デモ/効果測定**: CLI で「空 doc を scaffold → 値を埋める → validate → SKILL.md」を通しで実演
- **マイルストーン**: ★ **bootstrap 完成**（コア4engine が手作りで揃う）

---

## Sprint 3 — dogfood 準備（Spec を書ける状態にする）

> **ゴール: SpecSchema/v1.json を確定し、has-udd 自身の Spec を scaffold/validate/render できる状態にする。**

| バックログ | タスク | 完了条件 |
|---|---|---|
| Phase 6: SpecSchema/v1.json | #27 | specKind=bounded-context/domain-model/usecase を Coding/Skill 同規約で定義／render に `.feature`(x-render "feature") を追加／scaffold/validate/render が通る |

- **依存**: Sprint 1（render）・Sprint 2（scaffold）
- **デモ/効果測定**: SpecSchema を scaffold → サンプル UsecaseSpec を fill → HTML と `.feature` に render
- **得られるもの**: dogfood の入口

---

## Sprint 4 — dogfood 開始（自分自身を spec 駆動へ）

> **ゴール: has-udd 自身の UsecaseSpec を書き、bootstrap を has-udd 管理下へ snap-in。以降の機能を spec 駆動で作る。**

| バックログ | タスク | 完了条件 |
|---|---|---|
| Phase 7: dogfood 開始 | #28 | コア engine の UsecaseSpec を retroactive に作成（移行レディ ①〜⑥ を回収）／新機能を1つ spec 駆動で実装してみる |

- **依存**: Sprint 1-3
- **デモ/効果測定**: 「Spec → 赤テスト → 実装 → 緑 → @spec 逆引きでリンク確認」の一周が **has-udd 自身のコードで**回る
- **得られる確信**: has-udd の有用性の本検証（自分で自分を駆動できる）

---

## プロダクトバックログ一覧

| Sprint | Phase | タスク | 増分 |
|---|---|---|---|
| 1 | P0 | #21 | パッケージ骨格 |
| 1 | P1 | #22 | validate engine |
| 1 | P2 | #23 | render engine → SKILL.md（一次実証） |
| 2 | P3 | #24 | query engine + 動的_index |
| 2 | P4 | #25 | scaffold engine（★コアループ完成） |
| 2 | P5 | #26 | CLI |
| 3 | P6 | #27 | SpecSchema/v1.json |
| 4 | P7 | #28 | dogfood 開始 |

---

## 今スプリントに入れないもの（後続バックログ）

Hooks（design-hooks.md）・保守ループ（design-maintenance-loop.md）・reconcile engine（engine-set ES-3）・HarnessAgent・マルチツール互換・MCP adapter・サンプル鮮度④⑤。
→ コアループ（Sprint 1-2）と dogfood（Sprint 4）が回ってから着手。
