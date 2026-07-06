# ブレインストーミング: waffle Schema(agg-schema)のバージョニング/移行機構

**目的:** agg-schema.jsonのCommands(publishVersion/deprecateVersion/migrateDocuments)の未整理な仕様（曖昧な状態遷移・後方互換の判定基準・移行ロジック・旧版の保管モデル）を固め、「schemaが進化した際に既存Documentが気づかれず陳腐化する」という第4のドリフトを検知できる土台を作る。
**モード:** 問題解決

> **★後日談・一部撤回（同セッション後半）**: このブレストで合意した`migrateDocuments`コマンド・`x-migration`語彙(`MigrationMetaSchema`)・`MigrationEngine`・`sd-schema-governance`subdomainは、実装完了後の見直しで**撤去された**。理由: (1)実際にx-migrationを必要とした実schemaが1つも存在しなかった（DomainSpecSchema自身のv1→v2進化も、この機構を使わずgitでの手動リネームで行われた実績がある）、(2)各schemaを使う実document数が最大でも十数件程度と少なく、機械的な一括変換の必要性が薄い、(3)Waffleのdocumentは大量の同質データではなく少数の練り上げられた設計仕様であり、schema進化のたびにAIが個別に判断して直す方が適切、という3点から「本当に必要になる前の投機的な先行実装（YAGNI違反）」と判断した。**「第4のドリフト検知」という本来の目的は`check_schema_version_drift.py`（本ブレスト論点5の成果）だけで引き続き果たされており、これは撤去せず維持している**。`publishVersion`/`deprecateVersion`もコマンド/engineとしては撤去し、`x-schema-status`はschema作成/更新時にAIが直接記述する単純なフィールド＋不変条件（PUBLISHED→DEPRECATEDの順にのみ進む）として表現し直した。詳細はメモリ`project-waffle-schema-versioning-migration`参照。

---

## 問題の再定義

agg-schemaのCommands仕様が未整理なせいで、schemaがv1→v2と進化しても既存Documentが新schemaに対して有効かどうかを機械的に検証・追跡できず、第4のドリフト（schema進化によるDocument陳腐化）が検知不能になっている。

## 原因候補

- **表面的原因**: publishVersionのrequiresStateが「（新規）」という未確定のプレースホルダーのまま放置されていた
- **根本原因**: agg-schemaのCommandsは「集約として持つべき振る舞い」を先に列挙しただけで、各コマンドが実際に何を検証し何を変換するかという実装可能な詳細まで詰めずにスキーマ設計フェーズを終えてしまった（spec先行はよいが、検証可能な粒度まで至っていなかった）

---

## 論点 1: publishVersionのrequiresStateは何にすべきか

### AI 初期見解
**見解:** publishVersionは実は「初回公開」と「新版の公開」で状態遷移が異なるため、単一のrequiresStateでは表現しきれない。requiresStateを`null`（新規schemaの初回公開）と`PUBLISHED`（既存schemaの新版公開、旧版はPUBLISHEDのまま新版が並行して公開される）の両方を許容する形にするか、コマンドを`publishInitialVersion`/`publishNewVersion`に分割すべき。
**根拠:**
- 「（新規）」というプレースホルダーは、まさにこの2ケースを区別できていないことの現れ
- 他のコマンド（deprecateVersion: requiresState=PUBLISHED、migrateDocuments: requiresState=PUBLISHED）は単一の具体的な状態を持てているのに、publishVersionだけ持てないのは、このコマンドが実は2つの異なる操作を1つの名前に押し込めているサイン
- DDDの原則（集約のコマンドは公開インターフェースとして明確な事前条件を持つべき）に照らすと、事前条件が曖昧なコマンドは分割すべき

### ユーザー見解
> 「つまり新しいバージョンでインスタンスを作り直すってことですかね？」との確認を経て、「その方針がいいかもしれないね」と合意。`v1`を書き換えるのではなく、`v1`を残したまま`v2`という別インスタンスを新規に作る、という理解。

### AI 再考見解
**見解:** Schema集約の同一性は`(name, version)`のペアであり、`v1`と`v2`は別々の独立したライフサイクル（CREATED→PUBLISHED→DEPRECATED）を持つ別インスタンス。`publishVersion`は既存インスタンスの状態遷移コマンドではなく、**新しいバージョンインスタンスを新規生成するコマンド**であり、`requiresState=null`（まだ存在しない）はその生成型コマンドの正当な事前条件——曖昧なプレースホルダーではなかった。
**根拠:**
- 現状の実際の運用（`v1.json`を`v2.json`にリネーム/上書き）は、旧版を物理的に消してしまうため、`deprecateVersion`/`migrateDocuments`という「旧版が残っている前提のコマンド」と矛盾していた
- 「別インスタンスとして作る」という結論は、論点4（旧バージョンの保管モデル＝v1.jsonを残す）と表裏一体の同じ設計判断

### 合意決定
**決定:** Schema集約の同一性は`(name, version)`。`publishVersion`は新規バージョンインスタンスの生成コマンドであり、`requiresState=null`が正しい。
**理由:** 旧版を物理的に残し続けることで、`deprecateVersion`/`migrateDocuments`が意味を持つ前提が保たれる。
**次のアクション:** 論点4（保管モデル）でこの結論を具体的なファイル配置規約に落とし込む。

---

## 論点 2: 後方互換性の判定基準をどう一般化するか

### AI 初期見解
**見解:** JSON Schemaの一般的な後方互換性理論（「既存のvalidな値が新schemaでも引き続きvalidであり続けること」）を判定基準の軸に据え、変更種別ごとに破壊的/非破壊的を分類する。
**根拠（分類案）:**
- 破壊的（後方互換を壊す）: 新規必須フィールドの追加・既存フィールドの削除・型の変更・enum値の削除・制約の強化（minLength増加等）
- 非破壊的（許容）: 新規オプショナルフィールドの追加・enum値の追加・制約の緩和（minLength減少等）
- この分類は「既存Documentの値がそのまま新schemaでvalidかどうか」という機械的テスト（旧Documentサンプルを新schemaでvalidateしてみる）で実際に検証可能——GuaranteeScenariosの1例（必須フィールド追加→拒否）はこの一般原則の特殊ケースに過ぎない

### ユーザー見解
> 「ルールベースの分類ではなく、実データによる実証的検証」という再考案に合意。

### AI 再考見解
**見解:** JSON Schemaキーワードごとの破壊的/非破壊的ルール表を新規に作るのではなく、**「旧バージョンで既に検証済みの実在するdocument.jsonインスタンスを、新バージョンのvalidateにそのまま通す」という実証的検証**を後方互換性の判定基準に据える。1件でも失敗すれば後方互換性違反として拒否する。
**根拠:**
- 「後方互換を壊さない」の定義そのものが「既存の有効なDocumentが新schemaでも引き続き有効であること」なので、理論的な分類より実証的な検証の方が定義に忠実
- 新しい"schema差分解析エンジン"を書く必要が無く、**既存の`uc-validate-document`（ValidateEngine）をそのまま再利用**できる——今日確立した「新しい仕組みを作る前に既存の道具で足りないか確認する」という原則に合致
- JSON Schemaのあらゆるキーワードの組み合わせを網羅するルール表は、保守コストが高く漏れも生じやすい

### 合意決定
**決定:** 後方互換性の判定は、旧バージョンの既存document.jsonサンプル群を新バージョンのvalidateに通す実証的検証で行う。ルールベースの静的分類は採用しない。
**理由:** 定義への忠実性・既存資産（ValidateEngine）の再利用・保守コストの低さ。
**次のアクション:** migrateDocuments/publishVersionの実装時に、このvalidate再利用の検証ステップを具体的な手順として組み込む。

---

## 論点 3: migrateDocumentsの変換ロジックはどう設計するか

### AI 初期見解
**見解:** 完全自動変換は現実的でない（新フィールドの初期値や意味的変換はschemaの構造だけからは導出不可能）。Harness原則（AIは値だけ生成・engineが構造を担う）に倣い、**バージョンペアごとの「移行マッピング」を人（またはAI）が値として記述し、engineがそれを機械的に適用する**という2層構造にすべき。
**根拠:**
- scaffold_engineが「構造はengine・値はAI」という分担で成功しているのと同じ設計思想を移行にも適用できる
- 完全自動変換を目指すと、結局「新フィールドの妥当な初期値は何か」という業務判断をengineに持たせることになり、Harness原則（engineは決定的・ロジックを持たない）に反する

### ユーザー見解
> 初期見解（人/AIが移行スクリプトを丸ごと1つ書く案）に対し「機械的にできる部分とAIの推論が必要な部分を正しく区別しないと完全なマイグレーションはできないのでは」と指摘。さらに「この方法が有効かどうかはシミュレーションしたほうがいい」とPoCでの実証を要求。

### AI 再考見解
**見解:** 移行を1つの`migrate()`関数に丸投げする案は誤りだった。**フィールドごとに`x-migration`宣言を持たせ、機械変換（`rename`/`default`）とAI推論（`ai-infer`）を明示的に区別**し、AI推論が必要なフィールドは**scaffoldの`fillTemplate`と同じ形の「移行ワークシート」**をDocumentごとに生成してAIに埋めさせ、最後に論点2で合意した実証的検証（ValidateEngine相当での検証）で安全網をかける、という設計に修正。

**PoCで実証した内容**（`/tmp/migration_poc/`）:
1. トイschema（v1→v2、rename+default+ai-inferの3種の`x-migration`を持つ）と2件の既存v1 Documentを用意
2. 機械変換フィールド(`biography`のrename・`createdAt`のdefault)とAI推論フィールド(`seniorityLevel`)をx-migration宣言から正しく分離し、機械変換を適用・ワークシートを生成
3. AI(私自身)が各Documentの`biography`の内容を実際に読み、`senior`/`junior`を判定して回答(doc1="10年間シニアエンジニア"→senior、doc2="新卒1年目"→junior)——機械では不可能な意味判断
4. 機械変換結果とAI回答をマージし、実際のjsonschemaライブラリで検証→両方VALID
5. **安全網の実証**: AIが誤ってenum範囲外の値(`mid-level`)を返すケースを模擬→正しくINVALIDとして拒否されることを確認

**根拠:**
- PoCにより、機械/AI推論の分離・ワークシート生成・AIによる意味判断・実証的検証による安全網の4点すべてが期待通り機能することを確認済み
- 「移行スクリプトを1つの塊として書く」設計では、AIが全パターンを事前に網羅しようとして必ず漏れが出るという当初の指摘が正しかった

### 合意決定
**決定:** `migrateDocuments`は、新schemaの各フィールドに`x-migration`（`rename`/`default`/`ai-infer`のいずれか）を宣言させ、機械変換はengineが直接適用、`ai-infer`フィールドはscaffold同型の移行ワークシートをAIに埋めさせる。最終結果は論点2の実証的検証（新バージョンのvalidateに通す）で確認する。
**理由:** PoCで機械/AI推論の分離・ワークシート生成・意味判断・安全網の全てが実際に機能することを確認したため。
**次のアクション:** `x-migration`をDomainSpecSchema等の実際のschema meta構造（RenderMetaSchemaのx-render相当の位置）に正式なフィールドとして定義する。

### ★追記（論点5の実データ検証を受けて）: x-migrationの語彙不足を発見・追加

論点5で、git履歴から実際に削除された`SpecSchema/v1.json`と、当時実在した`dm-document.json`（v1形状）を復元し、現行`DomainSpecSchema/v2.json`で実証的検証（論点2の手法）にかけたところ、**実際にINVALID**という結果が出た（`documentType: "Spec"`→`"DomainSpec"`、`specKind: "domain-model"`→現行enumに存在しない値、という実在した後方互換性の破壊）。

この実例を`x-migration`の`rename`/`default`/`ai-infer`だけで処理できるか検証したところ、**2つの語彙不足が判明**:
- **`value-map`**: フィールド名は変わらず、値そのものが変わるケース（`"Spec"→"DomainSpec"`）。固定の対応表で機械的に処理できる（AIは不要）
- **`discriminator-remap`**: 旧content内のフィールド構成（構造パターン）から新しい値を機械的に判定するケース（`domain-model`が`aggregateRoot`等のフィールドを持っていれば`aggregate`と判定）。意味を読んで考えるのではなく、構造の有無を機械的に照合するだけなのでAIは不要

**重要な発見**: 実際にあった歴史的な移行ケースは、`ai-infer`を一切使わず、全て機械的な`value-map`/`discriminator-remap`で処理できることが分かった。論点3の核（機械変換とAI推論の明示的区別）という設計判断自体は変わらない。

**x-migrationとx-prompt-write/x-prompt-queryの関係（明確化）**: 同義ではない。`x-migration`は`x-render`と同じ立ち位置——`as`種別を持つ宣言的な指示語彙で、大半（`rename`/`default`/`value-map`/`discriminator-remap`）はengineが機械的に実行する命令であり、AIへの語りかけではない。例外は`ai-infer`種別のみで、その内部に`prompt`という自然言語テキストフィールド（`x-prompt-write`相当）を持つ。つまり「`x-migration`全体の一部（`ai-infer`のみ）が、内部にプロンプトを持つ」という関係。

### 合意決定（更新）
**決定:** `x-migration`の語彙を`rename`/`default`/`value-map`/`discriminator-remap`/`ai-infer`の5種に拡張する。うち前4種は完全に機械的（engineが直接実行）、`ai-infer`のみがAIの推論を要し、内部に`prompt`フィールドを持つ。
**理由:** 実際の歴史的移行ケース（git履歴から復元した実データ）で実証した結果、機械的な語彙を広げるだけで対応でき、AI推論(`ai-infer`)は最後の手段として温存すれば十分だと判明したため。

---

## 論点 4: 旧バージョンの保管モデルはどうあるべきか

### AI 初期見解
**見解:** 現状のようにv1ファイルを上書き/改名するのではなく、**旧バージョンのファイルを`{SchemaName}/v{N}.json`として並存させ続ける**べき。実際`DocstringSchema/v1.json`のように「ディレクトリ名+バージョンファイル名」という規約は既に存在するので、単に「新版を作る時に旧版を消さない」というルールを徹底するだけで実現できる。
**根拠:**
- 旧版が物理的に残っていなければ、そもそも「後方互換性の検証」（旧版の値が新版でもvalidか）を機械的にテストする対象が無くなる
- deprecateVersionは「新規Document作成を止める」だけで既存版を消す操作ではないと既に定義されているため、この方針と整合する

### ユーザー見解
> 「OKです」と合意。

### AI 再考見解
**見解:** 保管モデルの器は既に存在している——ファイル配置規約`{SchemaName}/v{N}.json`と、スキーマファイル自体が既に持つ`x-schema-status`（`PUBLISHED`/`DEPRECATED`）フィールドを組み合わせるだけでよい。新しい仕組みは不要で、**「新版作成時に旧版ファイルを削除/リネームしない」という運用規律の徹底**と、**`deprecateVersion`実行時に旧版ファイルの`x-schema-status`を`DEPRECATED`へ機械的に更新する**操作だけで実現できる。
**根拠:**
- 今日実際に「DomainSpecSchema v1→v2」を行った際、この規律を破って旧ファイルを削除してしまっていたことが、そもそも論点1〜4全体の発端だった
- 既存の`x-schema-status`フィールドをそのまま再利用でき、新しいデータ構造の追加が不要

### 合意決定
**決定:** 旧バージョンのファイルは`{SchemaName}/v{N}.json`として削除せず並存させ続ける。`deprecateVersion`実行時に該当ファイルの`x-schema-status`を`DEPRECATED`に更新する。新しい保管機構は作らない。
**理由:** 既存の規約・フィールドの再利用で十分に実現できるため。
**次のアクション:** 論点5（第4のドリフト検知）で、この`x-schema-status`と各Documentの`schemaRef`を突き合わせる仕組みを設計する。

---

## 論点 5: これは「第4のドリフト検知」の土台としてどう機能するか

### AI 初期見解
**見解:** 各document.jsonは既に`schemaRef`（例: "DomainSpecSchema/v2"）でバージョン込みの参照を持っている。旧版がdeprecateされた時点で、**「そのバージョンをまだ参照しているdocument一覧」を検出する新しいチェックスクリプト**（`check_scenario_drift.py`・`check_spec_referential_integrity.py`に続く第3のスクリプト）を作れば、第4のドリフト検知が実現できる。
**根拠:**
- `schemaRef`は既存のフィールドであり、新しいデータ構造の追加なしに検知が可能
- 今日確立した「機械的な名前の突き合わせで検知する」という一貫したパターン（テストシナリオ名↔テスト関数名、bc.members↔subdomain.members）をここでも踏襲できる

### ユーザー見解
> 「詳しく」との要望を受け、AIが実データ（git履歴から復元した実際のv1 schema・v1形状のdocument）でPoCを実施。結果、実際に後方互換性の破壊が実証された。さらに「仮に疑似的にv1を復元して確かめることはできたりしないかな？」という提案がPoCの出発点になった。

### AI 再考見解
**見解:** 提案を受け、`git show`でリポジトリ履歴から実際に削除された`SpecSchema/v1.json`（コミット`f7c83a3`で削除）と、その直前に実在した`dm-document.json`（v1形状、コミット`826447c^`時点）を復元し、現行`DomainSpecSchema/v2.json`で実証的検証（論点2の手法）にかけた。結果は**INVALID**——`documentType: "Spec"→"DomainSpec"`・`specKind: "domain-model"`（現行enumに存在しない）という、実際に起きていた後方互換性の破壊が判明した。
**根拠:**
- 合成PoC（論点3のトイschema）だけでなく、実際の歴史的データで検証できたことで、論点2の実証的検証手法・第4のドリフト検知という設計全体の妥当性がより強く裏付けられた
- ただし現状のチェッカーを今のリポジトリに対して実行すると0件（旧バージョンファイルが既に削除されているため比較対象が無い）——これは論点4の規律が今後徹底されて初めて機能するものであり、過去に遡っての検証はできないという限界がある

### 合意決定
**決定:** 第4のドリフト検知として、`schemaRef`が指すschemaファイルの実在確認（`broken_references`）・`x-schema-status`が`DEPRECATED`かの確認（`deprecated_references`）・より新しいバージョンの存在確認（`newer_version_available`）の3段階を報告する新チェックスクリプトを作る。
**理由:** 実データでのPoCにより、この設計が実際に機能する（少なくとも論点2の実証的検証と組み合わせて本物のドリフトを検知できる）ことが確認できたため。
**次のアクション:** 論点4の規律（旧版ファイルを削除しない）が今後守られることを前提に、`check_schema_version_drift.py`として実装する。過去分（既に削除されたv1群）は遡って検証できないため対象外とする。

---

## セッションまとめ

### 合意事項一覧
1. **Schema集約の同一性は`(name, version)`**。`v1`/`v2`は別々の独立したライフサイクルを持つ別インスタンス。`publishVersion`は新規バージョンの生成コマンドで`requiresState=null`が正しい。
2. **後方互換性の判定は実証的検証**（旧バージョンの既存document.jsonサンプルを新バージョンのvalidateに通す）で行う。ルールベースの静的分類は採用しない。
3. **`migrateDocuments`は`x-migration`宣言（`rename`/`default`/`value-map`/`discriminator-remap`/`ai-infer`の5種）で機械変換とAI推論を明示的に区別する**。PoC（合成データ＋実際の歴史的データの両方）で設計の妥当性を実証済み。`ai-infer`のみが内部に`prompt`（x-prompt-write相当）を持つ。
4. **旧バージョンのファイルは削除せず`{SchemaName}/v{N}.json`として並存させ続ける**。`deprecateVersion`実行時に`x-schema-status`を`DEPRECATED`に更新する。新しい保管機構は不要。
5. **第4のドリフト検知（`check_schema_version_drift.py`）**: `broken_references`（schemaファイル不在）・`deprecated_references`（DEPRECATED版参照）・`newer_version_available`（より新しい版の存在）の3段階を報告する。

### 次のアクション一覧
- `x-migration`をDomainSpecSchema等の実際のschema meta構造に正式なフィールドとして定義する
- `check_schema_version_drift.py`を実装する（論点4の規律が今後守られることが前提）
- `publishVersion`/`deprecateVersion`/`migrateDocuments`コマンドをMigrationEngineとして実装する

### 未解決・保留の論点
- なし（論点1〜5すべて合意済み）
