# coding-standard-python-hexagonal

---

## 命名

| 対象 | 規約 |
|---|---|
| モジュール / ファイル | スネークケース |
| クラス | パスカルケース |
| 関数・メソッド | スネークケース |
| private（内部限定） | 先頭 _ |
| 語彙 | 仕様（spec）のユビキタス言語に一致させる（勝手な言い換え禁止） |
| domain層の識別子（クラス名・メソッド名） | ユビキタス言語のみで構成する。Impl/DTO/Manager/Helper等の技術的接尾辞を付けない（付いている時点でその概念はドメインの言葉でなく実装都合で存在している兆候） |
| application層の識別子（usecase名） | 動詞＋目的語の業務操作として命名する（例: RequestPickup、CancelOrder） |
| outbound adapter層の識別子 | 使用する技術名を明示してよい（例: PostgresShipmentRepository、InMemoryShipmentRepository）。ポート実装であることが責務そのものなので技術名を隠す理由が無い |
| レイヤー境界を越えるDTOの識別子 | 層の外に出るための入れ物であることが分かる名前にする（例: ShipmentStatusResponse）。ドメインオブジェクトと同じクラス名を使い回さない |

---

## スタイル

| 種別 | 規約 |
|---|---|
| 必須 | 型注釈を公開関数の引数・戻り値に付ける |
| 必須 | 1関数＝1責務 |
| 禁止 | 循環 import |
| 推奨 | 早期 return で条件分岐の入れ子を避ける |

---

## docstring

- **スタイル**: Google スタイル docstring
- **対象**: 公開要素（module / 公開 class / 公開 function）は必須・private は任意

### 要約行の書き方

「何をするか・いつ使うか」を1行の平叙文で。インデックスに載る前提で、検索と判断に効く語を選ぶ。

```
def render_parts(parts: list[dict], data: dict, level: int = 2) -> str:
    """宣言的 x-render の部品配列を Markdown に描画する。

    schema が宣言した部品（table/section/keyvalue 等）を block の値から
    決定的に Markdown へ変換する。書式判断はすべて部品定義側にあり、
    呼び出し側は整形ロジックを持たなくてよい。

    Args:
        parts: RenderMetaSchema に適合する部品宣言の配列。
        data: 描画対象 block の値。
        level: 小見出しの基準レベル。

    Returns:
        描画済み Markdown（部品間は空行区切り）。

    Raises:
        RenderError: 部品種別が未知、または from 参照が解決できない場合。
    """
```

---

## 決定ルール

| 種別 | 規約 |
|---|---|
| 必須 | 公開要素に要約行つき docstring を書く |
| 禁止 | コードから導出できる情報を docstring に書く（型の列挙・実装手順の逐語説明） |
| 禁止 | 仕様と異なる語彙での命名（ユビキタス言語の言い換え） |
| 推奨 | コメントは「なぜ」を書く（「何を」は命名と要約行が表現する） |
