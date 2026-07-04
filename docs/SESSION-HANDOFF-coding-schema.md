# セッション引き継ぎ資料 — CodingSchema / DocCommentSchema（2026-07-04時点）

このドキュメントは、**本セッションで行った CodingSchema（Stage K）関連の議論・実装**を、
別セッションが会話の全履歴を読まずに引き継げるようにするためのブリーフィングです。

> リポジトリ全体の構造・並行編集の競合解決基準は `HANDOFF.md`（repo root）・
> `docs/SESSION-HANDOFF.md` を参照。本ドキュメントは**このセッションで進んだ設計議論の中身**を扱う。
>
> ⚠️ **パス表記の注意**: 本セッション中に別セッションが `waffle/` を完全自己完結ディレクトリへ
> 移行した（`HANDOFF.md` 参照）。このため、以下で `.has-udd/documents/coding/...` 等と書いている
> 箇所は、現在は **`waffle/.waffle/documents/coding/...`** に対応する。ソースは `waffle/` 配下、
> パッケージ名は `waffle`（旧 `has_udd`）。
>
> ⚠️ **用語変更**: 「DocComment」は一般的でないため `DocCommentSchema` → **`DocstringSchema`**、
> `uc-lint-doc-comment` → **`uc-lint-docstring`** に改名済み（CodingSchema の `docComment` ブロックも
> `docstring` に統一）。以下の本文中の旧名称は当時の記録として残す。

---

## 1. 到達点（CodingSchema Stage K）

### CS-1〜CS-3：全 CLOSED（`docs/brainstorm/brainstorm-coding-schema.md`）

- **層モデル確定**：DDD に「詳細設計」層は無い。仕様＝ドメインモデル（SpecSchema）｜実装＝コード。
  CodingSchema＝「規約」（詳細設計文書ではない）。
- **codingKind＝4種＋examples/**：`tech-stack` / `architecture` / `coding-standard` / `test-standard`。
- **概念キー化は architecture が担う**（specKind と 1:1）。
- **アンカー（`@spec`/`@stack`）は全廃**（後述の D-1〜D-6 参照）。

### content schema 化：完了

- `waffle/src/waffle/domain/model/CodingSchema/v2.json` を実装（旧 v1 は削除・supersede）。
- 4 codingKind のブロック構造を全て確定（各ブロックのサンプル値入り後イメージが
  `docs/design/coding-render-image-{tech-stack,architecture,coding-standard,test-standard}.md` にある）。
- python-hexagonal スタックの instance 4本を作成・validate/render 確認済み
  （`waffle/.waffle/documents/coding/{tech-stack,architecture,coding-standard,test-standard}-python-hexagonal.json`）。

### 動く最小サンプル：完了（`docs/brainstorm/brainstorm-minimum-sample.md` MS-1〜5 全 CLOSED）

- **サンプルは has-udd 自身のコードを指してはいけない**（規約と共進化してきたコードは
  「規約だけで実装できるか」の検証にならない＝カンニング）。
  **規約確定後に初めて着手する新規コード**でなければならない、という原則が確定。
- 実際に **worktree で隔離した新規エージェント**（`src/has_udd`/`waffle` 側のソースを一切読ませない）
  に、render 済みガイド（.md ×4）だけを渡して `examples/python-hexagonal/` を新規実装させた。
  ConceptPlacement の全9概念（usecase/aggregate/entity/value-object/domain-service/repository/
  port/inbound-adapter/outbound-adapter）をプレースホルダ題材で総当たり実装。
- **比較の結果**：architecture 規約（配置・依存方向・概念の形）は独立した実装者を正しい骨格に
  収束させることを実証。一方でエラー表現（構造化コード vs 単純メッセージ）は規約に無く分岐した
  ＝規約の欠落と判定し、architecture の Rules に「失敗は識別可能なエラーコードを伴う結果型で返す」
  を追加。**サンプルもこのフィードバックに追従済み**（`Err(code, message)` へ移行・テスト緑）。
- サンプルは `examples/python-hexagonal/`（pytest 13/13 緑・独立動作）。

---

## 2. AI 時代の DDD 詳細設計 — 憲法級の議論（`docs/brainstorm/brainstorm-ai-era-detail-design.md` D-1〜D-6）

サンプル実装中に「リンク管理が陳腐化しそう」という違和感から始まり、既存の
`sim-code-spec-link-projection`・`design-coding-schema` の結論を **supersede** する形で決着した。

- **D-1（アンカー全廃）**：`@spec`/`@stack` は正典配置（architecture の ConceptPlacement）で
  代替できる冗長情報だった。リンクは「管理」でなく「計算」するもの。
- **D-2（詳細設計＝クエリ結果）**：詳細設計書は作らない。永続化するのは意図(spec)・制約(規約)・
  振る舞い検証(シナリオ)のみ。API仕様/DBスキーマ等はコードから生成する投影（手書き禁止）。
- **D-3（導出物＝キャッシュ）**：「保存するな」ではなく「手で維持するな」。機械が決定的に
  再生成できるものは保存してよい。陳腐化とは「キャッシュを人が手で更新すること」の別名。
  **has-udd の存在意義＝「AI開発のトークン経済を成立させるハーネス」**という言語化に到達。
- **D-4（DocComment 動的インデックス）**：`code_scan` という新機構を設計。**PoC で技術検証済み**
  （`/tmp/poc_code_scan.py`・Python 標準 `ast` のみ・AI 0・実測 1/7 圧縮）。
- **D-5（DocComment 形式）**：カスタムタグ禁止・言語標準スタイル（Google docstring 等）のみ。
  要約行＝x-prompt-write と同じ役割（検索・判断に効く語で書く）。
- **D-6（gen-gap 廃止）**：`impl-start/end` マーカーも同じ理由で全廃（コードは AI が author する
  もので機械再生成しないため保護対象が無い）。

**実施済み**：既存コード11ファイルから `@spec:`/`@stack:`/gen-gap を除去・docstring 整形
（pytest/behave 緑を維持）。

---

## 3. DocCommentSchema：全論点 CLOSED（`docs/brainstorm/brainstorm-doc-comment-schema.md`）

中断していたこの議論は**再開・完全決着**した。詳細は `brainstorm-doc-comment-schema.md`（DC-1〜6）参照。

**要点：**
- `uc-scan-source-code`（index・sd-harness-core）/`uc-lint-doc-comment`（適合判定・sd-validation）の
  2 usecase 分割は確定のまま、postconditions/acceptanceCriteria を**具体的フィールドまで書き下し済み**。
- `DocCommentSchema/v1.json`（`waffle/src/waffle/domain/model/`）を実装。**普通の JSON Schema**（RenderMetaSchema
  と同格のメタ schema）＋**`x-extraction-rules`**（テキスト→JSON のマーカー対応規則を宣言データ化。
  `summaryBoundary` + `sections[].marker/itemPattern`）。google/tsdoc/javadoc/godoc/rustdoc の**5 kind すべて
  公式仕様（Google styleguide/tsdoc.org/Oracle Javadoc spec）で裏付け済み**。
- Python(google)・TypeScript(tsdoc) は実ソースから抽出→schema検証まで実証済み（エラー0件）。
- **`uc-lint-doc-comment` は自前実装でなく既存 lint ツールを呼ぶアダプタに設計変更**：
  google→**pydoclint**／tsdoc→**eslint-plugin-jsdoc**／javadoc→**Checkstyle JavadocMethod**／
  godoc→**revive**（有無のみ）／rustdoc→**rustc 組み込み missing_docs**（有無のみ）。
- 全 spec validate/render 済み・pytest 15/15・behave 69/69 緑。

**残作業：**
1. `code_scan`/`lint` を `waffle` の query engine 拡張として正式実装（`x-extraction-rules` を読む汎用パーサ・
   既存ツール呼び出しアダプタ）。
2. **DC-3（未決着のまま）**：「各 schema の具体フィールドは spec で裏付けるべき」という原則は合意したが、
   **CodingSchema/SkillSchema/RenderMetaSchema への遡及適用は引き続きバックログ**。今回は
   「DocCommentSchema だけ正しい手順で進める」（選択肢A）で進行した。

## 3'. （旧・以下は歴史的経緯として残す）当初の中断メモ

サンプルの docstring を「機械的に構造検証できるか」を突き詰めていく中で、長い DDD 論争になった。

### 到達した設計判断（確定）

- **DocCommentSchema は「テキスト文法の宣言」ではなく普通の JSON Schema でよい**。
  理由：`code_scan` が docstring を先に `{summary, body, args, returns, raises}` という
  **JSON に構造化抽出**するので、それを検証するのは既存の `Validator` port /
  `jsonschema_validator.py` アダプタで足りる（新規検証機構は不要）。
- **usecase は2本、新規作成済み・validate/render確認済み**：
  - `uc-scan-source-code`（sd-harness-core・aggregateRef無し）
  - `uc-lint-doc-comment`（sd-validation・aggregateRef無し）
  - 当初「query engine を拡張すればいい」と考えたが撤回——**アクターの意図が違う**
    （Document のクエリ ≠ ソースコード走査）ため、実装都合でドメインモデルを結合する誤りだった。
- **DocCommentSchema は新しい集約ではない**。RenderMetaSchema と同格の「メタ schema」
  （Document 本体でなく別のアウトプットを検証する schema）として `agg-schema` の枠内で扱える
  ——ただし agg-schema の Entities（`blocks: "構造定義(...)"` という曖昧な1属性）が
  実際の schema 間の多様性（SpecSchema と CodingSchema で content block が全く違う）を
  隠しているという**ユーザ指摘は正当**（leaky abstraction）。

### ★未決着（再開時にここから）

**ユーザの核心的な指摘**：「各 schema の具体的なフィールド表現が spec として書かれていないと
実装できない」。これは DocCommentSchema だけの話ではなく、**普遍的な原則**（値オブジェクトの
ように、どの schema にも一様に適用されるべき規律）。

遡って確認すると、**既存の CodingSchema/SkillSchema/RenderMetaSchema も同じ穴を持つ**——
これらの具体的なフィールド設計は `docs/brainstorm/brainstorm-coding-schema.md` 等の
**ブレスト doc でのみ**決定されており、正式な UsecaseSpec（Postconditions/AcceptanceCriteria で
具体的フィールドを規定するもの）による裏付けが無い。**SpecSchema 自身だけは例外**
（spec を書く仕組みそのものなので、それを裏付けるメタ spec は原理的に存在し得ない＝鶏と卵）。

**提示した2択（ユーザ未回答のまま中断）：**

| 選択肢 | 内容 |
|---|---|
| A | 今回は DocCommentSchema だけ正しい手順（`uc-scan-source-code` の postconditions/
      acceptanceCriteria を具体的フィールドまで書き下してから実装）で進め、既存 schema の
      遡及修正はバックログとして記録するに留める |
| B | 一旦立ち止まり、CodingSchema 等の既存 schema にも遡って裏付け spec を書く |

**再開時の最初のアクション**：この A/B をユーザに確認する。A で進める場合の次の手順：

1. `uc-scan-source-code.json` の postconditions/acceptanceCriteria を具体的フィールド列挙
   （kind・path・elementKind・name・summary・body・args:[{name,description}]・returns・
   raises:[{exceptionType,condition}]）まで書き下す。
2. `uc-lint-doc-comment.json` も同様に、違反コード（MISSING_DOC_COMMENT/EMPTY_SUMMARY/
   ARGS_MISMATCH 等）と検査対象の具体的な対応関係を書き下す。
3. その記述から `DocCommentSchema/v1.json`（google kind から。将来 tsdoc/javadoc/godoc/rustdoc
   の discriminator 拡張余地を残す）を実装。
4. `code_scan` を `waffle` の query engine 拡張として正式実装（PoC は済み）。

---

## 4. 参照先

- **memory**（`~/.claude/projects/.../memory/`）：
  `project-coding-schema-stage-k.md`（本セッションの全経緯・最も詳しい）・
  `project-ai-era-detail-design.md`（憲法級・D-1〜D-6）・`project-why-spec-exists.md`。
- **brainstorm doc**：`docs/brainstorm/brainstorm-coding-schema.md`（CS-1〜3）・
  `docs/brainstorm/brainstorm-ai-era-detail-design.md`（D-1〜D-6）・
  `docs/brainstorm/brainstorm-minimum-sample.md`（MS-1〜5）。
- **design doc**：`docs/design/coding-render-image-*.md`（4 codingKind の後イメージ）・
  `docs/design/coding-architecture-blocks-filled.md`（architecture ブロックの中身入り検討過程）。
- **実装**：`waffle/src/waffle/domain/model/CodingSchema/v2.json`・
  `waffle/.waffle/documents/coding/*-python-hexagonal.json`・
  `waffle/.waffle/documents/specs/uc-{scan-source-code,lint-doc-comment}.json`・
  `examples/python-hexagonal/`。

## 5. 小さな積み残し（ついでに気づいたもの）

- リポジトリルート直下の `.has-udd/specs/*.md`・`.has-udd/coding/*.md` に、
  ソースが `waffle/.waffle/documents/` へ移動した後の**古い render 出力が git 管理下のまま残っている**
  （22ファイル）。ソース側は移動済みなので、これらは孤立した stale artifact。削除するか、
  `waffle/.waffle/` 側で再レンダリングし直すか、次回整理が必要。
