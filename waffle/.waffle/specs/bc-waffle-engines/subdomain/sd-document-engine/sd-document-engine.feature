Feature: sd-document-engine

  Scenario: パステンプレートは変数を解決する
    Given 変数を含むパステンプレートと解決に必要な値
    When resolve する
    Then 全ての変数が値に置き換わった実パスが返る

  Scenario: 逆解析は実パスからテンプレート変数を復元する
    Given パステンプレートと、そのテンプレートから解決された実パス
    When reverse-parse する
    Then resolve時に使った値と同じ変数が復元される

  Scenario: テンプレートと一致しないパスは復元できない
    Given テンプレートの区切り構造と一致しない実パス
    When reverse-parse する
    Then 復元は失敗する

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
