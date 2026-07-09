# architecture-python-hexagonal

---

## レイヤーと依存方向

### 様式

ポートとアダプター（ヘキサゴナル）

| レイヤー | 責務 | 依存してよい先 |
|---|---|---|
| domain | ドメインモデル・不変条件・値 |  |
| application | usecase の調整・トランザクション境界 | domain / ports |
| ports | application が要求する抽象（driven interface） | domain |
| inbound adapter | 外部からの入口（driving：API・CLI 等） | application |
| outbound adapter | 外部への出口（driven：DB・外部サービス） | ports |

---

## ディレクトリ構成

```
src/{package}/
  domain/
    model/          value-object, entity, aggregate
    services/       domain-service
  application/
    usecases/       1 usecase = 1 module
    ports/          driven interface の定義
  adapters/
    inbound/        driving（api, cli, ...）
    outbound/       driven（db, external, ...）
  shared/           共通（エラー・結果型 等）
```

### 合成ルート（結線・DI）

inbound adapter の起動点にのみ置く

---

## 概念 → 実現形

| 概念 | 配置 | 形（決定レベル） |
|---|---|---|
| `usecase` | application/usecases | application service・エントリメソッド1つ・ドメインは port 経由で呼ぶ |
| `aggregate` | domain/model | 整合性境界を持つクラス・不変条件をメソッド内で強制・コマンドはメソッド・永続化は repository 経由 |
| `entity` | domain/model | 同一性は id・集約の内側でのみ可変 |
| `value-object` | domain/model | 不変（frozen dataclass）・値等価 |
| `domain-service` | domain/services | ステートレス・複数集約を跨る計算 |
| `repository` | application/ports（interface）＋adapters/outbound（impl） | aggregate の load/save・集約1つに1リポジトリ |
| `port` | application/ports | application が要求する driven インターフェース（ABC / Protocol） |
| `inbound-adapter` | adapters/inbound | 外部入力を application 呼び出しへ変換・ロジックを持たない |
| `outbound-adapter` | adapters/outbound | port / repository を実装・外部ライブラリをここに閉じ込める |

---

## 規約（守るべきルール）

| 種別 | 規約 |
|---|---|
| 必須 | 依存は内向きのみ |
| 必須 | 外部 I/O・外部ライブラリは outbound adapter（port 実装）に閉じ込める |
| 必須 | 合成ルート（結線・DI）は inbound adapter の起動点にのみ置く |
| 必須 | 依存（port / repository）はコンストラクタ注入で受け取る（生成は合成ルートのみ） |
| 必須 | application 境界は結果型（Result 等）で成否を返す・domain は不変条件違反をドメイン例外で表す |
| 必須 | 失敗は識別可能なエラーコード（定数文字列）を伴う結果型で返す（メッセージ文字列のみは不可） |
| 禁止 | domain / application が外部ライブラリ（DB・HTTP 等）を直接 import する |
| 禁止 | domain 層での副作用（I/O・グローバル可変状態） |
| 禁止 | ドメイン例外の握り潰し（境界で結果型に写像する） |
| 必須 | 配線（どのportにどのadapterを使うか）はコンポジションルート（起動処理）の1箇所に集約する。個々のusecase・adapterの中でアダプターの具体クラスをnewしない |
| 禁止 | コンポジションルート自体に業務ロジックを書く（配線専用に保つ） |
| 必須 | outbound adapterで発生した技術的例外（DB接続エラー等）は、境界を越える前にport契約が定める失敗の形（Result等）に翻訳する。ライブラリ固有の例外型をapplication/domainまで伝播させない |
| 必須 | domainが投げる例外は業務ルール違反のみを表す（業務語彙の専用例外型）。技術的失敗の翻訳先として使わない |
| 禁止 | レイヤー境界を越えてdomainオブジェクト（entity・aggregate）をそのまま渡す。外部に出る形は必要な値だけをコピーした専用のDTOにする |
| 必須 | ロギング・キャッシュ・パフォーマンス計測のような純粋に技術的な横断的関心事は、domain/applicationのコードに書かず、外側（デコレーター・ミドルウェア等）で一括して適用する |
| 必須 | 監査ログ・認可のような業務要件としての横断的関心事は、「何を記録すべきか・誰が実行を許可されるか」の判断をdomain/applicationが持ち、実際の記録・認証手段はport経由でoutbound adapterに委譲する |
| 禁止 | 認可チェックをdomain層の集約メソッドの中に直接書く（「誰がログインしているか」という技術的な認証の仕組みにdomain層が依存し、依存方向が逆転する） |

---

## サブドメイン別の厚み

| Category | 実装の厚み |
|---|---|
| 中核 | 厚い設計（明示的なドメインモデル）。トランザクションスクリプト・アクティブレコードは使わない（業務ルールが複雑になるほど、これらは同じ判断ロジックの重複・不整合を招くため） |
| 一般 | ライブラリを adapter で薄く包む |
| 補完 | 最小のトランザクションスクリプト（またはデータ構造が複雑ならアクティブレコード）。唯一の規律は、一連の更新処理が完全に成功するか完全に失敗するかのどちらかで終わること（トランザクション管理）。業務ルールが複雑化してきたら中核と同じ厚い設計への移行を検討する |
