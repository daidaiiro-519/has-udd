# test-standard-python-hexagonal

---

## テスト方針

- **方針**: ピラミッド形（単体テストを重視・統合はやや軽め・E2E は行わない）
- **根拠**: 業務ロジックの実装方法＝ドメインモデル

---

## テスト計画

| 項目 | 値 |
|---|---|
| 実行タイミング | すべての変更（コミット前ローカル・PR で CI） |
| CI トリガー | push / pull_request |
| ゲート | シナリオ（.feature）緑が必須（Stage B ゲート1） |
| 対象外 | パフォーマンステスト（中核だが性能要件なし） |

---

## テストタイプ

| テストタイプ | ツール | 対象 |
|---|---|---|
| `unit` | pytest | domain / application |
| `integration` | pytest | adapters |
| `acceptance` | pytest（AIが .feature を読んで直接執筆） | usecase（.feature は参照専用の仕様書） |
| `contract` | pytest（スキーマ差分検査） | CLI / MCP の公開インターフェース |

---

## フレームワーク

- **単体テスト**: pytest
- **受け入れテスト**: pytest

---

## シナリオの束ね方

| 項目 | 規約 |
|---|---|
| .feature の生成元 | spec の TestScenarios（手書きしない） |
| .feature の配置 | .waffle/specs/{contextRef}/subdomain/{subdomainRef}/usecase/{documentId}.feature（DomainSpecSchemaのx-render-target.featurePath通り・生成物・参照専用・実行対象ではない） |
| ネイティブテストの配置 | tests/acceptance/test_{spec-id}.py（AIが .feature を見て執筆・手編集前提） |
| 対応関係 | 1 spec（usecase）＝1 .feature（参照専用・render生成）＝1 ネイティブテストファイル（AI執筆） |
| ドリフト検知 | シナリオ名⇔テスト関数名の名前突き合わせで機械検出（未実装/孤立を検出、中身の妥当性はAIが評価） |

---

## テスト対象別の配置

| 対象 | テスト種別 | 配置 |
|---|---|---|
| domain | 単体 | `tests/domain/` |
| application | 単体（port はテストダブル） | `tests/application/` |
| adapters | 結合 | `tests/adapters/` |
| usecase の受け入れ | 受け入れ（ネイティブ・.feature は参照専用） | `tests/acceptance/` |

---

## 決定ルール

| 種別 | 規約 |
|---|---|
| 必須 | .feature は実行対象ではなく、AIが読んで実装すべき参照専用の仕様書とする |
| 必須 | .feature は spec から生成する（手で書かない） |
| 禁止 | 単体テストが実物の DB・外部サービスに依存する |
| 推奨 | 不変条件はテストダブルなしで検証する |
| 必須 | テストファイル名は test_{対応するspecのdocumentIdをsnake_case化したもの}.py で統一する（domain/adapters/acceptanceの全層に適用。実装モジュール名を由来にした命名は禁止） |
