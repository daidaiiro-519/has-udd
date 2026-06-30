# SpecSchema/v2 設計（Stage S・spec 層の完成）

**目的:** SpecSchema を v2 に再構成する。specKind 階層を DDD 語彙に合わせ（A案）、集約の宣言的 realize（Re-1）・status 遷移（Re-2）・「何でも集約」防止（Re-4）・subdomain/VO（Re-5）を schema に落とす。
**前提（確定済み）:** Re-1（集約は宣言的に schema が不変条件・engine は executor）／Re-5（specKind=bc/subdomain/aggregate/usecase・subdomain 分類・VO は aggregate 内）。
**出典:** `brainstorm-coding-schema-redefine` Re-1〜5 ／ ddd-advisor（domain-model/subdomain/bounded-context/closing-heuristics）。

---

## 1. specKind と discriminator

`specKind ∈ { bounded-context, subdomain, aggregate, usecase }`（allOf discriminator）。v1 の `domain-model` は `aggregate` にリネーム、`subdomain` を新設。

```
bounded-context ── 1:N ── subdomain ── 1:N ── usecase ── aggregateRef ──▶ aggregate
                                                        （usecase は集約を操作）
```

---

## 2. 各 specKind の content ブロック（確定版・実例＝`docs/design/spec-v2/`）

全 specKind 共通: 先頭に **`Summary`（1〜2文・必須）**。

### bounded-context
| ブロック | 要否 | 中身 |
|---|---|---|
| Summary | 必須 | 1〜2文 |
| UbiquitousLanguage | 必須 | 用語→定義（**この bc 内に閉じる**） |
| Members | 必須 | 配下の subdomain / aggregate / usecase の id |
| ContextMap | 任意 | 他 bc との関係（相手・関係種別・内容） |

### subdomain（新）
| ブロック | 要否 | 中身 |
|---|---|---|
| Summary | 必須 | 1〜2文 |
| Category | 必須 | `core` / `generic` / `supporting` |
| CategoryRationale | 必須 | 差別化の根拠 |
| Members | 必須 | 所属 usecase の id |
| ImplementationGuidance | 必須 | 実装の厚み（CodingSchema subdomainRichness の宣言元） |
| ExternalSolution | 任意 | 一般のみ・採用既製品＝@stack の宛先 |
| **DomainServices** | 任意 | **集約に跨るステートレス業務ロジック**（`{name, 責務}`）。★aggregate でなくここに置く（DDD: 業務サービスは1集約の所有物でない） |

### aggregate（旧 domain-model を精緻化）
| ブロック | 要否 | 中身 |
|---|---|---|
| Summary | 必須 | 1〜2文 |
| AggregateRoot | 必須 | ルート名 ＋ **外部参照**（他集約は ID 参照） |
| **Entities** | 必須(≥1=ルート) | `{ name, role, identifier, isRoot, attributes: [{name, type}] }`。**ルート＋子を1表に・属性の型は VO 名** |
| ValueObjects | 任意 | **型カタログ** `{ name, represents, behavior }`（属性の型定義） |
| Invariants | 必須(≥1) | `{ rule, enforcement: schema \| guard }`（Re-4:「不変条件0＝集約でない」） |
| Lifecycle | 状態あれば必須 | `{ states[], transitions: [{from, to, command}] }`（Re-2: 遷移表を guard が読む） |
| Commands | 必須 | `{ name, does, args, requiresState, emits, postState }` |
| DomainEvents | 任意 | `{ name（過去形）, payload, trigger }` |
| UnitTestScenarios | 任意 | TestScenarios 構造（下記）＝不変条件の検証 |

### usecase（v1 ＋ 参照・アクター追加）
| ブロック | 要否 | 中身 |
|---|---|---|
| Summary | 必須 | 1〜2文 |
| subdomainRef / aggregateRef | 必須 | 所属 subdomain ／ 操作する集約 |
| **主アクター / 意図** | 必須 | 主アクター（誰）＋意図（何をしたいか）を分離 |
| **関与する外部** | 任意 | 副アクター・隣接コンテキスト（図1-3 の「同じ外部システム」） |
| Preconditions | 任意 | 事前条件 |
| **基本フロー**（旧 MainFlow） | 必須 | シーケンス図（「主成功フロー」→「基本フロー」に改称） |
| Postconditions | 必須 | 事後条件 |
| AcceptanceCriteria | 必須 | EARS 記法 |
| Errors | 任意 | `{ code, 条件 }` |
| TestScenarios | 必須 | 構造（下記）＝AI 実装の主契約・.feature へ render |

### TestScenarios / UnitTestScenarios の構造（確定）
```
{ background?: string,                              // 共通の前提（Gherkin Background・任意）
  scenarios: [ { name,                              // シナリオ名（概要）
                 category,                          // 分類: 正常系 / 異常系 / 境界値
                 viewpoint,                         // 観点: 側面＋検証の狙い
                 gherkin,                           // Given/When/Then（実行可能・.feature 化）
                 covers? } ] }                      // 任意: 検証する不変条件/受け入れ基準への参照（トレーサビリティ）
```
render: 各シナリオを `### 名前` ＋ `| 分類 | 観点 |` 表 ＋ Gherkin コードブロックに整形。

---

## 3. Re-2 解決: status 遷移の表現

- **値（静的）** = `status` は ValueObjects の1つ。enum 制約は schema（validate が守る）。
- **遷移（動的）** = aggregate.Lifecycle.transitions に**宣言的な遷移表**として持つ:
  ```
  states: [CREATED, VALIDATED, RENDERED, SUPERSEDED]
  transitions:
    - { from: CREATED,   to: VALIDATED, command: validate }
    - { from: VALIDATED, to: RENDERED,  command: render }
    - { from: RENDERED,  to: SUPERSEDED, command: supersede }
  ```
- **enforce は guard**（Stage B で“置き場”を決める）。guard は遷移表を読む決定的コード＝harness（ロジックを散らさない）。

## 4. Re-4 解決: 「何でも集約」を防ぐ

aggregate spec は **(a) 識別子 ＋ (b) enforce する不変条件を最低1つ ＋ (c) 一貫性境界の宣言** を必須にする。**「不変条件を1つも挙げられない概念は集約でない」**（VO か単なるデータ）＝DDD の集約の定義そのものをスキーマ必須項目で強制する。これで貧血な「何でも集約」を schema レベルで弾く。

## 5. has-udd の v2 spec ツリー（具体）

```
bc-has-udd-engines (bounded-context)
├ sd-harness-core (subdomain, core)     members: uc-scaffold, uc-query, uc-reconcile(将来)
├ sd-validation   (subdomain, generic)  members: uc-validate   externalSolution: jsonschema
├ sd-rendering    (subdomain, supporting) members: uc-render, uc-deploy(将来)
├ agg-document (aggregate)  VO: status/documentId/schemaRef/tags/discriminator・Lifecycle 遷移表
├ agg-schema   (aggregate)  VO: version 等・Lifecycle(PUBLISHED→DEPRECATED)
└ uc-scaffold / uc-validate / uc-render / uc-query (usecase・各 aggregateRef=agg-document・subdomainRef=上記)
```

## 6. 移行計画（v1→v2）

1. SpecSchema/v2.json 作成（specKind enum 改訂・subdomain ブロック追加・aggregate に Invariants{enforcement}/Lifecycle{transitions} 追加・usecase に subdomainRef 追加）。
2. dm-document→agg-document / dm-schema→agg-schema にリネーム＋ Invariants/Lifecycle を v2 形に。
3. sd-harness-core / sd-validation / sd-rendering を新規作成。
4. bc-has-udd-engines の Members 更新。
5. uc-* に subdomainRef 追加。
6. validate/render/scaffold engine は**schema 駆動なので新コードほぼ不要**（v1→v2 は scaffold/validate がそのまま処理）。

---

## 7. 描画方針（MD 正本・HTML は機械変換）★今回追加

- **part_renderer は MD を正本にする**（md/html 二重ロジックを廃し html 分岐撤去＝コード減）。理由＝OKF が MD 前提なので render と OKF の素材を一本化。
- **HTML = MD を載せた client-side viewer（marked＋mermaid＋独自CSS のエンベロープ）**。サーバ側 MD→HTML 依存ゼロ・OKF viewer（`okf-prototype.html`）と統一・#32/#33 と合流。
- **pandoc は見送り**（重い外部バイナリ・PDF/多形式/JS無し静的が要るまで不要・[[feedback-library-selection]]）。**marp は不採用**（スライド用＝用途違い）。JS 無し静的が要る時のみ後で markdown-it-py（pure Python）。
- x-render-target: **md 正本 ＋ html=viewer ＋ feature**（test-scenario block から .feature は従来通り）。

## 8. content 設計原則（2レンズ: AI実装に十分／人に見やすい）★今回追加

| 観点 | 効かせるブロック |
|---|---|
| AI 実装の主契約（曖昧さを消す） | **TestScenarios(Gherkin)**・Pre/Post/Errors・Invariants{enforcement}・Commands{requiresState/postState} |
| 人の即スキャン | **Summary（新・必須・1〜2文）**・MainFlow(sequence図)・各種表 |

**refinement:**
- 全 specKind に **`Summary`（1〜2文）を必須**追加（両レンズに効く唯一の追加）。
- TestScenarios＝AI 実装の主契約として厚く／MainFlow 図＝人向けの主役、と役割分担。
- 自由テキストは増やさない（構造化→x-render で整形・書き手依存を排除）。
- **十分性の判定基準**: その spec 単体を別 AI に渡して実装着手でき、TestScenarios が受け入れになる（Stage B 効果測定 Re-3 と直結）。

## 決定事項（確定）

- **D-1** ✅ subdomain は独立 doc。
- **D-2** ✅ 「何でも集約」防止＝aggregate は不変条件≥1 必須（error）。
- **D-3** ✅ 移行は今やる（dm-*→agg-*）。
- **D-4** ✅ 描画＝MD 正本＋client-side viewer（pandoc/marp 不採用・ブラウザ閲覧前提で合意）。
- **D-5** ✅ 全 specKind に Summary 必須。
- ✅ **DomainServices は aggregate でなく subdomain に置く**（DDD 準拠）。
- ✅ **Entities はルート＋子を1表に・属性（型=VO）を明示**／VO は型カタログ。
- ✅ **TestScenarios は {background?, scenarios:[{name,category,viewpoint,gherkin,covers?}]}**・render は表＋Gherkin。
- ✅ usecase は **主アクター/意図/関与する外部** を分離・MainFlow→**基本フロー**。
- ✅ **画面仕様は spec に含めない**（プレゼンテーション層＝ドメインの外）。

→ 次は **A: SpecSchema/v2.json 実装**（本構成を JSON Schema に落とす）。
