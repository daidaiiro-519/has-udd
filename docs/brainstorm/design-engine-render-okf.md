# render engine の OKF 対応「設計」ブレスト（#34）

**目的:** OKF を render の出力対象にする設計を確定する。戦略合意（OKF 互換 emit＋自前 viewer＋core 不変・fork 禁止）は `brainstorm-okf-has-udd.md` で CLOSED。本 doc はその **HOW（render engine 設計）**。`design-engine-render.md` と同格。
**前提（戦略合意・不変）:** バンドル形式は OKF v0.1 互換／viewer は自前（Cytoscape+marked+mermaid+独自CSS）／source(document.json)・query・schema は触らない（rendered 層のみ）。

---

## 全体像（これを掴めば論点が分かる）

### いま render がやっていること（document 単位）
```
1つの document.json  ──render──▶  1つの成果物（SKILL.md / HTML / .feature）
```

### OKF 対応で増えること（ディレクトリ＝バンドル単位）
```
.has-udd/documents/specs/        ──render --format okf 〔ディレクトリを渡す〕──▶   specs.okf/
  ├ uc-render-document.json                                                       ├ index.md            ← 目次(RO-2)
  ├ dm-document.json                                                              ├ uc-render-document.md ← frontmatter＋md本文(RO-4,A)
  └ ...                                                                           ├ dm-document.md
                                                                                  └ (graph.html)        ← Cytoscapeグラフ(RO-5,B)
```

### 出てくる .md は具体的にこう（uc-render-document.md・実データ）
```markdown
---                                         ← ここが frontmatter【RO-4＝段階A・新しいのは実質ここだけ】
type: usecase                               #  type/title/description/tags/timestamp を x-frontmatter から
title: uc-render-document
description: 検証済み Document を成果物に描画し配置先へ反映する。
tags: [context:has-udd-engines]
relations:                                  ← 関係の種類を構造で保持【RO-3】
  - { type: aggregateRef, to: dm-document }
---
# uc-render-document                         ← ここから下は md 本文【既存 part_renderer で出る・段階A】
## 主成功フロー
``` mermaid
sequenceDiagram
  Orchestrator->>Document: render する
  ...
``` 
対象集約: [dm-document](/dm-document.md)      ← OKF 互換のクロスリンク【RO-3】

### 段階A / 段階B の切り分け（難易度が違う）
| | 中身 | 関係する論点 | 規模 |
|---|---|---|---|
| **段階A: OKF 互換 md を出す** | frontmatter（↑の `---`）＋ md 本文（既存） | **RO-4** | 小（あなたの「フロントマターの話」） |
| **段階B: グラフ可視化まで** | クロスリンク＋バンドル＋graph viewer | RO-1/2/3/5 | 中（出力単位が"バンドル"に変わる） |

→ **各論点は上の絵のどこを決めるか**です（RO-1=ディレクトリを渡す入口 / RO-2=index.md と中身 / RO-3=relations とリンク / RO-4=frontmatter / RO-5=graph.html と format の置き場）。

---

| # | 論点 | 状態 |
|---|---|---|
| RO-1 | 出力単位とトリガ（単一 document → バンドル） | 議論中 |
| RO-2 | バンドル構成の決定（含める document・index.md/log.md） | 議論中 |
| RO-3 | cross-link 解決（ref→md リンク・関係種別の持ち方） | 議論中 |
| RO-4 | frontmatter マッピング（OKF 最小集合 ↔ x-frontmatter） | 議論中 |
| RO-5 | graph viewer の構造と format 体系（md/html/feature と okf/graph の共存） | 議論中 |

---

## 論点 RO-1: 出力単位とトリガ（単一 document → バンドル）

### AI 初期見解
**見解:** render を **「バンドルモード」に拡張**。トリガは **対象がディレクトリ（文脈）か単一ファイルか**で分岐（query の `scan`(単一) vs `index_scan_dir`(横断) と同じ二分法）。schema は単一 document の型定義のままで、バンドルは render の上位集約処理。
**根拠:**
- query が既に「ファイル→単一／ディレクトリ→横断」で分岐しており、render も同じ二分法が一貫・直感的。
- バンドル＝「ディレクトリ/文脈」が単位（OKF も dir）。`render --path <dir> --format okf` で配下を OKF バンドルに。
- 既存の document-level x-render-target はそのまま使い、bundle は別レイヤ（後述 RO-5）。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 RO-2: バンドル構成の決定（含める document・index.md/log.md）

### AI 初期見解
**見解:** **含める document ＝ 対象ディレクトリ配下の全 document（query の `index_scan_dir` で集約）**。`index.md` ＝ その index を md 化（OKF 予約・目次）。`log.md` ＝ 任意（status/updatedAt の履歴）。
**根拠:**
- `index_scan_dir` が既に「ディレクトリ配下を schema 駆動で集約」する＝素材集約を**流用**（新規実装不要・core 不変の query をそのまま使う）。
- フォルダ＝文脈境界（SP-2「メンバーはフォルダ由来」）と一致。
- index.md は各 document の type/title/description が並ぶ目次（OKF consumer も使える）。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 RO-3: cross-link 解決（ref→md リンク・関係種別の持ち方）

### AI 初期見解
**見解:** **構造化 ref（aggregateRef / knowledgeRefs）→ OKF の bundle 相対 md リンク `/path.md`**（本文 or frontmatter）。**関係種別は frontmatter 拡張 `relations:` に構造で載せ**、本文リンクは OKF 互換に保つ（種別は OKF の「拡張 k-v 任意」で両立）。
**根拠:**
- OKF は「リンク＝関係あり・種類は散文」で曖昧。has-udd は種別を機械可読に保ちたい → **frontmatter 拡張 `relations: [{type: aggregateRef, to: dm-document}]`** に構造で持ち、OKF 標準リンクも併記。
- documentId ＝ バンドル内ファイル名 → `to`→path 解決は素直（OKF concept ID＝path−.md と一致）。
- これで OKF 標準ツールはリンクを辿れ（互換）、has-udd/自前 viewer は relations から**正確な種別グラフ**を描ける。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 RO-4: frontmatter マッピング（OKF 最小集合 ↔ x-frontmatter）

### AI 初期見解
**見解:** **各 schema の x-frontmatter に OKF 標準フィールドを宣言**（`_resolve_path` の既存仕組みで足すだけ）。`type`=discriminator（specKind/skillKind/codingKind）、`title`=documentId、`description`=purpose.text、`tags`=tags、`timestamp`=updatedAt。
**根拠:**
- x-frontmatter は既に「フィールド→ドットパス」宣言（engine 汎用のまま OKF フィールドを追加できる・#31 の低コスト作業）。
- `type` の粒度は **discriminator** が最も意味的（OKF の type＝概念の種類と一致・グラフ色分けに有用。documentType(Spec/Skill) より細かい）。
- 既存値とほぼマップ可。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 RO-5: graph viewer の構造と format 体系（md/html/feature と okf/graph の共存）

### AI 初期見解
**見解:** **format を2層に分ける**。**document-level**（md/html/feature・単一 doc 出力・既存のまま）と **bundle-level**（`okf`=md バンドル emit／`graph`=自前 viewer HTML）。bundle は render-target でなく**render の bundle 処理**として扱う（出力単位が違うため formats 配列に混ぜない）。graph viewer ＝「bundle を JSON 化してテンプレに埋込んだ自己完結 HTML（Cytoscape+marked+mermaid+独自CSS・全部 CDN）」。
**根拠:**
- 既存 formats（md/html/feature）は単一 document 出力。okf/graph は**バンドル出力**＝単位が違うので、同じ formats 配列に並べると意味が壊れる → 別レイヤが整合的。
- graph viewer は既存の「HTML エンベロープ（mermaid.js 入り自己完結 HTML）」の延長＝Cytoscape を足すだけ。bundle JSON は `index_scan_dir`＋各 doc の frontmatter/refs から作る。
- okf(md) と graph(HTML) は**同じ bundle データから両方出せる**（part_renderer の md ＋ viewer テンプレ）。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_
