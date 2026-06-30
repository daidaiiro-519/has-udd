# ブレインストーミング: OKF（Open Knowledge Format）の has-udd への適用

**目的:** OKF の優れた点（概念グラフ可視化・最小 frontmatter・寛容な消費・クロスリンク）を has-udd にどう取り込むか、また OKF と相互運用すべきかを探索し、取り込む範囲を決める。
**モード:** アイデア発散

> OKF v0.1 要約（調査済み）: markdown ディレクトリ＋YAML frontmatter（必須は `type` のみ／推奨 title/description/resource/tags/timestamp／拡張任意）。本文の普通の markdown リンクで概念をクロスリンク（**関係の種類は周辺の散文**で表す・壊れたリンク許容）。予約 `index.md`(目次)/`log.md`(更新履歴)。適合は最小・**寛容な消費**。CLI `visualize` で自己完結 HTML（**Cytoscape.js 力学グラフ**・type 色分け・クロスリンクのエッジ・**"Cited by" バックリンク**・検索/フィルタ）。

---

## アイデアダンプ

1. render に「バンドルグラフ HTML」モードを足す（Cytoscape.js・documents=ノード/refs=エッジ）
2. query の index_scan_dir → グラフ用 JSON を集約生成（既存機能の流用）
3. has-udd document → OKF export（md＋frontmatter）を render の1フォーマットに
4. x-frontmatter に OKF 標準フィールド（type/title/description/tags/timestamp）を採用
5. aggregateRef / knowledgeRefs → グラフのエッジ＋"Cited by" バックリンク
6. 「関係の種類」を構造（ref の種類）or frontmatter 拡張で機械可読に保持
7. OKF の index.md ＝ has-udd の動的 index（index_scan_dir）の render 出力
8. log.md ≈ status lifecycle / updatedAt の履歴を render
9. 外部 OKF bundle を query の raw / 寛容 import 経路で消費
10. **Spec ツリー（bc→dm→uc aggregateRef）をグラフで俯瞰** → ドメインモデル理解に直結
11. メモリ（[[name]] リンク）を OKF bundle として visualize
12. 「内部＝厳格 schema／外部 I/F＝OKF 寛容」の二層（腐敗防止層）
13. OKF を render-target の新 format（"graph" / "okf"）として宣言的に足す
14. knowledge 集約 schema を OKF 互換に設計（type は producer 定義）

**絞り込み候補（実現性×インパクト 上位）:**
- **①バンドルグラフ可視化**（render の graph format）= インパクト最大・素材が既に揃う
- **④frontmatter を OKF 標準に整合** = 低コスト・ポータビリティ獲得
- **③export（has-udd→OKF）** = 相互運用・deploy 思想（多ツール配布）と同方向
- **⑥関係の種類は"構造"で維持**（OKF の散文には倒さない）= Harness 思想を守る前提

---

## 論点 1: 概念グラフ可視化を has-udd に足すか／どの形で？

### AI 初期見解
**見解:** 取り込む価値が高い。ただし新 engine ではなく **render engine の HTML 出力に「バンドルグラフ」モード（render-target の新 format）** を足すのが筋。
**根拠:**
- has-udd は既に型付き documents と構造化リンク（aggregateRef / knowledgeRefs）を持つ＝**グラフ化の素材が揃っている**。Spec ツリー（bc→dm→uc）がそのままグラフになり、ドメイン理解に直結。
- OKF の visualize は「bundle を JSON 埋め込み＋Cytoscape.js の自己完結 HTML」。has-udd も **query の index_scan_dir で集約 → JSON 化 → グラフ HTML** で同型に作れる（集約は query が担当・render が描画）。
- ただし「ノード＝単一 document の描画」と「グラフ＝バンドル横断の俯瞰」は出力レベルが違う。後者は **render-target が単一 document でなく『ディレクトリ/コンテキスト』単位**になる新しい出力＝ここが設計判断。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 2: has-udd document ⇄ OKF の相互運用は要るか／どの方向で？

### AI 初期見解
**見解:** **export（has-udd→OKF）は価値高・import は慎重**（生 OKF は query の raw 扱いに留める）。
**根拠:**
- export: has-udd document を OKF（md＋frontmatter）に落とせば、他エージェント/ツール（Obsidian/MkDocs/他 LLM）が has-udd の知識を消費できる＝ポータビリティ。has-udd の deploy 思想（多ツール配布）と同方向で、render の1フォーマットとして自然。
- import: OKF は「type だけ必須・寛容」で構造が緩く、has-udd の「schema で構造を機械担保」と**思想が逆**。OKF を一級市民として import すると検証可能性が薄まる。
- → **source は has-udd document.json のまま・OKF は rendered 成果物レベルの相互運用**（export 中心）が安全。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 3: クロスリンクと「関係の種類」をどう表すか？

### AI 初期見解
**見解:** has-udd は **「関係の種類を構造で持つ」を維持**し、OKF 互換 export では path link＋関係種別を frontmatter 拡張に落とす。
**根拠:**
- OKF は「リンク＝関係あり・種類は散文」＝機械には曖昧。has-udd は aggregateRef（usecase→domain-model）等で**関係の種類が機械可読**＝Harness 思想に合致。ここは退化させたくない。
- メモリの `[[name]]`（人間メモ）と document 間の ID 参照（機械 spec）は**層が違う**ので使い分けてよい。OKF は前者寄り。
- export 時は構造化リンク → OKF の path link に変換し、**関係種別は OKF の「拡張 k-v 任意」を使って frontmatter に載せる**（OKF 互換を保ちつつ種類も残せる）。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 4: frontmatter を OKF 標準集合に整合させるか？

### AI 初期見解
**見解:** **OKF の最小集合（type/title/description/tags/timestamp）を x-frontmatter の"標準フィールド"として採用**し、低コストでポータビリティを得る。
**根拠:**
- 既存とほぼマップ可能: `type`≈documentType/discriminator、`description`≈purpose、`tags` 既存、`timestamp`≈updatedAt、`title`≈documentId。整合コストが低い。
- これらは query/index にも有用（OKF が「クエリ可能フィールド」と位置づけるのと同じ動機）。
- 整合すれば has-udd の rendered md/html が **OKF consumer にそのまま食える**＝相互運用の土台。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 5: knowledge 集約の設計に OKF 思想をどこまで取り込むか？

### AI 初期見解
**見解:** OKF の「**寛容な消費・producer 定義 type・index.md/log.md**」は has-udd と親和的だが、has-udd は **「内部＝厳格 schema／外部 I/F＝OKF 寛容」の二層**で取り込む（検証可能性は捨てない）。
**根拠:**
- OKF「type だけ必須・未知を許容」= has-udd の discriminator＋頑健化（raw フォールバック）と同方向。**外部 import 経路**で OKF 寛容モードを採れる。
- has-udd の中核価値は「schema で構造を機械担保」＝OKF の緩さの逆。だから**境界（腐敗防止層）で OKF を受け、内部は厳格 schema**に変換するのが整合的。
- OKF の `index.md`（目次）＝ has-udd の `index_scan_dir`（保存しない動的算出）と**機能が同じ** → has-udd は「動的 index を OKF の index.md として render 出力」できる。`log.md` ≈ status/updatedAt 履歴。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 6: query engine と OKF/generic の責務分け（検索性）

### AI 初期見解
**見解:** 「発見/全文/メタ検索＋グラフ可視化」は一般領域 → OKF＋既製に委譲（自作しない）。「schema 駆動の意味的構造アクセス（{prompt,value}・get_block/filter・動的 _index）」は has-udd の中核（Harness）で OKF では代替できない。

### ユーザー見解
> 構造的な情報取得は has-udd の query engine の責務。query engine と OKF/generic の責務分けをはっきりさせたい。

### AI 再考見解
**見解:** 責務は **「source の意味アクセス（query・中核）」 vs 「rendered の発見/俯瞰（OKF・一般）」** で層を境界に分かれる。query は痩せも太りもせず意味アクセスに専念し、発見/全文/グラフへ育てない。判断ルール:「どの doc の・どの意味単位が欲しいか分かっていて値で機械的に動く」→ query／「探す・横断・人/AI に俯瞰共有」→ OKF。`index_scan_dir`(query=機械ルーティング用 index) と OKF `index.md`(人間目次) は目的が違い両立。

### 合意決定
**決定:** **query=source層の精密・意味的構造アクセス（中核・自作）／ OKF＋既製=rendered層の発見・全文・関係グラフ（一般・委譲）。境界=source vs rendered。query は全文/グラフ検索に拡張しない。**

---

## ★ セッションまとめ（合意事項）

**総括: 「OKF 互換 emit ＋ 自前 viewer ＋ core 不変。フォーマットは fork しない」。**
OKF の概念・ワイヤ互換は頂くが、viewer と core は has-udd。fork した瞬間に相互運用（OKF 最大のうまみ）が消えるので、**バンドル形式は OKF 標準互換を保つ**。

| 論点 | 合意 |
|---|---|
| 1 グラフ可視化 | **自作 viewer（Cytoscape.js）**で。OKF 標準 visualize には依存しない（独自 CSS・mermaid 図のため）。重い描画は generic lib を再利用 |
| 2 相互運用 | **rendered を OKF 互換 md バンドルで emit（export 中心）**。import は慎重（生 OKF は query の raw 扱い） |
| 3 cross-link | 構造化 ref（aggregateRef/knowledgeRefs）は**維持**。OKF 化時は md リンク＋関係種別を frontmatter 拡張へ |
| 4 frontmatter | **OKF 最小集合（type/title/description/tags/timestamp）を x-frontmatter 標準に採用** |
| 5 knowledge 思想 | **内部＝厳格 schema／外部 I/F＝OKF 寛容（腐敗防止層）**。index.md＝動的 index、log.md≈履歴 |
| 6 検索の責務 | **query＝source の意味アクセス（核）／ OKF＝rendered の発見・俯瞰（一般）** |
| HTML | 自前 HTML（単一 doc の CSS view）→ **「OKF 互換 md ＋ 自前 graph viewer」** に発展。OKF 標準 HTML 単体は独自 CSS/mermaid で詰まる |

**OKF のうまみ（確認）:** 新しい内部能力ではなく、**外側の generic 層（グラフ可視化・検索・ポータビリティ）を肩代わりさせること**。価値が出るのは rendered/外向き層だけ。

---

## ★ 実現方法（実装方針・将来タスク）

### 1. render に `okf` フォーマットを追加（出力単位＝ディレクトリ/文脈・バンドル）
- **本文** = part_renderer の **md 出力**（既存・table/list/section/code/sequence の md 版が既にある）
- **frontmatter** = OKF 最小集合を x-frontmatter で宣言（`type`=specKind/documentType・`title`=documentId・`description`=purpose・`tags`・`timestamp`=updatedAt）
- **cross-link** = aggregateRef/knowledgeRefs → **md リンク `[..](/path.md)`** ＋関係種別を frontmatter 拡張（例 `relations:`）
- **index.md** = query の `index_scan_dir` 由来の目次／**log.md** ≈ status・updatedAt 履歴（任意）
- 出力は OKF v0.1 適合（frontmatter parse 可・`type` 非空・予約ファイル準拠）→ **OKF 標準ツールでも消費可能**

### 2. graph viewer（自前・generic lib 再利用）
- 1枚の**自己完結 HTML** にバンドルを **JSON 埋め込み**
- **Cytoscape.js**（力学グラフ・refs エッジ・Cited by）＋ **marked.js**（md→HTML）＋ **mermaid.js**（図）＋ **独自 CSS**（全部 CDN・既存 HTML エンベロープの延長）
- render の出力フォーマット（例 `graph`）or `okf` の付随出力。＝ OKF 標準 viewer の弱点（独自 CSS 不可・mermaid 出ない）を補う

### 3. core は不変
- document.json（source）・query engine（意味アクセス {prompt,value}）・schema は触らない。OKF は rendered/外向き層のみ。

### 4. 段階（優先度）
1. **frontmatter を OKF 標準に整合**（最小・低コストの足がかり）
2. **md（okf）emit**（part_renderer md ＋ frontmatter ＋ リンク変換 ＋ バンドル化）
3. **graph viewer**（Cytoscape＋marked＋mermaid＋CSS）

### 5. 位置づけ
- Phase 6（multi-tool 互換）/ knowledge 集約設計と合流（OKF＝外部相互運用の標準形）。
- 今は実装せず本 doc に記録。中断中の実装（resume-point）に戻る。

### 次のアクション
- 合意を本 doc に記録（済）。
- 将来タスク化: render `okf` format / graph viewer / frontmatter OKF 整合（優先＝frontmatter 整合）。
- 中断していた実装（uc-detect-drift or Phase5 Hooks）に復帰。

### ⚠️ 実装の前提（層の切り分け）
- 本ブレストは **OKF 適用の「戦略・価値・責務」層（WHAT/WHY）** で CLOSED。
- **`okf` を render の出力対象にする「設計」層（HOW）は別ブレストが必要**（#32 render okf format / #33 graph viewer の前提）。理由: render の出力単位が**単一 document → バンドル（ディレクトリ/文脈）**に変わり、cross-link 解決・index.md/log.md 生成・viewer 構造・既存フォーマットとの位置づけ等、`design-engine-render.md` と同格の render engine 設計論点が立つ。
- `#31 frontmatter OKF 整合`は小規模（フィールド対応のみ）＝設計ブレスト不要で着手可。
