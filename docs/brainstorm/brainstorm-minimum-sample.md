# ブレインストーミング: 動く最小サンプル（examples/）

**目的:** CodingSchema（規約＝決定ルール）と対をなす「正確な形」の担い手＝動く最小サンプル（examples/）を、**何を・どう作るか（プロセス込み）**で確定する。
**モード:** 設計判断（論点 → 見解 → 合意）

**確定前提（揺らさない）:**
- 規約＝決定ルール（Layers/Layout/ConceptPlacement/Rules 等）／正確な形（署名・結線コード）＝サンプルが担う、という分担（[[project-coding-schema-stage-k]]）。
- has-udd 自身が python-hexagonal の実例＝ドッグフード対象そのもの。
- 詳細設計はコードに宿る・コードから導けない意図だけが spec（[[project-ai-era-detail-design]]）。サンプルは「意図の説明」ではなく「規約の体現」。

---

| # | 論点 | 状態 |
|---|---|---|
| MS-1 | サンプルは何を対象にするか（スコープ・題材） | ✅ CLOSED |
| MS-2 | サンプルはどう作るか（プロセス：ガイド→実装の順序と手順） | ✅ CLOSED |
| MS-3 | サンプルの置き場・構成・命名 | ✅ CLOSED（MS-4 合格基準1に統合） |
| MS-4 | サンプルの「正しさ」をどう確認するか（効果測定との関係） | ✅ CLOSED |

---

## 論点 MS-1: サンプルは何を対象にするか

### AI 初期見解
**見解:** **has-udd 自身のコードをサンプルとして指定する**（新規に別サンプルを書き起こさない）。

**根拠:**
- coding-standard 後イメージで既に「正確な形は examples/ を手本に」としているが、has-udd は自分自身の python-hexagonal 実装を持つ（`src/has_udd/`）。二重に書くと片方が腐る（drift の温床）。
- 「動くサンプル」の条件（1スライス：spec→コード→緑テスト）は既に満たされている——`uc-render-document` の spec（[[project-coding-schema-stage-k]] 参照）と `render_engine.py`・関連テストが実在する。
- ただし**指し方**が問題：architecture 規約は「walking skeleton を examples/ に」と書いたが、has-udd はサンプルを `examples/` ディレクトリとして別置きしていない（自分自身が対象なので当然）。→ **規約側の文言を「examples/ または当該スタックの実装そのもの」に補正**するか、あるいは他プロジェクトへの配布時だけ `examples/` を用意する、を決める必要がある。

### ユーザー見解
> このサンプルは CodingSchema が全て確定して初めて書けるもの。CodingSchema から作成されたインスタンスがベースとなって、そのアーキテクチャで動作する完全な見本サンプルをガイド付きで作成するイメージ。

### AI 再考見解（撤回＋再構成）
**見解:** 初期見解「has-udd 自身を指せばいい」は撤回する。正しい構図はこう:

```
【前提・確定済み】CodingSchema instance ×4（python-hexagonal）
   ↓ render（既存機構）
【ガイド】規約 .md ×4（人/AI が読む「どう作るか」の説明書）
   ↓ ＋ 小さな DomainSpec（「何を作るか」・例: 1 usecase + 1 aggregate）
【生成】完全な見本サンプル（新規プロジェクト。ガイドだけを見て一から実装）
   ↓
【検証】ガイドだけで矛盾なく・過不足なく実装できたか
```

**has-udd 自身を指すのが誤りだった理由:**
- has-udd の実装は**規約と共進化**してきた（規約を確定してから作ったのでなく、行き来しながら固めた）。だから「規約だけを見て正しく作れるか」の証明にならない——**カンニング**に近い。
- 「動くサンプル」の価値は**規約の自己充足性を実地検証すること**（architecture 確定時に行った自己テスト「uc-place-order を実装できるか」の、プロジェクト規模への拡張）。だから被験体は**規約確定後に初めて着手する新規コード**でなければならない。
- ★これで MS-1 の問い自体の前提が明確になった：**「サンプルは CodingSchema が確定して初めて書ける」＝順序の制約**。今まさに4 codingKind が確定した直後なので、着手可能な状態になった。

**サンプルの構成要素（初期・後に3.5.に整理）:**
1. CodingSchema instance ×4
2. render 済みガイド（.md ×4）
3. 小さな DomainSpec（新規）
4. 見本サンプル本体
5. 付随ガイド（対応表）

### ユーザー見解（続）
> 正直これは全ての規約通りに動作する構造とアーキテクチャ、命名規則や実装 pattern などが分かればいいと思ってる。あと、ガイドって coding schema のインスタンスがもはやガイドなのでは？（もっとブレイクダウンした実際の実装ガイド、という意味ではなく）

### AI 再考見解（★最終・要素を2つに削減）
**見解:** 3（DomainSpec）と5（付随ガイド）を撤回する。

- **5（付随ガイド）を撤回する理由**: CodingSchema instance の render 済み .md が**既にガイドそのもの**。これとは別に「対応表」を挟むと、規約とサンプルの間に**もう1段の翻訳文書**を作ることになり、[[project-ai-era-detail-design]] D-2 で撤回したはずの「詳細設計書の再発明」に逆戻りする。規約かサンプルのどちらかが変わった瞬間に腐る第3の成果物は作らない。
- **3（DomainSpec）を撤回する理由**: ユーザーの目的は「**規約通りに動作する構造・命名・実装パターンが分かること**」＝**形式検証**であって、業務としての意味検証ではない。ConceptPlacement の各 concept を**総当たりでプレースホルダ実装**すれば足り、業務らしい題材（注文・顧客等）は不要。むしろ業務題材にすると「意味が正しいか」に気を取られ、目的（構造の検証）から焦点がぼやける。

**サンプルの構成要素（確定・2つ）:**
1. **CodingSchema instance ×4（＝ガイド。既にある）**
2. **見本サンプル本体**：ConceptPlacement の全 concept をプレースホルダ題材で総当たり実装したコード＋最小テスト

### ユーザー見解（続2）
> まあ、動くものというイメージだからなんらかの視覚的にわかるものがいいと思うよ。

### AI 再考見解（視覚化・★汎用化の訂正込み）
**見解（一度 has-udd 特化に寄った点を訂正）:** 「CLI で実行して結果を見る」は has-udd 自身の Interface（CLI/MCP）に引きずられた提案で誤り。**このブレストは has-udd 自身のアーキテクチャに限った話ではなく、has-udd を介してエンドユーザーが指定した任意のアーキテクチャ・言語・規約についての話**（ユーザー確認済み）。よって正しくは:

> **tech-stack instance の Interface 規約が定める様式で実行し、結果を確認できること**（CLI なら標準出力、REST ならレスポンス、Web UI なら画面）。様式は対象 tech-stack instance が決めるのであり、has-udd 側は決めない。

### 合意決定（MS-1）
**決定:** サンプルの構成要素は**2つ**（CodingSchema instance ×4＝ガイド／見本サンプル本体＝ConceptPlacement 全 concept のプレースホルダ総当たり実装）。DomainSpec・付随ガイドは作らない。視覚化（動作確認の様式）は対象スタックの tech-stack instance の Interface 規約が決める（has-udd は CLI/MCP に限定しない）。

**次に確定した前提（MS-1 の核心）:** サンプルは **CodingSchema が全て確定して初めて着手できる**（順序の制約）。has-udd 自身のコードは規約と共進化してきたためサンプルの被験体になれない（カンニングに近い）——**規約確定後に初めて着手する新規コード**である必要がある。

---

## 論点 MS-2: サンプルはどう作るか（プロセス）

### AI 見解（MS-1 の結論を踏まえた確定版・DomainSpec ステップを削除）
**見解:** spec は挟まない。**CodingSchema instance の render 済みガイドのみを入力に、ConceptPlacement の全 concept を総当たりでプレースホルダ実装する。**

手順:
1. 対象スタックの CodingSchema instance ×4（tech-stack/architecture/coding-standard/test-standard）を確定させる（前提条件）。
2. render してガイド .md ×4 を得る（既存の render engine で機械的に生成）。
3. **ガイドだけを見て**（instance の JSON は見ない・render 結果のみを入力にする＝実際のエンドユーザー体験を模す）、ConceptPlacement の各 concept（usecase/aggregate/entity/value-object/domain-service/repository/port/inbound-adapter/outbound-adapter 等）を最小のプレースホルダ題材で1つずつ実装する。
4. 実装中に「ガイドに書かれておらず詰まった点」をログする（＝規約の欠落発見。architecture 確定時の自己テストと同じ効果）。
5. tech-stack の Interface 規約が定める様式で実行し、結果を確認する。
6. test-standard の TestTypes（該当するもの）を実装し緑にする。
7. DocComment 規約に従って docstring を書く。
8. 詰まった点（4）を規約にフィードバックし、必要なら CodingSchema instance を更新する。

**根拠:** architecture 確定時の自己テスト（uc-place-order で実装できるか検証→repository/DI/エラー伝播の3欠落を発見）と同じ手法をプロジェクト規模に拡張する。spec を挟まないことで MS-1 の「業務題材ではなく構造検証が目的」という結論と整合する。

---

## 論点 MS-3: サンプルの置き場・構成・命名

### AI 見解（MS-1 の結論を踏まえた確定版）
**見解:** サンプルは**規約と共進化していない、独立した新規コード**でなければならない（MS-1 の核心）。よって has-udd リポジトリ内に置く場合も、`src/has_udd/` とは独立した場所（例: `examples/{stack}/`）に置き、**has_udd 自身のコードを一切 import/流用しない**。

- 汎用的には「対象スタック向けのサンプルはどこに置くか」は各プロジェクトの判断だが、has-udd 自身がこの手法を検証する（ドッグフード）場合は `examples/python-hexagonal/` に置く。
- 中身は `src/has_udd/` と同じパッケージ構造を模してよいが、**参照するのは render 済みガイドのみ**（has_udd のソースコードは参照しない＝カンニング防止の徹底）。

### ユーザー見解
> 置き場所はそのアーキテクチャとして正しく動くように配置しますよ。これは動作保証も含めているんですから当然ですよね。

### AI 再考見解（訂正）
**見解:** その通り。サンプルの内部構造は **architecture instance の Layout ブロック（正典ツリー）どおりに配置することが必須要件**であり、これは合格基準1（「規約どおりの配置・命名」）に既に含まれていた話。ここを「選べる置き場」として別論点に立てたこと自体が誤り——動作保証は Layout 準拠と不可分。
リポジトリ内のどこに格納するか（`examples/{stack}/` 等）は動作保証と無関係な管理上の細目にすぎず、各プロジェクトの裁量でよい。

### 合意決定（MS-3）
**決定:** MS-3 は独立論点として立てない。「サンプルの配置＝architecture instance の Layout に従う」は MS-4 合格基準1に統合済み。管理上の格納場所（examples/ 等）は任意（各プロジェクトの裁量）。

---

## 論点 MS-4: 「正しさ」をどう確認するか（効果測定との関係）

### AI 見解（確定版・DocComment の機械保証範囲を訂正込みで反映）
**見解:** 合格基準は**汎用4項目**（has-udd 特化の検証手段を混ぜない）:

| # | 合格基準 | 検証手段 |
|---|---|---|
| 1 | ConceptPlacement 全概念が、対象スタックの規約どおりの配置・命名で実装されている | 目視 or 将来の Stage B ゲート2（構造照合） |
| 2 | tech-stack の Interface 規約が定める様式で実行し、結果を確認できる | 実行確認 |
| 3 | test-standard の TestTypes（該当するもの）が緑 | テスト実行 |
| 4 | DocComment 規約に従っている | **構造は機械検証・意味は AI レビュー（下記）** |

**4 の内訳（訂正の反映）:**
- **機械保証できる（構造）**: 公開要素に docstring が存在するか／要約行が空でないか／Args の引数名が実シグネチャと一致するか／禁止パターン（`@spec:`等のカスタムタグ）が混入していないか。
- **機械保証できない（意味）**: 要約行が「検索・判断に効く語」を選べているか＝AI が構造データを見て判断・推論する（Stage B ゲート3・言語検査の先取り）。

**「サンプルを作る行為」自体が最初の効果測定**（roadmap Stage K の効果測定項目を先取りする）という位置づけは維持。

---

---

## 論点 MS-5: DocComment の構造的検証（サンプル実装で発覚した欠陥）

### ユーザー見解（起点）
> 作成されたサンプルを見たが、docstring が構造的ではない。記載内容をブロック単位で表現していないと機械的なバリデーションができないでしょう。逆に今どうやっているんだ？ この形式でどうやってインデックスを作成するつもりなんだ？ document.json は持たないかもしれないが、docstring の schema は持っておかないと検証も構造の担保もできない。それは理解できているか？

### AI 見解（欠陥の告白と設計提案）
**見解:** ご指摘は正しい。coding-standard の DocComment ブロックは**自由記述のガイダンス**（style/summaryGuidance/example）であって、**機械が検証できる構造の宣言ではない**。`code_scan` PoC も「docstring の1行目を summary とみなす」というヒューリスティックに過ぎず、Google スタイルという構造を engine が形式的に理解しているわけではなかった。前回「Args とシグネチャの整合を機械検証できる」と言ったのは、この構造を宣言する schema が無いのに在る前提で話しており不正確だった。

**提案：`DocCommentSchema`**（RenderMetaSchema と同型の閉じた語彙）を新設する。
- has-udd の一貫原則（構造は schema が宣言・engine はそれに従い決定的に処理）を docstring にも適用。
- `code_scan` を「1行目決め打ち」から「宣言された構造に従って `{summary, body, args:[{name,description}], returns, raises:[...]}` に構造化抽出する」へ格上げ。
- **検証（validate/lint 相当）はこの構造化データに対して行う**：必須セクションの有無・Args の name が実シグネチャと一致するか等。ここで初めて機械保証が本物になる。

### ユーザー見解（続）
> これは考えられる言語ごとに決まっているから、それが kind になるよね？

### AI 見解（discriminator 設計）
**見解:** 同意。Google スタイル（セクション見出し）と TSDoc/Javadoc（行頭タグ `@param` 等）と Godoc（構造化セクションほぼ無し）は**マーカーの付け方自体が構造的に異なる**ため、「値の違い」でなく **specKind/codingKind と同じ allOf if/then の discriminator（`kind`）**にすべき。

```
DocCommentSchema/v1
  kind: google | tsdoc | javadoc | godoc | rustdoc（将来拡張）
  allOf if/then → kind ごとの Sections 定義（マーカー・item構造）
```

coding-standard の DocComment ブロックはこの `kind` を参照する（tech-stack の言語と対応：Python なら google）。

### ユーザー見解（続2・document.json 要否の切り分け）
> これをしておけば、document.json を保持しなくても機械的にバリデーションとインデックス作成はできる。document.json が欲しいのは md 等で視覚的に見せないといけない場合。ソースコードは md にしないから、これは筋が通っている。

### AI 見解（原則の確定）
**見解:** 正確。**document.json の存在理由＝「MD へレンダリングする必要があるか」**という切り分けが確定した。

| 条件 | 例 | document.json |
|---|---|---|
| 人が読む視覚的文書として render する必要がある | Spec・CodingSchema instance | ✅ 要る（JSON→MD 変換のため構造化保存に価値） |
| 構造を機械が直接ソースから抽出・検証でき、人向けMD化が不要 | ソースコードの docstring | ❌ 要らない（DocCommentSchema による直接検証・直接抽出で完結） |

これは [[project-ai-era-detail-design]] の「導出物＝キャッシュ（保存せず機械が都度計算）」原則の具体例であり、"レンダリング系"（json→md）と"抽出系"（source→構造化ビュー）という2つの経路に Harness 原則を正しく振り分けたもの。

### ユーザー見解（続3・手順の指摘）
> ひとまずブレストに追記。実装を開始する前にちゃんと spec を書くこと。既存の spec をちゃんと確認すること。

### ユーザー見解（続4・usecase分割の指摘）
> ユースケースは分けたくない？ ドキュメントをクエリするわけではないからね。

### AI 再考見解（訂正・実装都合でのドメイン結合を撤回）
**見解:** 正しい指摘。「query engine の実装を再利用できるから」という理由で uc-query-document を拡張しようとしたのは、**実装都合でドメインモデルを結合する誤り**。アクターの意図が異なる（「Document から意味単位を取得したい」≠「ソースコードから docstring 構造を取得したい」）ので usecase は分ける。

**確定した設計（DDD判断・advisor 主導）:**
- 既存 uc-query-document/uc-validate-document（対象=Document・aggregateRef=agg-document）は変更しない。
- **新規 uc-scan-source-code**（subdomainRef=sd-harness-core・aggregateRef無し=ソースコードはDocumentでなく has-udd 所有のライフサイクルを持たないため）。
- **新規 uc-lint-doc-comment**（subdomainRef=sd-validation・aggregateRef無し）。
- 既存の「query/validate」という対概念を、対象（Document→ソースコード）だけ変えてそのまま再現。実装（engine クラス）を共有するかは別問題（ES-3 の帰結どおり query engine 拡張でよい）——ドメインモデルとしては別 usecase。
- DocCommentSchema 自体は agg-schema 配下の新しい Schema インスタンス（SpecSchema/CodingSchema/RenderMetaSchema と同格）。

### 合意決定（MS-5）
**決定:** `uc-scan-source-code.json` / `uc-lint-doc-comment.json` を作成（specKind=usecase・aggregateRef無し）。両方 validate=VALIDATED・render/.feature生成確認済み。bc-has-udd-engines の Members に追加・features/validate.feature に2行追加。behave 70/70・pytest 15/15 緑。

**次のステップ:** DocCommentSchema/v1.json（google kind から）を実装。その後 code_scan/lint engine 実装（query engine 拡張）。

### 状態
✅ **CLOSED（UsecaseSpec 2本まで完了。実装は次ラウンド）**

---

## 全体まとめ（2026-07・MS-1〜4 到達点）

- サンプルの構成要素は**2つ**：CodingSchema instance ×4（＝ガイド）／見本サンプル本体（ConceptPlacement 全 concept のプレースホルダ総当たり実装）。DomainSpec・付随ガイドは作らない。
- **前提条件**：CodingSchema が全て確定して初めて着手できる（順序の制約）。has-udd 自身のコードは規約と共進化してきたため被験体になれない＝**規約確定後に着手する新規コード**が必須。
- **汎用性**：本ブレストは has-udd 自身のアーキテクチャに限らず、has-udd を介してエンドユーザーが指定する任意のアーキテクチャ・言語・規約についての話。視覚化・検証手段は対象スタックが決める（has-udd 側の機能に引きずられない）。
- 置き場：`examples/{stack}/`（has_udd 自身のコードを import/流用しない・ガイドのみを入力にする）。
- 合格基準4点（構造配置・実行確認・テスト緑・DocComment＝構造機械検証＋意味AIレビュー）。

<!-- MS-3 のユーザー見解は未記入。確認が要れば追記してから CLOSED にする。 -->
