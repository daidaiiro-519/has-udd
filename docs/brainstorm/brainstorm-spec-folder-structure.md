# ブレインストーミング: specフォルダ構成をDDD準拠の入れ子構造に是正する

**目的:** `.waffle/documents/specs/`配下のフラットな12ファイル構成を、既存の合意（`project-spec-schema.md` SP-2・CLOSED）が示す「bounded-context単位の入れ子」構造へ是正する。
**モード:** 問題解決

---

## 問題の再定義

**SP-2で既に合意されていた「フォルダ＝文脈境界」という入れ子構造が、bootstrap時の先送り判断のまま実行されず、specKindが4種（+PresentationSpecSchemaの2種）に増えた今もフラット12ファイルのままになっている。**

## 原因候補

- **表面的原因**: SP-2策定時に「ブートストラップはフラットパスでよい・入れ子は後段」と明示的に先送りされ、そのまま忘れられていた。
- **根本原因**: SP-2策定時（specKind=bounded-context/domain-model/usecaseの3種）から、Re-5でsubdomainが追加され4種に、さらに今回PresentationSpecSchemaで screen/flow の2種が加わり、**フォルダを分けないことのコスト（1階層に種別の異なるファイルが混在する読みにくさ）がSP-2策定時より明確に上がった**のに、フォルダ構成だけが据え置かれていた。

---

## 論点 1: 入れ子構造を今実行すべきか、複数contextが実際に出てくるまで待つべきか

### AI 初期見解
**見解:** 実行してよい。ただし「複数contextの恩恵」を主目的にはせず、「**同一context内でのkind別整理**」を主目的とする方が今の実態に合う。

**根拠:**
- 現状bounded-contextは`bc-waffle-engines`1つのみ。「フォルダ＝文脈境界」という当初の主目的（複数contextの分離）はまだ効かない。
- しかし今回、DomainSpecSchema（4 kind）とPresentationSpecSchema（2 kind）が合流し、1つのcontext配下だけで既に6種類のkindが混在する状況になった。**同じフォルダに`agg-*`・`sd-*`・`uc-*`・（将来）`scr-*`・`flow-*`が全部並ぶのは、文脈が1つしかなくても読みにくい**。
- 実行コストは低い（12ファイルのmv＋path templateの調整のみ・データそのものは変わらない）。今回のDomainSpecSchema改名と同様、「保留していたコスト低い決定は、思い出した時に片付ける」の方針に合う。

### ユーザー見解
> （実行に合意。同一context内でのkind別整理という理由づけで進める）

### 合意決定
**決定:** 今のタイミングでフォルダ入れ子構造を実行する。
**理由:** 複数context分離の恩恵はまだ効かないが、6 kind（DomainSpecSchema4種＋PresentationSpecSchema2種）が1フォルダに混在する読みにくさは既に実害。実行コストが低い（12ファイルのmv＋pathテンプレート調整のみ）。
**次のアクション:** 論点2〜4を決めてから一括実行する。

---

## 論点 2: 具体的なフォルダ名・階層

### AI 初期見解
**見解:** SP-2の`{context}/{context.json, domain-model/, usecase/}`案を、現行4 kind（＋PresentationSpecSchemaの2 kind）に合わせて次のように更新する。

```
.waffle/documents/specs/
  bc-waffle-engines/
    bc-waffle-engines.json        ← bounded-context 自身はcontextフォルダ直下（1文脈=1つなので特別扱い）
    aggregate/
      agg-document.json
      agg-schema.json
    subdomain/
      sd-harness-core.json
      sd-harness-core/
        uc-scaffold-document.json
        uc-query-document.json
        uc-scan-source-code.json
        uc-lint-docstring.json
      sd-validation.json
      sd-validation/
        uc-validate-document.json
      sd-rendering.json
      sd-rendering/
        uc-render-document.json
```

**根拠（初期案から修正）:**
- SP-2の`domain-model/`は現行の`aggregate`kindに相当（Re-5でdomain-model→aggregateに改称済み）ので追従してリネーム。
- **usecaseは`subdomainRef`を持ち必ず1つのsubdomainに属するため、subdomainの子フォルダとして入れ子にする**（初期案の「subdomain/と並列のusecase/」は実データの参照構造と食い違っていたため修正）。
- **aggregateはsubdomainの子にしない**——`agg-document`は`sd-harness-core`/`sd-validation`/`sd-rendering`全ての usecase から`aggregateRef`で参照されており、特定の1 subdomainに属する概念ではなくbc全体で共有される中核集約であることを実データで確認した。bcレベルの兄弟フォルダのままが正しい。
- `bounded-context`だけはフォルダの直下に置く特別扱い（1 context = 1個しか無いためサブフォルダを切る意味が無い）。

### ユーザー見解
> ユースケースフォルダは作成してください（＝usecaseをsubdomainの子として入れ子にする修正版に合意）。

### 合意決定
**決定:** `aggregate/`はbcレベルの兄弟フォルダ、`usecase`は各`subdomain`の子フォルダとして入れ子にする（上記修正版のツリー構成で確定）。
**理由:** 実データの参照構造（usecase.subdomainRef・aggregateRefの実際の使われ方）に基づく検証で、aggregateはbcレベル共有・usecaseはsubdomain子、という非対称な構造が正しいと確認できたため。
**次のアクション:** 論点3（パステンプレート機構）・論点4（PresentationSpecSchemaの配置）を続けて検討する。

---

## 論点 3: x-source-target/x-render-targetのテンプレート機構をどう対応させるか

### AI 初期見解
**見解:** パステンプレートに`{contextRef}`という新しい変数を導入し、**scaffold create時に明示的に渡す入力**とする（documentの`content`には保存しない＝SP-2の「所属文脈は保存しない」原則を維持しつつ、パス構築にだけ使う）。

**根拠:**
- 現在の`x-source-target`は`.waffle/documents/specs/{documentId}.json`という単一テンプレート。入れ子にするには`{contextRef}`と`{specKind}`（サブフォルダ名）をテンプレートに追加する必要がある: 例 `.waffle/documents/specs/{contextRef}/{specKind}/{documentId}.json`（bounded-context自身は`{contextRef}`＝自分の`documentId`なので`.waffle/documents/specs/{documentId}/{documentId}.json`という特殊系になる）。
- 「所属文脈をどこで知るか」が課題: SP-2は「メンバーは保存しない・フォルダ由来」としたが、**新規作成時に何らかの形でどのcontextに属するかをAIが指定しないと、そもそも配置先フォルダが決まらない**（フォルダに書く前にフォルダを知る必要があるという循環）。よって「メンバーシップの参照」はcontentに保存しないが、**scaffoldの入力パラメータ（discriminatorに準ずる扱い）としては必要**、というのが筋が通る。
- 代替案（採用しない）: bcのMembers一覧から逆引きしてパスを決める、という案も考えたが、create時点ではまだbcのMembersにも登録されていない（鶏と卵）ので不採用。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---

## 論点 4: PresentationSpecSchema（screen/flow）の配置

### AI 初期見解
**見解:** 同じ`{contextRef}/`配下に、`subdomain`/`aggregate`/`usecase`と並ぶ形で`screen/`・`flow/`サブフォルダを追加する（documentType(DomainSpec/PresentationSpec)が違っても、所属する bounded-context は同じ）。

**根拠:**
- PresentationSpecSchemaのDocumentも「同じ製品文脈（bounded-context）に属するUI」であることは変わらない。documentTypeが違うことは「別のcontextに属する」ことを意味しない。
- 対称性: DomainSpecSchemaの4 kindとPresentationSpecSchemaの2 kindを、同じ階層原理（kind名＝サブフォルダ名）で扱えば、スキーマが増えてもフォルダ構成のルールは変わらない。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

---
