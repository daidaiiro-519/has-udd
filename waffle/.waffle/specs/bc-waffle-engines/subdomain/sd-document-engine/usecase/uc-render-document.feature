Feature: uc-render-document

  Scenario: 同じDocumentを2回renderしても同一の成果物になる
    Given 変更されていないDocument
    When 同じDocumentを2回renderする
    Then 1回目と2回目の成果物は同一である

  Scenario: paragraph/listが正しく整形される
    Given paragraph/listを宣言するx-render
    When renderする
    Then paragraphは地の文、listは箇条書きとして整形される

  Scenario: tableはパイプ文字をエスケープしboolを整形する
    Given パイプ文字やbool値を含む行データ
    When tableとしてrenderする
    Then パイプ文字はエスケープされ、boolは✓/-に整形される

  Scenario: sectionは入れ子とitemLabelを整形する
    Given itemLabelを持つsection宣言と入れ子のeach部品
    When renderする
    Then 各itemの見出しにitemLabelが付与され、入れ子の部品も正しく描画される

  Scenario: keyvalueが正しく整形される
    Given keyvalueを宣言するx-render
    When renderする
    Then ラベルと値の組が箇条書きとして整形される

  Scenario: sectionはbadgeで条件付き強調を付与する
    Given badge条件を満たすitemを含むsection宣言
    When renderする
    Then 条件を満たすitemの見出しにのみ強調語が付与される

  Scenario: tableはmarkFieldで識別子を太字強調する
    Given markFieldが真の行を含むtable宣言
    When renderする
    Then 該当セルが太字＋markSuffixで強調される

  Scenario: statediagramが正しいMermaid構文になる
    Given 状態遷移の配列を宣言するx-render
    When renderする
    Then stateDiagram-v2として正しいMermaid構文が生成される

  Scenario: statediagramは疑似状態を表現する
    Given pseudoStatesFromで疑似状態を宣言するx-render
    When renderする
    Then choice/fork/joinの疑似状態宣言がMermaid構文の先頭に出力される

  Scenario: sequenceはactor/participantを区別する
    Given kind:actor/participantを含む参加者宣言
    When renderする
    Then actor/participantそれぞれの宣言がMermaid構文で区別される

  Scenario: sequenceはloop/altを入れ子で表現する
    Given loop/alt種別のstepを含むsteps配列
    When renderする
    Then loop/altブロックが正しく入れ子のMermaid構文になる

  Scenario: sequenceはactivate/deactivateを表現する
    Given activate/deactivateフラグを持つstep
    When renderする
    Then Mermaidのアクティベーション記法(+/-)が正しく付与される

  Scenario: architectureが正しいMermaid構文になる
    Given zones/connectionsを宣言するx-render
    When renderする
    Then architecture-betaとして正しいMermaid構文が生成される

  Scenario: flowchartが正しいMermaid構文になる
    Given stages/transitionsを宣言するx-render
    When renderする
    Then flowchart LRとして正しいMermaid構文が生成される

  Scenario: kvtableは単一行として整形される
    Given kvtableを宣言するx-render
    When renderする
    Then block自身の値が1行のtableとして整形される

  Scenario: tableはjoin指定で配列セルを結合整形する
    Given join/sepを指定したcolumns宣言と配列値を持つセル
    When renderする
    Then 配列の各要素がjoinテンプレートで整形されsepで連結される

  Scenario: render_engineはschemaのx_render宣言をpart_rendererへ正しく配線する
    Given interfaceブロック(x-render宣言=table)を持つDocument
    When render engine経由でrenderする
    Then schemaのx-render宣言どおりに整形されたMarkdownテーブルが出力に含まれる

  Scenario: 検証済み Document を成果物に描画する
    Given 描画対象の Document
    When render する
    Then 成果物が生成され、生成パス一覧が返る

  Scenario: schemaRef を持たない Document は描画しない
    Given schemaRef の無い Document
    When render する
    Then MISSING_SCHEMA_REF エラーが返る

  Scenario: 存在しないパスは描画しない
    When 存在しないパスを対象に render する
    Then INVALID_PATH エラーが返る

  Scenario: deploy すると canonical と deploy 先の両方に書く
    Given deploy 先を持つ Document
    When deploy を有効にして render する
    Then canonical と deploy 先の両方に成果物が書かれる

  Scenario: SkillSchemaをMarkdownにレンダリングする
    Given SkillSchemaのDocument
    When renderする
    Then 見出し・目的・パラメータ表・オペレーション選択・呼び出し例が全て出力に含まれる

  Scenario: frontmatterはx_frontmatterのドットパスを解決して生成する
    Given x-frontmatterを宣言するSchemaのDocument
    When renderする
    Then 出力冒頭にname/description等を含むYAML frontmatterが生成される

  Scenario: CodingSchemaはMarkdownとして描画できる
    Given CodingSchemaのDocument
    When renderする
    Then Markdown形式で見出しを含む出力が生成される

  Scenario: usecase_Specは基本フローをシーケンス図に受け入れシナリオをMarkdownに出す
    Given usecase SpecのDocument
    When renderする
    Then 出力にmermaidのsequenceDiagramとテストシナリオ節が含まれ、feature出力にも同じシナリオが含まれる

  Scenario: aggregate_Specは集約の構造とライフサイクルをMarkdownに出す
    Given aggregate SpecのDocument
    When renderする
    Then 出力にコマンド節・ドメインイベント名・mermaidのstateDiagram-v2が含まれる

  Scenario: 不正なJSONはINVALID_JSON
    Given 不正なJSONの対象ファイル
    When renderする
    Then INVALID_JSONエラーが返る
