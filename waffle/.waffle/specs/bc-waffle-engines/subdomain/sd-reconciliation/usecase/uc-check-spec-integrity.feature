Feature: uc-check-spec-integrity

  Scenario: 存在しないbc.jsonはINVALID_PATH
    When 存在しないbc.jsonのパスで参照整合性検査を実行する
    Then INVALID_PATHエラーが返る

  Scenario: 全ての宣言と実態が一致するとき差分なしと判定する
    Given bc.jsonの宣言とディスク上の実ファイルが完全に一致するspecツリー
    When 参照整合性検査を実行する
    Then 6フィールド全てが空配列で返る

  Scenario: 宣言されたsubdomainがディスクに無いことを検出する
    Given bc.jsonがsubdomainを宣言するが、そのディレクトリが実在しないspecツリー
    When 参照整合性検査を実行する
    Then declared_subdomains_missing_on_diskにその名前が含まれる

  Scenario: 未宣言のsubdomainがディスクにあることを検出する
    Given ディスク上に実在するがbc.jsonに宣言されていないsubdomainを含むspecツリー
    When 参照整合性検査を実行する
    Then subdomains_on_disk_not_declared_in_bcにその名前が含まれる

  Scenario: どのsubdomainにも属さない宙に浮いたusecaseを検出する
    Given bc.jsonがusecaseを宣言するが、どのsubdomainのmembersにも含まれないspecツリー
    When 参照整合性検査を実行する
    Then usecases_orphaned_no_subdomainにその名前が含まれる

  Scenario: subdomainには属するがbcに未宣言のusecaseを検出する
    Given いずれかのsubdomainのmembersが宣言するがbc.jsonには宣言されていないusecaseを含むspecツリー
    When 参照整合性検査を実行する
    Then usecases_in_subdomain_not_declared_in_bcにその名前が含まれる

  Scenario: 宣言されたusecaseの実ファイルが無いことを検出する
    Given subdomainがusecaseを宣言するが、対応するjsonファイルが実在しないspecツリー
    When 参照整合性検査を実行する
    Then usecase_files_missing_on_diskにその名前が含まれる

  Scenario: 未宣言のusecaseファイルがディスクにあることを検出する
    Given ディスク上に実在するがどのsubdomainのmembersにも宣言されていないusecaseファイルを含むspecツリー
    When 参照整合性検査を実行する
    Then usecase_files_orphaned_on_diskにその名前が含まれる
