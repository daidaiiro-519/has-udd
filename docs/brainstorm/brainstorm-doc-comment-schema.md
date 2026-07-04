# ブレインストーミング: DocCommentSchema（構造化 docstring 検証・多言語対応）

**目的:** サンプル実装中に発覚した「docstring は機械検証できる構造になっているか」という指摘を起点に、
`DocCommentSchema`（code_scan の出力を検証する schema）と `uc-lint-doc-comment`（適合判定）の設計を確定する。
**モード:** 設計判断（論点 → 見解 → 合意）
**状態: 全論点 CLOSED（2026-07）**

**確定前提（揺らさない）:**
- `code_scan`（`uc-scan-source-code`）が docstring を先に JSON へ構造化抽出する。DocCommentSchema は
  **その JSON 出力を検証する普通の JSON Schema**（RenderMetaSchema と同格のメタ schema）。
- カスタムタグ禁止（D-5）・「spec が schema を決める」（`uc-scan-source-code`/`uc-lint-doc-comment` の
  postconditions/acceptanceCriteria を具体的フィールドまで書き下してから schema を実装する）。

---

| # | 論点 | 状態 |
|---|---|---|
| DC-1 | usecase は分けるか（query engine 拡張 vs 新規） | ✅ CLOSED |
| DC-2 | DocCommentSchema は新集約か | ✅ CLOSED |
| DC-3 | 各 schema の具体フィールドは spec で裏付けるべきか（既存 schema への遡及） | 🟡 A で暫定進行（既存schema遡及はバックログ） |
| DC-4 | マーカー→フィールド対応・summary/body 検出規則の宣言 | ✅ CLOSED |
| DC-5 | 適合判定（lint）は自前実装か既存ツールか | ✅ CLOSED |
| DC-6 | 多言語対応の範囲 | ✅ CLOSED |

---

## 論点 DC-1: usecase は分けるか

**決定:** `uc-scan-source-code`（sd-harness-core・index/抽出）と `uc-lint-doc-comment`（sd-validation・適合判定）の
**2本を新規作成**。当初「query engine を拡張すればいい」と考えたが撤回——アクターの意図が違う
（Document のクエリ ≠ ソースコード走査）ため、実装都合でドメインモデルを結合する誤りだった。
既存の query/validate という対概念を、対象（Document→ソースコード）だけ変えてそのまま再現する形。

## 論点 DC-2: DocCommentSchema は新集約か

**決定:** 新集約は作らない。RenderMetaSchema と同格の「メタ schema」として扱う（Document 本体でなく
別のアウトプットを検証する schema・agg-schema の概念に自然に収まる・agg-schema.json への編集も不要
＝RenderMetaSchema/SkillSchema も個別に列挙されていない前例と同じ）。

## 論点 DC-3: 各 schema の具体フィールドは spec で裏付けるべきか

**ユーザー見解:** 「各 schema の具体的なフィールド表現が spec として書かれていないと実装できない」
——これは DocCommentSchema だけでなく普遍的な原則（値オブジェクトのように、どの schema にも一様に
適用されるべき規律）。遡って確認すると CodingSchema/SkillSchema/RenderMetaSchema も同じ穴を持つ
（ブレスト doc でのみ決定・正式な UsecaseSpec による裏付けが無い）。SpecSchema 自身だけは例外（鶏と卵）。

**合意（水準1）:** 今回は **A**（DocCommentSchema だけ正しい手順で進め、既存 schema の遡及修正は
バックログとして記録するに留める）で進行。`uc-scan-source-code`/`uc-lint-doc-comment` は実際に
postconditions/acceptanceCriteria を具体的フィールドまで書き下してから DocCommentSchema を実装した
（本ブレストの DC-4〜6 がその実践）。**既存 schema への遡及は未着手のまま残る**（次回以降の判断）。

## 論点 DC-4: マーカー→フィールド対応・summary/body 検出規則の宣言

**見解の変遷（重要な自己訂正の連続）:**
1. 当初、DocCommentSchema を「テキスト文法の宣言」として設計しようとしたが、`code_scan` が先に
   JSON化するなら普通の JSON Schema でよいと判断→撤回。
2. しかしユーザー指摘：「DocComment に content にあるフィールドが書かれていないのに、なぜ紐づけられるのか」
   ——JSON の**検証**（出力の形）は schema 化したが、**テキスト→JSON への変換ルール**（`"Args:"` や
   `"@param"` がどのフィールドに対応するか）はどこにも宣言されず、抽出スクリプトにハードコードされたままだった。
3. さらにユーザー指摘：「なぜ Summary を明示的に書かせないのか」——確認の結果、**両言語の実際の標準慣習に
   summary の明示マーカーが存在しない**（Google: 1行目という位置規約のみ／tsdoc: 最初のブロックタグまでが
   summary という位置規約のみ）。summary は「マーカー方式」でなく「位置方式」で検出するのが言語の実態として正しい。
4. ユーザー指摘：「正式なフォーマットはどれか」——WebFetch で公式仕様を確認したところ、
   **私が最初に想定した tsdoc のルールは誤りだった**（`@remarks` の存在・`@throws {@link Type}` 記法など）。

**最終決定：`DocCommentSchema` に `x-extraction-rules` を追加。**
kind ごとに `summaryBoundary`（line / first-block-tag / sentence の3種）・`sections`（marker + itemPattern の配列）を
宣言データとして持つ。抽出エンジンはこの宣言を読んで汎用的にパースし、マーカー文字列を言語ごとにハードコードしない
（RenderMetaSchema が閉じた語彙を宣言し part_renderer.py が汎用実行するのと同じパターンを、逆方向
（テキスト→JSON）に適用）。

**実証:** Python(google)・TypeScript(tsdoc) の実ソースから実際に抽出し、`DocCommentSchema/v1.json` で
検証エラー0件を確認（`/tmp/doc_comment_schema_demo_python.py`・`/tmp/doc_comment_schema_demo_ts.js`）。

## 論点 DC-5: 適合判定（lint）は自前実装か既存ツールか

**ユーザー見解:** 「index 作成には schema が要るかもしれないが、バリデーションは公式 lint があるならそれを使うべきでは？」

**調査結果（WebFetch で確認済み）:**
- Python: **pydoclint**（Google/Sphinx/NumPy 対応・darglint より高速・引数/戻り値/例外の不整合を検出）
- TypeScript/JS: **eslint-plugin-jsdoc**（`check-param-names` 等・TS 対応済み）
- Java: **Checkstyle `JavadocMethod`**（`javadoc.unusedTag`/`javadoc.expectedTag` の組み合わせで
  過不足両方向を検出可能と確認）
- Go: 公式に args/returns/raises の構造化記法が無い（`revive` 等で docstring の有無のみ検出可）
- Rust: 公式に引数ごとの構造化記法が無い（rustc 組み込み `missing_docs` lint で有無のみ検出可）

**決定:** `uc-lint-doc-comment` は**自前の照合ロジックを持たず、kind ごとに確立された既存ツールを呼び出す
アダプタ**として設計変更。「既存手段で代替できないか先に確認する」という方針（ライブラリ選定フィードバック）に
最初から従うべきだった、という反省を伴う決定。

## 論点 DC-6: 多言語対応の範囲

**決定:** 5 kind（google/tsdoc/javadoc/godoc/rustdoc）を**全て公式仕様で裏付けて確定**。
godoc/rustdoc は「言語の公式規約に args/returns/raises に相当する構造が無い」という事実をそのまま
schema に明記し、無理に構造を作らない（カスタムタグ禁止の原則と一貫）。

---

## 帰結（実装済み）

- `waffle/src/waffle/domain/model/DocCommentSchema/v1.json`：kind enum 5種＋`x-extraction-rules`。
- `waffle/.waffle/documents/specs/uc-scan-source-code.json`：postconditions/acceptanceCriteria を
  具体的フィールド（`hasDocstring`・`signatureParams` 含む）まで書き下し済み。
- `waffle/.waffle/documents/specs/uc-lint-doc-comment.json`：アダプタ方式に設計変更・5 kind 対応・
  `TOOL_NOT_AVAILABLE` エラー追加。
- 全 spec validate 済み・render 済み・pytest 15/15・behave 69/69 緑。

## 残作業（次回以降）

1. `code_scan`/`lint` を `waffle` の query engine 拡張として実際に実装（宣言（`x-extraction-rules`）を
   読んで動く汎用パーサ・既存ツールを呼び出すアダプタ）。
2. DC-3 の「既存 schema（CodingSchema等）への遡及」— 引き続きバックログ。
