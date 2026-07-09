# architecture-typescript-hexagonal

---

## レイヤーと依存方向

### 様式

ポートとアダプター（ヘキサゴナル）

| レイヤー | 責務 | 依存してよい先 |
|---|---|---|
| domain | 業務ロジック・不変条件 |  |
| application | usecaseの編成・ポート呼び出し | domain |
| ports | domain/applicationが外部に要求するインターフェース定義 | domain |
| inbound adapter | 外部プロトコルの受け口（usecaseを呼ぶだけ） | application / ports |
| outbound adapter | portの実装。外部システムとの実際のやり取り | ports |

---

## ディレクトリ構成

```
src/
  domain/
    model/
    value-objects/
    services/
  application/
    usecases/
  ports/
  adapters/
    inbound/
    outbound/
```

### 合成ルート（結線・DI）

inbound adapter の起動点（エントリーポイント）にのみ置く

---

## 概念 → 実現形

| 概念 | 配置 | 形（決定レベル） |
|---|---|---|
| `usecase` | application/usecases | エントリメソッド1つ・ドメインは port 経由で呼ぶ |
| `aggregate` | domain/model | 不変条件はメソッド経由でのみ変更できる形にする（コンストラクタとメソッド内に検証ロジックを閉じ込める）。外部の集約はIDで参照し、直接オブジェクトとして保持しない。集約はできるだけ小さく設計する |
| `entity` | domain/model | 同一性はidで判定する（フィールドの値ではない）。単独では実装しない。必ず集約の内部にのみ存在する |
| `value-object` | domain/value-objects | 構造的等価性・不変（readonlyフィールドのみ・状態変更メソッドを持たない） |
| `domain-service` | domain/services | ステートレス（同じ入力に対して常に同じ結果を返す）。複数集約を単一トランザクションでまとめて変更するための抜け道にはしない（1集約=1トランザクションの原則は業務サービスがあっても変わらない） |
| `repository` | ports（インターフェース）／adapters/outbound（実装） | インターフェースはports、具象はoutbound adapterに置く |
| `port` | ports | インターフェース定義のみ・実装を持たない。ポートは常にコア（domain/application）が「何を必要とするか」の視点で定義し、アダプター側の実装都合に引きずられない |
| `inbound-adapter` | adapters/inbound | 外部プロトコルの受け口。usecaseを呼び出すだけで業務ロジックを持たない |
| `outbound-adapter` | adapters/outbound | portの実装。外部システムとの実際のやり取りを担う |

---

## 規約（守るべきルール）

| 種別 | 規約 |
|---|---|
| 必須 | 依存は常に内向き（adapters→application→domain）。domainは他レイヤーを一切importしない |
| 必須 | DIはコンストラクタ注入で行い、実装（class）ではなくport（インターフェース）型で受け取る |
| 必須 | domain/applicationからadapter/外部ライブラリの型を直接importしない（型はportsで自前定義する） |
| 推奨 | エラー伝播は、想定内の失敗（業務ルール違反等）は戻り値の判別可能な型（Result型相当）で表現し、想定外の異常はthrowする。境界（adapter）で例外を捕捉し外部向けの形に翻訳する（内側→外側方向の翻訳） |
| 必須 | outbound adapter内で発生した技術的失敗（特定ライブラリ・ドライバ固有の例外型）を、そのままdomain/applicationへ伝播させない。抽象化した失敗の形に翻訳してから内側へ渡す（外側→内側方向の翻訳） |
| 禁止 | outbound adapterの実装の詳細（クエリ・スキーマ形式等）がapplication/domainへ漏れること |
| 必須 | レイヤー境界を越えてdomainオブジェクト（entity・aggregate）をそのまま渡さない。外部に出る形は必要な値だけをコピーした専用のDTOにする |
| 禁止 | 個々のusecase・adapterの中でアダプターの具体クラスをnewする。配線（DI）はcompositionRootの1箇所に集約する |

---

## サブドメイン別の厚み

| Category | 実装の厚み |
|---|---|
| 中核 | aggregate/entity/value-object/domain-serviceをフルに用いて厚く設計する |
| 一般 | 既存ライブラリ・外部サービスを薄いadapterで包む程度に留める |
| 補完 | 最小限のusecase内の手続き的処理のみ（別途domainモデルを起こさない） |
