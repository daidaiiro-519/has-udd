# ブレインストーミング: OKF frontmatter で文書間の「関連性（グラフのエッジ）」をどう設計するか

**目的:** has-udd の文書に OKF frontmatter を載せるとき、グラフのエッジ＝関連性をどう持たせるかを決める。特に「tags（タグ付け）と relations（文書間リンク）の責務分け」「relations の源泉（手書き vs 既存 ref から生成）」「関係種別の語彙」を確定する。
**モード:** 設計判断（論点 → 見解 → 合意）
**前提（確定済み）:** PoC（docs/design/okf-prototype.html）で frontmatter 駆動グラフ（type→ノード色 / relations→エッジ）は実現可能と確認。core 不変（source/query/schema は触らず rendered 層のみ）。本 doc は既存 `design-engine-render-okf.md` の RO-3（cross-link）/ RO-4（frontmatter mapping）を「関連性」に絞って深掘りするもの。
**★対象スコープ（ユーザー指摘で確定）:** グラフの対象は **Document 集約の全インスタンス**＝spec だけでなく **knowledge / agent(role) / skill / coding** も含む。各型が既に持つ ref（下表）を束ねて **型をまたいだ1枚の知識グラフ**にするのが OKF 適用の本体。spec を例にしたのは PoC 都合で意図的除外ではない。

| 文書型 | 既存 ref（エッジの素） | 向き先 | ref の在処 |
|---|---|---|---|
| spec (bc/dm/uc) | `aggregateRef` / bc `contains` | spec → spec | 構造化フィールド |
| skill (engine/custom) | `knowledgeRefs` | skill → knowledge | 構造化フィールド |
| agent (role) | `skillRefs` / `knowledgeRefs` | agent → skill/knowledge | 構造化フィールド |
| knowledge | `knowledgeRefs` / 本文 `[[name]]` | knowledge → knowledge | 構造化＋**本文** |
| coding | `@spec` / `@stack` | code → spec/stack | **コード内アンカー** |

---

## アイデアダンプ（関連性の持たせ方の候補）

1. 本文 markdown リンクだけ（OKF 標準そのまま・種別は散文）
2. frontmatter `relations: [{type, to}]` だけ（機械可読・OKF 標準ツールはリンクを辿れない）
3. 両方併記（frontmatter relations ＋ 本文 md リンク）
4. tags に関係も混ぜる（`tags: [aggregate:dm-document]` のように）
5. 既存の構造化 ref（aggregateRef/knowledgeRefs/skillRefs）を源泉に render が relations を生成
6. relations の type を自由文字列にする / 閉じた語彙（enum）にする
7. グラフのエッジ＝relations のみ、tags はフィルタ用に別レイヤ

**絞り込み候補:** 3（両方併記）× 5（ref を源泉に生成）× 7（tags とエッジを分離）の組み合わせが本命。以下の論点で詰める。

---

## 論点 1: tags（タグ付け）と relations（文書間リンク）は同じものか、分けるべきか

### AI 初期見解
**見解:** **別物として明確に分ける。** tags＝「属性による分類（多対多・無向）」、relations＝「特定の相手文書を名指しで指す有向リンク」。**グラフのエッジになるのは relations だけ**で、tags はエッジにしない（フィルタ／グルーピング用の別レイヤ）。
**根拠:**
- グラフのエッジは「A が B を必要とする／指す」という**有向で1対1の意味**。tags は「同じ属性を持つ文書の集合」で**無向・多対多**。両者を同じ線にすると、意味の違う線が混在して可視化も機械処理も壊れる。
- has-udd は既に役割分担している：構造化 ref（aggregateRef 等）＝有向リンク、tags（`context:has-udd-engines`）＝分類。OKF 化でもこの分担をそのまま写すのが自然。
- OKF 自身も frontmatter の `tags`（分類）と本文リンク（関係）を別概念として扱う。混ぜるのは OKF の設計とも不整合。

### ユーザー見解
> 関連性（線）が大事なのは分かる。ただ「そのドキュメントが何にカテゴライズされるか」も一定大事では？ どの doc がどれに紐づくかの視覚化に加えて、それが何のカテゴリの中のものか、も見えてほしい。

### AI 再考見解
**見解:** **再考して修正する。tags/category を「ただのフィルタ」と切ったのは過小評価だった。** 「関連性（線）」と「カテゴリ（所属）」は**別軸で、両方を視覚化対象**にする。frontmatter は **relations（エッジ）と category（type＋context）の両方**を持たせ、tags は捨てずカテゴリ軸の一級市民に格上げする。
**根拠:**
- 線（relations）と面/色（所属）は別の認知次元。両方あって初めて「地図」になる。関係だけだと「どの文脈の中の話か」が見えない。
- カテゴリ可視化は OKF も標準（type で色分け）。has-udd では type（spec/skill/agent/knowledge/coding）＝色、context/bounded-context＝クラスタ（Cytoscape の compound node で囲む）の2段で表現できる。
- has-udd の文脈境界は重要な構造。「この doc はどの bc の中か」を見せるのは設計把握に効く。
- 具体的な描画方法（compound node 等）は viewer（#33）の領域なので、本ブレストは「frontmatter が category を持つ」ことの合意までに留める。

### 合意決定
**決定:** **frontmatter は relations（エッジ）と category（type＋context）の2軸を持つ。** tags は category 軸として残す（フィルタ専用に格下げしない）。
**理由:** ユーザー指摘で「所属の視覚化」が要件として確立。線と所属は別軸＝両方を frontmatter が担保すべき。描画手段は viewer 設計（#33）へ送る。

---

## 論点 2: relations の「源泉」はどこか（手書きするか、既存 ref から生成するか）

### AI 初期見解
**見解:** **relations の唯一の源泉＝has-udd の既存構造化 ref（aggregateRef / knowledgeRefs / skillRefs）。** frontmatter の `relations:` も本文の md リンクも、**両方とも render が ref から機械生成する派生物**とし、手書きはしない。
**根拠:**
- **Harness 原則**: 関係（構造）は engine が持つ。frontmatter に手書き relations を足すと source（document.json の ref）との二重管理になり drift する。ref を single source of truth にすれば矛盾が出ない。
- 出し分けの理由は明確：**本文 md リンク**＝OKF 標準ツール互換（散文リンクを辿れる）／**frontmatter relations**＝自前 viewer が種別付きエッジを描くため。同じ ref から両方 emit すれば常に一致する。
- 実装が軽い：#31 は「x-frontmatter に relations を宣言し、`_resolve_path` で ref を引いて生成する」だけに落ちる（新規概念ゼロ）。

### ユーザー見解
> 対象は spec だけの話か？ Document 集約された物すべて（knowledge / agent / skill など）が関連するはず。spec 以外をあえてやらない、という意味なのか？

### AI 再考見解
**見解:** **ご指摘どおり、源泉は「全 document 型の構造化 ref フィールド」に一般化する**（spec の aggregateRef は一例にすぎない）。**x-frontmatter が型ごとに「どのフィールドが relations か」を宣言し、render が一律に走査して frontmatter relations と本文 md リンクを生成**する。spec 限定は PoC のサンプル都合で、意図的除外ではない。
**根拠:**
- Document が集約＝全型が同じ render パイプラインに乗る（scaffold/validate/render が既に型横断で動く実証あり）。relations 生成も型横断の1機構で済む。
- 各型に既に ref がある（skill→knowledge / agent→skill,knowledge / knowledge→knowledge / spec→spec / coding→spec,stack）。束ねれば .has-udd 全体が1つの知識グラフ＝OKF の「横断俯瞰」の本体。
- ただし **ref の在処が2種**ある（前提の表）：構造化フィールド（機械抽出が確実）と、本文 `[[name]]`・コード `@spec` アンカー（別経路の抽出が要る）。この差は論点4で扱う。

### 合意決定
**決定:** 保留（論点4の決着とセットで確定）。方向性＝**全 document 型を対象に、構造化 ref を源泉に render が relations を生成**。
**理由:** ユーザー指摘で対象スコープが「全 Document」に拡張され、源泉一般化の方向は合意。残りは「本文/コード埋め込み ref をエッジに含めるか」（論点4）。

---

## 論点 3: 関係種別（relations[].type）の語彙をどう定めるか

### AI 初期見解
**見解:** **has-udd の ref 種別をそのまま type に写す**（`aggregateRef` / `knowledgeRef` / `skillRef` / `contains` …）。**閉じた語彙（schema 由来の enum）**にし、自由文字列にはしない。
**根拠:**
- 種別が閉じていれば viewer がエッジを**色／線種で描き分け**でき、機械可読。OKF の弱点「関係種別は散文＝曖昧」を has-udd 側で補える（＝OKF 適用の合意済みの強み「内部は rich」を体現）。
- 語彙の出どころは既にある：ref の種類は schema が定義済み。**新しい語彙を発明しない**（最小依存・既存資産流用の方針と一致）。
- 閉じた語彙は **validate できる**＝未知の関係種別を drift として検出できる（保守ループに乗る）。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 4: frontmatter の外にある ref（コード `@spec` / `@stack` アンカー）をエッジに含めるか

### AI 初期見解（★ユーザー指摘で前提を訂正）
**初期の誤り:** 「本文 `[[name]]`（knowledge）が frontmatter の外」と書いたが**これは誤り**。`[[name]]` は AI のメモリ機構（MEMORY.md）の記法で、has-udd の knowledge document とは無関係。**has-udd の knowledge は document.json＝knowledgeRefs は構造化フィールド**。
**訂正後の見解:** **document.json になっている型（spec/skill/agent/knowledge/coding）の ref はすべて構造化済み → frontmatter に昇格するだけ**（本文パース不要）。**document.json の外にある ref は coding の `@spec`/`@stack` アンカー（生成コードの DocComment 内）のみ**。よって論点は「コードアンカー由来のエッジ（code→spec/stack）を含めるか／いつ含めるか」に縮小する。提案＝**段階分け**：document.json の構造化 ref は全型まとめて即エッジ化／code アンカーは reconcile（ripgrep 逆引き）経由で後段。
**根拠:**
- document.json の構造化フィールドは `_resolve_path` で確実に機械抽出＝render の既存仕組みだけで完結（#31 の低コスト範囲）。型をまたいで一律に処理できる。
- code アンカーは frontmatter の外＝reconcile が ripgrep で schema 適合 JSON に逆引きする既存設計の領域。そこからエッジを作れるが render でなく reconcile の責務。
- 段階を分けても最終グラフは同じ（ノード ID＝documentId / コード単位で合流）。先に「document 全型の確実な骨格グラフ」を出し、code→spec エッジは増分で足す方が安全。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

<!-- 合意後に各論点へ「AI 再考見解」「合意決定」を追記する -->
