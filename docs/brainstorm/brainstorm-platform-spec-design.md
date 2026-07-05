# ブレインストーミング: PlatformSpec（第3のSpec家族・非DDD・インフラ/プラットフォーム層）の内容schema設計

**目的:** `DomainSpecSchema`（DDD）・`PresentationSpecSchema`（非DDD・UI層）に続く第3のSpec家族として、プロダクト全体レベルのインフラ/プラットフォーム意図を表現する`PlatformSpec`の内容schemaを設計する。
**モード:** 設計判断（論点 → 見解 → 合意）

**確定済みの前提（揺らさない）:**
- `PresentationSpecSchema`と同じ理屈（「コードから導けない、authoredなプロダクト全体レベルの意図」）で正当化される
- usecase単位の`OperationGuarantees`とは別スコープ（`PlatformSpec`はusecaseからは独立した、プロダクト/システム全体レベル）
- 実際のIaCコード自体は引き続きSSOT（構造の重複記述はしない）
- AIは`PlatformSpec`（WHAT）＋`CodingSchema`のcode-template（HOW・tech-lead-advisor管轄）の両方を見て、初めてIaCを安全に設計実装できる、という2層構造
- 「usecase要件からIaC構成を直接導出する」という接続は実現性が無いとして撤回済み（同じ過ちを繰り返さない）

---

## 論点 1: `PlatformSpec`の`specKind`はどう分けるか

### AI 初期見解
**見解:** `PresentationSpecSchema`が`screen`/`flow`の2種類に分けたのと同様に、`PlatformSpec`も**関心事の性質で複数の`specKind`に分ける**。候補: `capacity`（スケール/性能目標）・`resilience`（可用性・災害対策・バックアップ）・`security`（セキュリティ境界・コンプライアンス）・`topology`（デプロイ構成・ネットワーク区分）。
**根拠:**
- これらは互いに関心事が異なり（可用性目標とセキュリティ境界は別の著者・別のレビュー観点になりやすい）、1つの巨大な`specKind`に混ぜるとPresentationSpecSchemaで避けた「何でも屋」化のリスクがある。
- `DomainSpecSchema`のbounded-context/subdomain/aggregate/usecaseも「粒度の異なる関心事の分離」なので、同じ設計哲学に沿う。
- ただし4分類が本当に必要かは実例が無いと判断しづらく、過剰設計の可能性もある。

### ユーザー見解
> レンダリング実例を見て判断したい。実際に4分割(capacity/resilience/security/topology)をプロトタイプで検証 → ダミー例では薄すぎて「1つに統合すべき」と誤判定 → 現実的な粒度で埋め直すと4分割が妥当と分かった → しかし「AWSがComputeのように領域を分けているのが参考になる」という指摘があり、**目的軸(capacity/resilience/security/topology)ではなく、コンポーネント軸(Compute/Storage/Database等)の方が自然では**という再検討が発生。AWS公式の23カテゴリから、プロダクトの土台となる核だけに絞り込み、さらにCI/CD・IoT・ML・Analyticsを追加して**最終的に11分類（Compute/Storage/Database/Networking/Security/Integration/Observability/CI-CD/IoT/ML/Analytics）で合意**。

### AI 再考見解（最終）
**見解:** `specKind`は**コンポーネント軸（AWS公式カテゴリを参考にした11分類）**を採用する。当初提案した目的軸（capacity/resilience/security/topology）は撤回する。
**根拠:**
- 実際にレンダリングして検証した結果、目的軸で分類すると「同時アクセスユーザー数（Compute寄り）」と「データ増加率（Storage寄り）」のように、**1つのspecKind内でコンポーネント種別が混在**してしまうことが判明した。コンポーネント軸の方が実際のインフラ設計者の思考順序に自然に合う。
- `specKind`のenumは追加コストが低い（未使用のkindは単に使われないだけ）ため、YAGNI的な絞り込みより、ユーザーの指摘通り多様なプロダクト種別に備えた広めの列挙が理にかなう。
- AWS公式の23カテゴリ全部は「ほとんどのプロダクトに関係ない特化カテゴリ」（Game tech/Satellite/Quantum等）を含みすぎるため、核となるものに絞り、ユーザー指摘で復活させたCI/CD・IoT・ML・Analyticsを加えた11分類が妥当な着地点。

### 合意決定
**決定:** `specKind`は以下11分類とする: **Compute・Storage・Database・Networking・Security・Integration・Observability・CI-CD・IoT・ML・Analytics**。
**理由:** レンダリング実例で目的軸の混在問題が実証され、コンポーネント軸（AWS公式カテゴリを参考に絞り込み＋ユーザー指摘分を追加）の方が自然だと判明したため。
**次のアクション:** 論点2（各specKindのブロック構成）を、この新しいコンポーネント軸に合わせて再検討する。既存のプロトタイプ（`PlatformSpec/v1.json`・目的軸ベース）は作り直しが必要。

---

## 論点 2: 具体的にどんなブロックを持つか

### AI 初期見解
**見解:** `PresentationSpecSchema`の`Fields`（usecaseから導けない項目だけ書く）に相当する原則をそのまま踏襲し、**「IaCコードから機械的に導出できる情報は書かない・authoredな意図だけを書く」**というブロック構成にする。候補:
- `Summary`（既存パターン踏襲）
- `NonFunctionalTargets`（可用性%・RTO/RPO・スケール目標等、数値目標）
- `SecurityBoundary`（データ分類・アクセス境界・コンプライアンス要件）
- `PlatformTestScenarios`（`GuaranteeScenarios`と同型のGiven/When/Then。ただしプロダクト全体スコープ）

**根拠:**
- 数値目標（可用性%等）はコードから導けない、まさに authored な意図の典型。
- `PlatformTestScenarios`を持たせることで、今日確立した「spec→ネイティブテスト→ドリフト検知」の同じループにIaCも将来乗せられる（今日は実装しないが、拡張の芽を残す）。
- 具体的なIaC実装パターン（どのAWSサービスを使うか等）はCodingSchema code-templateの領分なので、ここには一切含めない。

### ユーザー見解
> 論点1の決定（コンポーネント軸11分類）を受けて、まず`compute`（資源系の代表）と`security`（横断系の代表）の2パターンを実際にレンダリングして検証してから残り9分類に展開する、という進め方に合意。その後、全11分類のレンダリング結果を見て「マーメイド図があえて無いのはなぜか」を指摘 →
> 調査の結果、(a) ブロック設計時に表(table)以外の描画形式を検討し直していなかったこと、(b) そもそも`architecture-beta`/`flowchart`が`part_renderer.py`にまだ実装されていなかったこと（`sequence`/`statediagram`の2種類のみ対応）の2点が判明。`networking`のゾーン分離とci-cdのリリース昇格は本来トポロジー/フロー図で表現すべきと確認。
> mermaid-guideの`pattern-architecture.md`/`pattern-flowchart.md`を参照して実装するよう指示。実装後、「トポロジーという言葉は一般的か・全体的に語彙がわかりにくい」と指摘 → `NetworkTopology`→`NetworkLayout`（ネットワーク構成図）、`ReleaseFlow`→`ReleasePipeline`（リリースパイプライン、表(ReleasePolicy)との一語違いによる紛らわしさの解消）に改名して合意。

### AI 再考見解
**見解:** ブロック構成は「資源系5分類=共通トリプレット(容量/耐障害性/コンポーネント固有セキュリティ)」＋「横断・特化系6分類=専用ブロック」の二層構成で決定。加えて、**表形式だけでなく図解形式（Mermaid）も部品の選択肢に含めて設計すべき**という教訓を得た——`DomainSpecSchema`の`MainFlow`/`Lifecycle`が既に`sequence`/`statediagram`を使っていたのに、`PlatformSpec`設計時にそれを踏襲せず表に偏ったのは設計上の見落とし。トポロジー（ネットワークのゾーン構成）とリリース昇格順序は、関係性・順序を伝える情報であり表より図が適切、という判断基準を得た。また、ユビキタス言語の観点では「表形式ブロックと図形式ブロックが同じ情報を指す場合、名前で役割の違い（詳細 vs 可視化）を明示する」必要があり、`Flow`/`Policy`のような1語違いの命名は避けるべき。
**根拠:**
- `part_renderer.py`の対応部品を`sequence`/`statediagram`から`architecture`（zones+connections→architecture-beta）/`flowchart`（stages+transitions→flowchart LR）に拡張し、`RenderMetaSchema/v1`の閉じた語彙・`test_schema_xrender_conforms`にも反映（`PlatformSpec/v1`を対象schemaに追加）。mermaid-guideの検証済み構文（非ASCIIラベルのクォート必須・`{group}`エッジの3グループ以上バグ回避）に準拠。
- 全11分類のexample document作成・validate・renderで実証し、pytest/behave回帰も緑を維持。

### 合意決定
**決定:** ブロック構成は資源系共通トリプレット＋横断/特化系専用ブロックの二層構成。**表形式に加え、関係性・順序を表す情報は図解部品（`architecture`/`flowchart`。既存の`sequence`/`statediagram`と合わせ計4種のMermaid部品）で表現する。** `networking`は`NetworkLayout`（ネットワーク構成図・architecture-beta）を追加、`ci-cd`は`ReleasePipeline`（リリースパイプライン・flowchart）を追加。表(`ReleasePolicy`)と図(`ReleasePipeline`)は「詳細 vs 可視化」の役割分担として共存。
**理由:** 実際のレンダリングで表現力不足（関係性が文章に埋没）を確認し、mermaid-guideの既存パターンを流用することで実装コストを抑えつつ表現力を確保できたため。命名は紛らわしさの指摘を受けて、役割の違いが名前から伝わる形に修正した。
**次のアクション:** 全11分類のexample documentをコミットし、残りの論点3(IaCコードとの対応関係)・論点4(著者role)に進む。

### 再検討・追加合意（2026-07-06）: `PlatformTestScenarios`ブロックの追加
論点3の「IaCコードとの対応関係=人の判断」という記述に対し、ユーザーから「通常のspec(DomainSpec)側はTestScenarios/GuaranteeScenarios→ネイティブテスト→`check_scenario_drift.py`という機械検知可能な仕組みを既に持っているはず」という指摘があり、論点2で最初に候補として出しながら合意過程で議論なしに落としていた`PlatformTestScenarios`を追加することで合意。
**確認した論点:** ①既存パターン(GuaranteeScenarios)との整合性、②IaCの世界にも policy-as-code(checkov/tfsec/OPA/Terratest等)という対応する実践があること、③ただし検証対象のIaCコード自体がこのリポジトリにまだ存在しないため、今日のスコープは「schemaへの追加とexample作成まで」であり「実際のドリフト検知が機能するところまでの実証」は将来に送る、という3点。`ddd-advisor`はDDD書籍スコープ外(PlatformSpecは非DDD)、`platform-advisor`は未設計のため、既存advisorへの確認はできず、上記3点を自己点検する形で判断した。
**実装:** `PlatformTestScenariosBlock`を`GuaranteeScenariosBlock`と同型(background+scenarios[{name,category,viewpoint,gherkin,covers}])で追加し、全8 content型にoptionalフィールド`testScenarios`として追加(**contentキー名をDomainSpecSchemaの`TestScenariosBlock`と同じ`testScenarios`に揃えた**ため、`check_scenario_drift.py`は一切のコード変更なしでPlatformSpecドキュメントも読める)。`x-render-target`に`feature`フォーマット・`featurePath`も追加。`plat-storage.json`に実例(保存時暗号化の検証シナリオ)を追加し、validate/render/`.feature`生成/`check_scenario_drift.py`(missing_in_tests検知)まで実地で確認、既存回帰(pytest 59/behave 49)は緑を維持。
**次のアクション:** 残り10分類へのtestScenarios実例追加は必要に応じて後日。IaC実装が実在する段階でネイティブテスト(policy-as-code)を書き、ドリフト検知ループの実動作を実証する。

---

## 論点 3: IaCコードとの対応関係（x-source-target/x-render-target）

### AI 初期見解
**見解:** `x-source-target`/`x-render-target`は**他のSpec家族と全く同じ形**（`.waffle/documents/specs/...`にsource・`.waffle/specs/...`にrendered MD）を踏襲する。**IaCコード自体への直接参照（ファイルパスの埋め込み等）は持たせない**——IaCコードとの結びつきは「同じテーマの`PlatformSpec`とIaCコードが、人（tech-lead-advisor）の判断によって整合していること」を前提にし、機械的なリンクは持たない。
**根拠:**
- 「リンクは管理でなく計算」という憲法級原則（`project-ai-era-detail-design`）に従うなら、PlatformSpecとIaCコードの対応関係も、保存されたリンクではなく、将来的にはIaCスキャン（Terramaid等）とPlatformSpecの内容を都度突き合わせる形にすべき。
- ただし今この場でその突き合わせ機構まで設計するのは、以前撤回した「usecase→IaC直接導出」と同じ過剰な野心になりかねない。今回はPlatformSpec自体の内容schemaに留め、IaCとの整合検証は将来の別ブレストに送る。

### ユーザー見解
> 「他のspecとソースコードの関係と同じでいいと思っている」——`DomainSpec`/`PresentationSpec`が実装コードに対して機械的リンクを持たず人の判断（reconcile）で整合を保つのと、`PlatformSpec`がIaCコードに対して持つ関係は、同じ扱いでよい。

### AI 再考見解
**見解:** AI初期見解のとおりで確定。`PlatformSpec`をIaCコードのための「特別な新しい対応関係」として設計する必要はなく、既存のSpec家族が実装コードと結ぶ関係（機械的リンク無し・authoredなWHAT＋人の判断による整合）をそのまま当てはめれば十分。
**根拠:** これは「PlatformSpecは第3のSpec家族であり、DomainSpec/PresentationSpecと同格」という当初からの前提そのものなので、対応関係だけ特別扱いする理由がない。特別なIaCリンク機構を作ろうとすると、以前撤回した「usecase→IaC直接導出」の過剰な野心が形を変えて再燃するリスクがある。

### 合意決定
**決定:** `x-source-target`/`x-render-target`は他のSpec家族と全く同じ形を踏襲する。IaCコード自体への機械的リンク（ファイルパス埋め込み等）は持たせない。PlatformSpecとIaCコードの整合は、人（platform-advisor）の判断に委ねる。
**理由:** PlatformSpecは他のSpec家族と対等な「第3の家族」であり、対応関係も同一原則（authored WHAT＋人が担保する整合）に従うのが一貫している。将来的なIaCスキャン照合機構は、必要になった時点で別ブレストとして起票する。
**次のアクション:** なし（既存のsource/rendered配置規約をそのまま適用すればよく、追加実装は不要）。

---

## 論点 4: 著者は誰か（tech-lead-advisor相当のroleが要るか）

### AI 初期見解
**見解:** `PlatformSpec`の著者は、既存の`ddd-advisor`（業務ドメイン判断）とは異なる専門性が要るため、**新しい`platform-advisor`（または`tech-lead-advisor`と統合）を新設**すべき。
**根拠:**
- 可用性目標・セキュリティ境界の判断は、DDDの知識体系（本セッションのddd-advisorの知識ベース）の範囲外であり、別の専門知識（SRE・セキュリティ・クラウドアーキテクチャ）が要る。
- 以前合意した「tech-lead-advisor」構想（code-templateを1回だけ判断する役割）と、PlatformSpec自体を著す役割は、対象（コード構造 vs プロダクト全体のインフラ要件）が異なるため、同一roleに統合するかは要検討。

### ユーザー見解
> 新規に`platform-advisor`を作る（tech-lead-advisorとは統合しない）。

### AI 再考見解
**見解:** AI初期見解で挙げた懸念（対象領域が異なる＝コード構造の判断 vs プロダクト全体のインフラ要件の判断）を理由に、統合せず独立roleとする判断は妥当。`ddd-advisor`が「ドメイン駆動設計をはじめよう」の書籍知識を土台にしたのと同様、`platform-advisor`もSRE/セキュリティ/クラウドアーキテクチャ領域の専門知識ベース（knowledgeファイル群）を別途持つ必要がある。
**根拠:** `tech-lead-advisor`はコードの構造判断（DDDのskilled-engineer依存問題を埋める役割）に特化しており、対象読者・判断基準・参照する知識体系のいずれもPlatformSpecの著者（インフラ/プラットフォーム要件の判断）とは異なる。1つのroleに無理に統合すると、`ddd-advisor`が業務ドメイン判断に特化して初めて機能したのと同じ理由で、責務が薄まる。

### 合意決定
**決定:** `platform-advisor`を新規roleとして設ける（`tech-lead-advisor`とは統合しない）。
**理由:** 対象領域（コード構造 vs プロダクト全体のインフラ要件）が異なり、専門知識ベースも別であるため、統合すると各roleの専門性が薄まる。
**次のアクション:** `platform-advisor`のknowledge構成（ddd-advisorの`references/knowledge/*`に相当するもの）の具体設計は別セッション/別ブレストで行う。

---

## セッションまとめ

### 合意事項一覧
1. **specKind**: コンポーネント軸11分類（Compute/Storage/Database/Networking/Security/Integration/Observability/CI-CD/IoT/ML/Analytics）。
2. **ブロック構成**: 資源系5分類=共通トリプレット(容量/耐障害性/コンポーネント固有セキュリティ)＋横断・特化系6分類=専用ブロック。表形式に加え、関係性・順序を表す情報は図解部品（`architecture`/`flowchart`）で表現する（`networking`=`NetworkLayout`、`ci-cd`=`ReleasePipeline`）。
3. **IaCコードとの対応関係**: 他のSpec家族と全く同じ形（機械的リンク無し・人の判断による整合）を踏襲する。特別な対応関係は設けない。
4. **著者role**: `platform-advisor`を新規roleとして設ける（`tech-lead-advisor`とは統合しない）。

### 次のアクション一覧
- `platform-advisor`のknowledge構成の具体設計（別セッション）
- 将来的なIaCスキャン照合機構の検討（必要になった時点で別ブレスト）

### 未解決・保留の論点
- なし（論点1〜4すべて合意済み）
