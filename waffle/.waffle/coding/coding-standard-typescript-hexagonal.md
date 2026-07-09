# coding-standard-typescript-hexagonal

---

## 命名

| 対象 | 規約 |
|---|---|
| ファイル | kebab-case。例: order-repository.ts |
| クラス・型・インターフェース | PascalCase |
| 関数・変数・メソッド | camelCase |
| 定数 | UPPER_SNAKE_CASE（モジュールスコープの真の定数のみ） |
| 業務語彙 | 仕様のユビキタス言語に一致させる（実装都合の言い換えをしない） |
| domain層の識別子 | ユビキタス言語のみで命名し、技術的接尾辞（Impl/DTO/Manager/Helper等）をつけない |
| outbound adapter層の識別子 | 使用する技術を明示してよい。例: SqliteOrderRepository, DrizzleOrderRepository |
| usecase | 動詞＋目的語の業務操作名で命名する。例: RequestPickup, RegisterOrder |
| レイヤー境界を越えるDTO | 境界を越えるための入れ物であると分かる名前にする。例: OrderStatusResponse, RegisterOrderCommand |

---

## スタイル

| 種別 | 規約 |
|---|---|
| 必須 | any型を使わない。型が未確定な場合はunknownを使い、使用箇所で型を絞り込む |
| 必須 | 関数は1つの責務のみを持つ。複数の判断軸を1関数に混ぜない |
| 禁止 | 循環import（相互にimportし合うモジュール） |
| 推奨 | 公開APIの引数・戻り値には明示的な型注釈を付ける（推論に任せない） |

---

## docstring

- **スタイル**: TSDoc
- **対象**: 公開要素（exportされるclass / function / interface）。非公開（内部専用）は任意

### 要約行の書き方

要約行（1行目）は、ソースを開かずに検索・判断できるよう「何をするか・いつ使うか」が分かる語で書く

```
/**
 * 注文の合計金額を計算する。割引適用後の金額を返す。
 *
 * @param order - 対象の注文
 * @returns 割引適用後の合計金額
 */
function calculateTotal(order: Order): Money { /* ... */ }
```

---

## 決定ルール

| 種別 | 規約 |
|---|---|
| 必須 | docstringは公開要素のみ必須とする |
| 禁止 | コードから自明に導出できる情報（型そのものの説明等）だけをdocstringに書く |
| 禁止 | 仕様と異なる語彙で命名する（業務語彙と実装上の命名は常に一致させる） |
