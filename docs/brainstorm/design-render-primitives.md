# render 部品セット選定（ステップ1: Markdown 記法 × HTML タグ 洗い出し）

## 目的

x-render を「生 Jinja（block ごと）」から「**宣言的・共通部品**」へ移すため（(b) 案）、まず
**Markdown で表現できる記法**と**それに対応する HTML タグ**を網羅的に洗い出す。
ここから「has-udd の render 部品セット」を選定する（次ステップ）。

前提: has-udd は **md（SKILL.md / AGENTS.md 等・ツールが読む）** と **html（specs/knowledge の人間向け）** の両方を生成する。
→ 部品は **md と html の両方に綺麗に落ちる**ものを選ぶ。Markdown は CommonMark + GFM（GitHub Flavored）を基準とする。

---

## 1. ブロックレベル（構造）

| 表現 | Markdown 記法 | HTML タグ | 備考 |
|---|---|---|---|
| 見出し | `#`〜`######` | `<h1>`〜`<h6>` | レベル6まで |
| 段落 | 空行区切りのテキスト | `<p>` | 基本 |
| 箇条書き（順序なし） | `- ` / `* ` / `+ ` | `<ul><li>` | |
| 箇条書き（順序つき） | `1. ` `2. ` | `<ol><li>` | |
| ネストリスト | インデント（2/4スペース） | `<ul>`/`<ol>` の入れ子 | |
| タスクリスト | `- [ ]` / `- [x]` | `<ul><li><input type="checkbox">` | GFM |
| 定義リスト | （CommonMark 非対応・一部拡張 `term`改行`: def`） | `<dl><dt><dd>` | md 弱い→html が正 |
| テーブル | `\| col \|`＋`\|---\|` | `<table><thead><tbody><tr><th><td>` | GFM |
| 引用 | `> ` | `<blockquote>` | ネスト可 |
| コードブロック | ` ```lang ` … ` ``` ` | `<pre><code class="language-">` | 言語指定可 |
| 水平線 | `---` / `***` | `<hr>` | |
| 画像 | `![alt](src)` | `<img>` | |
| 折りたたみ | （md 非対応・raw html 併用） | `<details><summary>` | md では生 html を埋める |
| 注記/警告（callout） | `> [!NOTE]` `> [!WARNING]` 等 | `<div class="note">` / `<aside>` | GFM(alert)・限定的 |
| 脚注 | `[^1]` … `[^1]: …` | `<sup><a>`＋`<section>` | 拡張 |
| 数式（block） | `$$ … $$` | `<math>` / MathJax | 拡張 |

---

## 2. インラインレベル（装飾）

| 表現 | Markdown 記法 | HTML タグ | 備考 |
|---|---|---|---|
| 太字 | `**text**` / `__text__` | `<strong>` | |
| 斜体 | `*text*` / `_text_` | `<em>` | |
| 太字＋斜体 | `***text***` | `<strong><em>` | |
| 打ち消し | `~~text~~` | `<del>` / `<s>` | GFM |
| インラインコード | `` `code` `` | `<code>` | |
| リンク | `[text](url)` | `<a href>` | |
| 自動リンク | `<url>` | `<a>` | |
| 改行（強制） | 行末2スペース / `\` | `<br>` | |
| ハイライト | `==text==`（一部拡張） | `<mark>` | md 弱い |
| 上付き/下付き | `^x^` / `~x~`（拡張） | `<sup>` / `<sub>` | md 弱い |
| 絵文字 | `:smile:`（GFM） | （unicode/img） | |
| インライン数式 | `$x$` | MathJax | 拡張 |

---

## 3. ドキュメントレベル

| 表現 | Markdown | HTML | 備考 |
|---|---|---|---|
| フロントマター | `---` YAML ブロック（先頭） | `<head>` meta（html では別扱い） | **SKILL.md で必須**（name/description）。md 固有・html では head/不要 |
| 目次(TOC) | 自動生成 or 手書きリンク | `<nav>` | 生成系 |
| セクション構造 | 見出しの階層で暗黙 | `<section>`/`<article>` で明示 | html は構造を明示できる |

---

## 4. Markdown に「無い / 弱い」が HTML にあるもの（要注意）

md と html のギャップ。部品化で **md では raw html を埋める or 代替表現**が要るもの:

| 表現 | md の状況 | 対応方針候補 |
|---|---|---|
| 折りたたみ `<details>` | 非対応（生 html 埋め込みは可） | html=タグ / md=生html or 通常見出しに展開 |
| 定義リスト `<dl>` | 非対応〜拡張依存 | md=テーブル or 「**term**: def」リストで代替 |
| callout/admonition | GFM alert のみ・限定 | html=`<aside>` / md=`> [!NOTE]` or 引用で代替 |
| ハイライト `<mark>` | 拡張依存 | md=太字で代替 など |
| 上付き/下付き | 拡張依存 | 用途が出たら検討 |
| 任意の class/style | 不可 | html のみ・md は無視 |

→ **部品は「md と html 両方に確実に落ちる」ものを基本セットに、ギャップのあるものは html 優先＋md 代替**で扱う方針が要る。

---

## 5. 観察（部品セット選定に向けて）

- **両対応が確実な基本**: heading / paragraph / list(ul,ol) / table / code / blockquote / hr / link / 太字・斜体・インラインコード
- **構造（入れ子）に必要**: section（見出し＋子・steps/substeps 用）
- **キー:値**: definition list は md 弱い → table か「**k**: v」リストで代替
- **callout/details**: あると表現力↑だが md ギャップ大 → 後回し or html 優先
- has-udd の現用途（Skill/Spec/Knowledge）で実際に使うのは: **見出し・段落・箇条書き・テーブル・コード・section（入れ子）・キー値・インライン装飾** が中心

---

## 次のアクション

この一覧から **has-udd の render 部品セット（採用する primitive と、その md/html 出力定義）** を選定する（ステップ2）。
選定基準: (1) md/html 両対応が確実 (2) 現用途で実際に使う (3) 宣言的に値を流せる形に落とせる。
