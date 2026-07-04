# ブレインストーミング: has-uddの2つの側面（engine / agent system）とOSS分離

**目的:** has-uddが実は「engine」（document.jsonスキーマ駆動処理）と「agent system」
（ハーネス/ループ駆動のエージェンティックスクラム）という2つの独立した側面（2枚のヘキサゴナル）で
構成されているという気づきを掘り下げ、それぞれを独立OSSとして分離すべきか・どう境界を引くべきかを
検討する。ブレスト`brainstorm-context-observability.md`論点2の次のアクションとして開始。
**モード:** アイデア発散

---

## 前提となる事実確認（着手前に実施済み）

`src/has_udd/`配下（domain/application/adapters）を確認したところ、中身は**完全に「engine」側だけ**
で構成されている: document.jsonのドメインモデル（`domain/model/{Spec,Skill,Coding,RenderMeta}Schema`）・
検証/query/render/scaffoldのusecase（`application/usecases`）・validator/schema_repositoryのport
（`application/ports`）・CLI/MCP/jsonschemaのadapter。

一方「agent system」（ハーネス/ループのもとで動くエージェンティックスクラム）に相当するコードは、
この`src/has_udd/`には**存在しない**。それは`.claude/skills/*`・`CLAUDE.md`・hooks設定という
**Claude Code層の規約・プロンプトとして**実装されている状態。

この事実は論点1の出発点になる。

---

## アイデアダンプ

1. has-udd engineを独立OSS化し、別名で公開する（document.jsonスキーマ駆動エンジン単体）
2. agent system側はengineに依存しない汎用インターフェース（プラグイン形式）でengineを呼び出す設計にする
3. 現状のモノレポ構成のまま、内部的にモジュール境界だけ明確化する（分離自体は将来に先送り）
4. LoomDBと同様、`git subtree split`で将来切り出せる自己完結ディレクトリ構成に今のうちにしておく
5. engineのインターフェースをMCPツールとして公開し、agent systemはMCP経由で呼ぶ
   （engine実装を差し替え可能にする）
6. agent system自体も汎用フレームワーク化し、「ハーネス/ループ駆動のアジャイルスクラムエージェント
   システム」として、has-udd engine以外のドメインにも転用できるようにする
7. 【突飛】engine・agent system・LoomDB・debug tool（前回ブレスト）の4つを独立した
   「製品ファミリー」としてブランディングし、has-uddはそれらを組み合わせた「リファレンス実装」
   という位置付けに変える
8. engineのバージョニングとagent systemのバージョニングを分離し、互換性マトリクスを明示する
9. 【突飛】agent systemを「スクラムエンジン」として、has-udd engine以外の別ドメイン（例: 別の
   構造化データ管理系OSS）と組み合わせるデモを作り、汎用性を実証する
10. has-udd engineのschemaRef/blockKey/x-prompt-query概念を、他プロジェクトでも使える汎用ライブラリ
    として先に切り出す（agent system分離より前にengine分離を優先する）
11. 現行の`src/has_udd/`のhexagonal構造を調べ、実際に境界が「engineポート」と「agent systemポート」
    に分かれているかをコードベースで検証する（→着手前の事実確認で実施済み。結果は上記の通り）
12. 【突飛】engine分離時の名前衝突チェック（LoomDBのように既存OSS名との衝突を先に調べる）
13. 分離の判断基準として「他プロジェクトから使いたいという需要が実際にあるか」を検証するPoC
    （例えば別の個人プロジェクトでhas-udd engineだけ試しに使ってみる）

**絞り込み候補:**

| # | アイデア | 選定理由 |
|---|---|---|
| 11 | コードベースでの境界の事実確認 | 実現性最高（着手前に実施済み）・全ての前提になる |
| 4 | 自己完結ディレクトリ構成（LoomDB方式） | LoomDBで実例あり・実現性高 |
| 10 | engine分離をagent systemより先に優先 | 現状engineは既にコードとして境界が明確・着手しやすい |
| 6 | agent systemの汎用フレームワーク化 | インパクト大きいが、現状コード化されていないため検討の難度が高い |

---

## 論点 1: 現行コードベースの境界は本当に「engine」と「agent system」に分かれているか？

### AI 初期見解
**見解:** 分かれている、ただし非対称な形で。「engine」は`src/has_udd/`にPythonコードとして
明確に存在するが、「agent system」は現状コード化されておらず、`.claude/`層の規約・プロンプトとして
のみ存在する。
**根拠:**
- `src/has_udd/`のディレクトリ構成（domain/application/adapters/ports）は完全にdocument.json処理
  （検証・query・render・scaffold）に閉じており、ハーネス/ループ/サブエージェント運用に関わる
  ポートや usecase は一つも存在しない。
- 一方「agent system」に相当する概念（ループ・ハーネス・サブエージェント委任・Hook）は、
  `.claude/skills/*/SKILL.md`・`CLAUDE.md`・`.claude/settings.json`のhooks設定として実装されている。
  これらはPythonコードではなく、Claude Codeというツールが解釈するテキスト/設定ファイル。
- つまり「2枚のヘキサゴナル」という表現は比喩として正確だが、現状「engineヘキサゴナル」だけが
  実際にports/adaptersという形式でコード化されており、「agent systemヘキサゴナル」はまだ
  そこまで明示的な構造を持っていない。

### ユーザー見解
> それであってるよ！今はエンジンの作り込み中だからね

### AI 再考見解
**見解:** この非対称性は設計上の欠陥ではなく、**開発の順序（今はengineの作り込みフェーズ）が
そのまま反映されている**だけ、と理解を修正する。agent systemがまだコード化されていないのは
「後回しにされている」のではなく「まだそのフェーズに来ていない」という自然な状態。
**根拠:**
- 開発順序として今はengine（Waffle）を作り込んでいる最中であり、agent systemの抽象化は
  その後のフェーズで着手される予定。境界の非対称性は現在地を正しく反映しているだけで、
  問題があるわけではない。
- これは論点2（どちらを先に分離すべきか）にそのまま接続する: 既にengine作り込み中という
  現在地がある以上、「先にengineを独立OSS化する」という進め方は新たな優先順位付けではなく、
  **今の開発の流れをそのまま延長するだけ**で済む。

### 合意決定
**決定:** engineとagent systemの境界は非対称（engineはコード化済み、agent systemは未着手）で
問題なし。これは現在の開発フェーズ（engine作り込み中）を正しく反映したものであり、
分離の判断はこの現在地を前提に進める。
**理由:** 非対称性の原因が「設計の欠陥」ではなく「開発順序」であることが確認できたため。
**次のアクション:** 論点2（分離順序）の議論に、この現在地（engine作り込み中）を前提として進む。

---

## 論点 2: 分離するとしたら、engineとagent system、どちらを先に独立OSS化すべきか？

### AI 初期見解
**見解:** engineを先に独立OSS化すべき。agent systemは現状コード化されていないため、分離以前に
「何を分離するのか」を定義する作業が別途必要になる。
**根拠:**
- 論点1の通り、engineは既にPythonコードとしてports/adaptersが明確に存在し、LoomDBが辿った
  「`loomdb/`ディレクトリを自己完結させてから`git subtree split`で切り出す」という手順を
  そのまま踏襲できる。着手コストが低い。
- agent systemは現状「Claude Code向けのSkill/CLAUDE.md/hooksの集合」であり、これを「独立OSS」と
  呼ぶには、まず汎用フレームワークとしての形（他のengineとも組み合わせられる抽象化）を新たに
  設計・実装する必要がある。これは分離作業というより新規開発に近い。
- 先にengineを独立させておけば、「engineをMCP経由で叩く」という現在合意している設計
  （前回ブレスト論点5）とも自然に整合し、agent system側の抽象化を後から進める際の土台になる。

### ユーザー見解
> ✏️ _（あなたの考え・反論・追加情報をここに書いてください）_

### AI 再考見解
**見解:** {{修正・深化された立場}}
**根拠:**
- {{根拠1}}
- {{根拠2}}

### 合意決定
**決定:** {{合意した内容}}
**理由:** {{なぜこれに合意したか}}
**次のアクション:** {{次のステップ}}

---

## 論点 3: 分離の形式は「別リポジトリ」か「同一リポジトリ内の自己完結ディレクトリ」か？

### AI 初期見解
**見解:** LoomDBと同じ「同一リポジトリ内の自己完結ディレクトリ」方式を踏襲すべき。
**根拠:**
- `loomdb/CLAUDE.md`に記録されている通り、LoomDBは「`git subtree split --prefix=loomdb`で
  そのまま独立リポジトリに切り出せる」という自己完結構成を既に採用しており、実績がある
  （設計としての検証コストが既に払われている）。
- 別リポジトリに最初から分けると、engine側の変更をhas-udd側で試しながら並行開発する際に
  パッケージ公開・バージョン待ちのオーバーヘッドが生じる。自己完結ディレクトリなら
  モノレポ内でローカルパス参照しながら開発でき、公開はタイミングを選んで後から行える。
- 「今すぐ複数リポジトリを管理する複雑さ」を避けつつ、「将来切り出せる」という選択肢を
  失わない、というLoomDBで既に合意済みの方針と一貫性が取れる。

### ユーザー見解
> loomdbと同じような形で管理したい。後々リポジトリ分離できるようにしておいてください。

### AI 再考見解
**見解:** 合意通り実行。ただし実行途中で、当初の想定より深い結合が2つ見つかり、それぞれ
ユーザーとの対話で解決した。
**根拠:**
- **見つかった結合その1**: バンドルされているschema（SkillSchema/SpecSchema/CodingSchema/
  RenderMetaSchema）を「has-uddの都合でengineに同梱されている外部依存」と誤認していたが、
  ユーザー指摘により「これらはWaffle自身のschema資産（Waffleというエンジンが定義する
  ドキュメント型そのもの）」という理解に訂正した。外部化はしない。
- **見つかった結合その2**: `features/`のテストがhas-uddの実document（`.has-udd/documents/...`）
  を参照していたのは、has-uddの任意コンテンツではなく「Waffle自身を説明するspec/skill
  document」（bc-has-udd-engines・uc-query-document等）だったため、これもWaffle自身の
  資産として`waffle/.has-udd/documents/`にコピーし、Waffle単体でテスト完結するようにした
  （repo root側の「本物」＝`.claude/skills/`へのdeploy元とは重複するが、render先パス解決の
  複雑化を避ける現実的な選択として許容）。
- 汎用skill（`analyze-domain-model.json`）だけはWaffle固有でないため、Waffleのドッグ
  フーディング対象から正しく除外した。

### 合意決定
**決定:** `waffle/`を`loomdb/`と同じ自己完結ディレクトリ構成にした（`git subtree split
--prefix=waffle`で将来切り出し可能）。バンドルschemaは外部化せずWaffle自身の資産として維持。
repo rootからの呼び出しは`uv run --project waffle waffle <command>`に統一。
**理由:** LoomDBで実績のある構成をそのまま踏襲でき、Waffle自身を説明するdocumentは実際に
Waffleの資産だったため、外部依存を作らずに自己完結できたため。
**次のアクション:** waffle単体のpytest 15件・behave 65シナリオ、repo rootからの実呼び出し
（`uv run --project waffle waffle validate ...`）の両方でgreenを確認し、コミット・
プッシュ済み（コミット`92a7651`）。

---

## 論点 4: agent systemは本当にhas-udd engineに依存しない汎用フレームワークとして成立するか？

### AI 初期見解
**見解:** 概念としては成立しうるが、現時点では未実証であり、断定は避けるべき。
**根拠:**
- 「ハーネス/ループ駆動でエージェントがアジャイルスクラムを回す」という仕組み自体は、
  対象がdocument.json（has-udd engine）である必然性はなく、理論上は別のドメイン
  （例: 別の構造化データ管理OSS）にも適用できそうに見える。
- ただし現状、agent system側の抽象化（「engineに何を要求するか」というインターフェース）は
  一度も明文化されたことがなく、「本当に汎用か」は少なくとも1つ、has-udd engine以外の
  対象と組み合わせて動かしてみるまでは検証できない（論点2で決めたengine先行分離が終わった後の
  話にはなる）。
- 汎用性を最初から前提にして設計すると、過剰な抽象化（本当に必要になる前の一般化）に陥る
  リスクもある。「まずhas-udd engineとの組み合わせで動くagent system」を固めてから、
  2つ目の対象で試す、という順序が安全。

### ユーザー見解
> waffle自体がUDDで仕様を整理し、コーディングをサポートしてくれるOSSなんです。つまり
> has-uddは、そのwaffleを使用して構成されるエージェントシステムだということです。この
> waffleを元にテキストベースでskillやagentをハンドリングしてエージェンティックスクラムを
> 構成することにagent systemの責務が発生する。ここで入出力インターフェースの共通言語を
> テキストベースにしたことが効いてきた——これがwaffleに依存したインターフェースだったら
> 終わっていた。付け加えるなら、has-uddは依然としてwaffle提供のMCPとCLIを利用することには
> 変わりない。

### AI 再考見解
**見解:** 定義を整理し直す。**Waffle = UDDで仕様を整理し、コーディングを支援するOSS。
has-udd = そのWaffleを使って構成されるエージェントシステムで、Skill/Agentをテキストベースで
ハンドリングし、エージェンティックスクラムを構成する責務を持つ。** agent systemの汎用性は
「Waffleのschema語彙を抽象的な契約として扱えるか」という理屈ではなく、もっと単純に
**agent systemとWaffleの入出力がテキストベース（CLI/MCP）に閉じており、Waffleの内部実装
（Pythonオブジェクト・schemaの内部構造）に一切触れない作りになっているから**、という
実装事実がそのまま答えだった。
**根拠:**
- `brainstorm-loomdb-has-udd-document-db.md`論点5で「共通インターフェースはテキストベース・
  各SkillがLLMによる成形を担う」という疎結合方針を既に決めていた（当時はAPI課金を避けて
  サブスクリプション内で回すためという別の理由だった）。この方針が、結果的にagent systemを
  Waffleの実装詳細から完全に切り離す働きもしていた。
- ただし「依存しない」は「使わない」という意味ではない。has-uddは依然としてWaffleが提供する
  MCP・CLIを実際の操作手段として利用し続ける。切り離されているのは**内部実装への結合**
  であって、**インターフェースとしての利用**ではない。

### 合意決定
**決定:** agent system（has-udd）はWaffleの内部実装には依存しないが、Waffleが提供する
MCP・CLIというインターフェースは引き続き利用する。この疎結合は新たに設計する必要はなく、
既存の合意（テキストベース共通インターフェース）がそのまま答えになっていた。
**理由:** agent systemとWaffleの入出力が最初からテキストベース（CLI/MCP）に閉じており、
Waffleの内部実装に一度も触れない作りになっているため。
**次のアクション:** 特になし（論点1〜5すべて決着。本ブレストはここで完了とする）。

---

## 論点 5: engineの改名は今すぐ行うべきか、名称はどうするか？

### AI 初期見解
**見解:** 改名は今すぐ行うべき。理由は論点2で触れた通り、engineはまだ外部にパッケージ公開されて
おらず（PyPI/npm未公開）、改名コストが実質ゼロな今のうちに済ませておくのが得策。

### ユーザー見解
> 今ならhas-uddを別の名前にするだけで済む。幸いagent systemにはまだ着手していないので、
> 「has-udd」という名前を（本来の意味通り）agent system側に残すなら今しかない。

### AI 再考見解
**見解:** `docs/brainstorm/brainstorm-has-udd-concept.md`を確認したところ、
**has-udd = Harness Agentic Scrum Usecase-Driven-Development** の略であることが判明。
頭字語は「HAS（Harness Agentic Scrum＝agent system）」と「UDD（Usecase-Driven-Development＝
engineが支える開発手法）」の合成語だった。この事実がユーザー見解を裏付ける: 「has-udd」という
名前は元々agent system側の概念（Harness Agentic Scrum）を核にしており、engineをこの名前で
呼び続ける方が実は本来の語源とズレている。
**根拠:**
- 「Usecase-Driven-Development」は**開発手法・プラクティス**の名前（Test-Driven Developmentと
  同じ構造）であり、それを実践するのはPO/SM/Devロールが回るagent system側。engine自体は
  「schemaで検証された構造化文書を扱う」汎用的な仕組みに過ぎず、ユースケース文書に限らず
  どんなスキーマ駆動文書にも使える。つまりengineは本質的に「usecase駆動」である必要がなく、
  UDDという色を無理に背負わせる理由がない。
- よってengineには手法的な意味合いを持たない、独立した新しい名前を与え、「has-udd」という
  名前（とUDDが指す開発手法という意味）はagent system側にそのまま残すのが、頭字語の由来にも
  素直に合致する。

### 合意決定
**決定:** engineの新名称を **Waffle** に確定する。
タグライン案: 「Waffle — スキーマという型で文書を焼き上げる、構造検証＋意味ガイダンス内蔵の
ドキュメントエンジン」。
「has-udd」（Harness Agentic Scrum Usecase-Driven-Development）という名前と、それが指す
開発手法（UDD）は、engineではなくagent system側に残す。
**理由:**
- LoomDB・Docker・Kubernetes等、多くのOSS/製品名は名前単体では機能を伝えず、
  タグラインが意味を担う。Waffleも同じパターンで問題ない。
- 名前の由来（blockKeyの格子構造＝ワッフルの格子、schema検証＝型に入れて焼き上げる）は
  比喩として一貫しており、cc-sddのような短くキャッチーな響きよりもユーモアを優先した結果だが、
  タグラインと組み合わせれば伝達性の問題は解消する。
- 副次的な効果として、Waffle自体をhas-udd（agent system）のUsecase-Driven Development実践で
  開発すれば、has-uddの方法論が実際に機能するかどうかの**最初のドッグフーディング事例**になる。
**次のアクション:** `src/has_udd/`を`Waffle`という名称でパッケージ分離する具体的な移行計画
（ディレクトリ構成・PyPI名の空き確認・既存の`.has-udd/documents/skills/harness-*-engine.json`等
呼称の更新範囲）は、論点2（engineを先に独立OSS化する）・論点3（自己完結ディレクトリ形式）の
議論と合わせて後続で詰める。

---

## 追記: パス規約の.waffle化・重複解消（論点3のフォローアップ）

論点3実行時、`waffle/`を自己完結ディレクトリ化したものの、Waffle自身を説明するspec/skill
documentがrepo root側`.has-udd/documents/`（本物）と`waffle/.has-udd/documents/`（テスト
フィクスチャの重複コピー）の2箇所に存在する状態を「既知のトレードオフ」として残していた。

ユーザーからの指摘（「document.jsonは`.has-udd`ではなく`.waffle`になりそうな気がする」
「schemaがWaffle自身の資産なら、パス規約もWaffle自身のものであるべき」）を受けて再検討し、
以下を実行・決着した:

- schemaの`x-source-target`/`x-render-target`パス規約を`.has-udd/`から**`.waffle/`**に変更
  （`.git/`が道具の名前を冠するのと同じ発想。schemaがWaffleの資産である以上、規約もWaffleの
  ものであるべき）
- Waffle自身を説明する14件のdocumentを`waffle/.waffle/documents/`に**一元管理**し、重複コピー
  を解消
- repo root側`.has-udd/documents/`には、Waffle固有でない汎用skill（`analyze-domain-model.json`
  等・has-udd/agent system自身の資産）だけが残る
- `bc-has-udd-engines`（documentId）を`bc-waffle-engines`に改名、参照タグ
  `context:has-udd-engines`も`context:waffle-engines`に追従

これにより「Waffleが担う責務」と「has-udd（agent system）が担う責務」の境界が、
ディレクトリ構成・パス規約のレベルでも明確になった。
