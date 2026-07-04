# PoC計画: LoomDBによるhas-uddドキュメント管理

**元ブレインストーム:** [brainstorm-loomdb-has-udd-document-db.md](./brainstorm-loomdb-has-udd-document-db.md)（論点1〜5 全合意済み）
**目的:** ブレインストームで合意した5論点を、検証可能な小規模PoCとして実行に移す。
**ステータス:** 計画段階（未着手）

---

## 0. このPoCで検証したいこと（仮説）

1. **既存 `document.json` 群を、LoomDBの `M` 型属性として丸ごと保持しつつ、
   メタデータ（`documentId`/`schemaRef`/`aggregateRef` 等）だけを索引可能な形で
   テーブル化できる**（論点1・論点2）。
2. **JOINで `aggregateRef`/`subdomainRef` の参照整合性チェックができる**
   ＝壊れた参照（存在しないdocumentIdを指すref）を機械的に検出できる（論点2）。
3. **経路B（既存フォーマット文書の変換）の再現性は、7ステップの構造化プロセス＋
   MCPツールで担保できる**（論点3・論点5）。
4. **`x-prompt-query` を意味検証ゲートとして転用できる**（論点4）。

このPoCは「LoomDBを本番導入する」ことが目的ではなく、上記の技術的成立性を
小さく確かめることが目的。**LoomDB本体への新規機能追加は想定しない**
（既存機能の組み合わせで完結する範囲に留める）。

---

## 1. スコープ

### やること
- has-udd の実ドキュメント数点をLoomDBに取り込み、JOINで参照整合性を検証する
  最小スクリプト（`loom-py` 経由）を書く。
- 経路B変換用のMCPツール3本を **SKILL.md として設計**し、うち構造検証
  （`validate_document` 相当）は既存 `validate_engine` の薄いラップで実装する。
- 意味検証ゲート（`x-prompt-query` 転用）を、既存の1文書に対して**手動で1回**
  試し、判定が実用的な粒度で返るかを確認する。

### やらないこと（Out of Scope）
- LoomDB本体のコア機能追加・変更
- 本番運用への切替（has-uddの正データソースは引き続きファイル）
- 全15文書の一括変換パイプライン化（対象は少数のサンプルに限定）
- ループラッパーskillの実装（設計メモのみ。実装は本PoCが成立してから）
- 監査ログ（`transact_write` 活用）の実装（有望だが今回は見送り。§6 に記録のみ）

---

## 2. PoC対象データ

`.has-udd/documents/` 配下 15文書のうち、参照関係が実在する組を使う。

| documentId | schemaRef | 役割 |
|---|---|---|
| `agg-document` | SpecSchema/v2 | 参照される側（aggregate） |
| `sd-validation` | SpecSchema/v2 | 参照される側（subdomain）。`members` で `uc-validate-document` を逆参照 |
| `uc-validate-document` | SpecSchema/v2 | 参照する側（`aggregateRef: agg-document` / `subdomainRef: sd-validation`） |
| `uc-render-document` | SpecSchema/v2 | 参照する側（正常参照の追加サンプル。内容未確認・投入時に参照先を確認） |
| `harness-query-engine` | SkillSchema/v1 | 別スキーマ種の混在確認用（documentType: Skill） |

さらに、**意図的に壊れた参照を持つ合成ドキュメント**を1件PoC専用に追加する
（例: `aggregateRef: "agg-does-not-exist"` を持つダミーのuseCase文書）。
JOIN整合性チェックが「正常系で何も出さない／異常系で確実に検出する」の両方を
実証するために必須（負例なしでは「たまたま何も出なかっただけ」を否定できない）。

配置場所: `loomdb/poc/has-udd-docs/`（LoomDB側の自己完結ディレクトリ内。
has-udd本体のデータには一切触れず、コピーで実験する）。

---

## 3. テーブル設計（Phase 1）

```
table: documents
  pk: documentId (S)
  sk: "META" (固定・単一アイテム構成のため)
  属性:
    documentType   (S)  — "Spec" | "Skill"
    schemaRef      (S)  — "SpecSchema/v2" 等
    specKind       (S, optional) — "aggregate" | "subdomain" | "usecase"
    status         (S)  — "CREATED" | "VALIDATED" | ...
    aggregateRef   (S, optional)
    subdomainRef   (S, optional)
    tags           (SS, optional)
    body           (M)  — document.json の内容をまるごと格納（丸ごとJSON保持の実証）

GSI: gsi-aggregateRef  (pk: aggregateRef)  — 「このaggregateを参照している文書一覧」
GSI: gsi-subdomainRef  (pk: subdomainRef)  — 同上（subdomain版）
```

- `pk=documentId, sk="META"` という単純キーにするのは、このPoCが「1文書=1アイテム」
  の粒度で十分だから（content内のblock単位での索引化は将来課題・本PoCの対象外）。
- `body` を `M` 型にすることで、元の `document.json` を情報欠落なくLoomDB内に
  復元可能な形で保持できることを実証する（論点2の合意事項そのもの）。

**Acceptance Criteria (Phase 1):**
- [ ] 上記5+1文書を `loom-py` 経由でput → `get` した結果が元の `document.json` と
      （キー順序を除き）意味的に一致する（round-trip検証）
- [ ] `body` から特定パス（例 `body.content.summary.text`）をProjectionで
      取り出せる

---

## 4. JOIN参照整合性チェック（Phase 2）

`uc-validate-document.aggregateRef` → `agg-document.documentId` を
inner JOINで解決するクエリと、壊れた参照を持つ合成文書を含めた同じクエリを走らせる。

```
JOIN: usecases (root) -> documents (aggregateRef = documentId, kind: left)
```

- LEFT JOINで実行し、結合先が`null`になった行 = 参照切れ、として抽出する
  （inner JOINだと「切れている」こと自体が結果から消えてしまうため、
  検出ロジックとしてはLEFTが正しい）。

**Acceptance Criteria (Phase 2):**
- [ ] 正常参照（`uc-validate-document` → `agg-document`, `sd-validation`）が
      LEFT JOINで**結合成功**として現れる
- [ ] 合成した壊れた参照文書が、結合先`null`として**確実に検出**される
- [ ] 検出結果を人間可読な形（例: `documentId, 壊れたref種別, 参照先`）で出力する
      簡易レポート関数を書く

---

## 5. MCPツール設計（Phase 3・経路B専用）

論点5で合意した3ツールを、has-udd既存の `adapters/inbound/mcp/main.py` パターンに
倣ってSKILL.md相当の設計メモとして書き下ろす。**この段階では設計のみ**
（実装は本PoCのAcceptance Criteriaを満たしてから着手するかを再度確認する）。

```python
tool: get_conversion_target(source_path: str) -> {
    raw_text: str,        # 変換元の生テキスト
    target_schema: dict,  # 変換先スキーマ本体
    x_prompt_write: str,  # スキーマに埋め込まれた執筆ガイダンス（静的定数）
}

tool: validate_document(candidate_json: dict, schema_ref: str) -> {
    valid: bool,
    errors: list[str],
}
# 実装方針: 既存 ValidateEngine / jsonschema_validator を薄くラップするだけ。
# 新規の検証ロジックは書かない（構造検証は既に確立済みという合意を踏襲）。

tool: save_converted_document(document_json: dict, target_path: str) -> {
    status: "CREATED" | "VALIDATED",
}
```

- 3ツールとも**決定的**（同じ入力なら同じ出力）。LLMによる意味判断は一切含まない
  ＝呼び出し元セッション（Claude Code等）がツールの返り値を見て推論する側に徹する
  （論点5の合意通り）。
- `get_conversion_target` の `x_prompt_write` は静的にスキーマから読み出すだけで、
  動的に生成・判断されるものではない（論点4での確認事項と整合）。

**Acceptance Criteria (Phase 3):**
- [ ] 3ツールの入出力契約がSKILL.md形式で書かれている
- [ ] `validate_document` が既存 `ValidateEngine` の呼び出しだけで実装可能なことを
      コードレベルで確認する（新規バリデーションロジックが不要なことの確認）
- [ ] （実装するかは任意判断）最小限、`get_conversion_target` のみ試作し、
      1つの合成「レガシーフォーマット文書」に対して生テキスト＋スキーマ＋
      x-prompt-writeが正しく返ることを確認する

---

## 6. 意味検証ゲートの手動トライアル（Phase 4）

論点4で合意した「`x-prompt-query` を書込検証の第二ゲートとして使う」を、
自動化はせず**手動で1回**試す。

手順:
1. `uc-validate-document.json` の `content.mainFlow`（`blockType: "MainFlow"`）を
   例にとり、そのスキーマ定義の `x-prompt-query` を取得する
2. その `x-prompt-query` の指示に沿って、実際のセッション内推論（このチャット内）で
   「このMainFlowブロックの中身は、意図された意味を満たしているか」を判定してみる
3. 判定結果が実用的な粒度（yes/no + 理由）で得られるかを確認する

**Acceptance Criteria (Phase 4):**
- [ ] 少なくとも1ブロックについて、手動トライアルの判定結果と理由を記録する
- [ ] 「構造検証はpass・意味検証はfail」となるケースを意図的に1つ作り、
      2段階ゲートが独立して機能することを確認する（例: MainFlowの型は合っているが
      内容が別の話題を書いている合成データ）

---

## 7. 見送り事項（記録のみ・実装しない）

- **`transact_write` による監査ログ**（論点2 PoC軸(b)）: 有望だが本PoCのスコープ外。
  Phase 1〜4が成立した後、必要なら別PoCとして着手する。
- **ループラッパーskill**（論点5）: マルチツール対応のskill設計は、本PoCで
  MCPツール自体の成立性が確認できてから着手する方が手戻りが少ない。

---

## 8. 実行順序

1. Phase 1（テーブル設計＋投入・round-trip確認）
2. Phase 2（JOIN整合性チェック・正例＋負例）
3. Phase 3（MCPツール設計・`validate_document`のみ試作判断）
4. Phase 4（意味検証ゲートの手動トライアル）

Phase 1・2が通らない場合、Phase 3・4は着手しない（土台が崩れるため）。

## 9. ユーザーに確認が必要な点

- PoC専用ディレクトリ `loomdb/poc/has-udd-docs/` の作成でよいか
  （has-udd本体データには触れない前提）
- Phase 3の `get_conversion_target` 試作まで今回のPoCでやるか、設計のみに留めるか
- Phase 4の「合成データで意味検証を意図的に失敗させる」ケースを誰が作るか
  （AIが合成してよいか、実例が要るか）
