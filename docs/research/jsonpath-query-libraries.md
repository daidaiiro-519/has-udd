# JSON クエリライブラリ調査メモ

**調査日:** 2026-06-21
**目的:** has-udd の QuerySkills（document.json からの値取得・集計）に使える JsonPath の機能把握

---

## 選定結果 ✅

**採用: `jsonpath-ng`（Python）**

has-udd の実装言語が Python に確定したため、`jsonpath-ng` を採用する。
集計・演算は JSONata ではなく Plain Python で対応する。

---

## 参考: 言語別の代表ライブラリ

JsonPath は RFC 9535 として標準化されており、主要言語に実装がある。

| 言語 | 代表ライブラリ |
|---|---|
| **Python（採用）** | **jsonpath-ng** |
| TypeScript | jsonpath-plus |
| Go | github.com/PaesslerAG/jsonpath |
| Rust | jsonpath-rust |

本資料では JsonPath の**クエリ機能自体**を調査する。

---

## 前提: Schema バリデーションとは別の関心事

```
① Schema バリデーション  → 別資料参照（言語確定後にライブラリ選定）
② JSON クエリ           → 本資料で検討
```

QuerySkills は document.json から値を取り出す・フィルタする・集計するためのクエリ層。
Schema バリデーションとは独立した関心事。

---

## 主要ライブラリ比較

| | jsonpath-plus | JSONata |
|---|---|---|
| npm DL/週 | 415万 | 181万 |
| GitHub Stars | 1,137 | 1,425 |
| できること | クエリ・抽出・フィルタ | クエリ + **変換・集計・演算** |
| 構文 | JsonPath（RFC 9535 ベース） | 独自の式言語 |
| 学習コスト | 低い | 中〜高 |
| パフォーマンス | 高速 | 大規模データで劣化の報告あり |
| TypeScript 対応 | ✅ | ✅ |
| 標準準拠 | RFC 9535 ベース | 独自仕様 |

---

## jsonpath-plus の特徴

```
基本的な JsonPath クエリに特化。シンプルで高速。
標準 JsonPath 仕様に追加演算子を加えた実装。

できること:
  値の抽出     $.content.acceptanceCriteria[*]
  フィルタ     $[?(@.status == "VALIDATED")]
  再帰検索     $..documentId
  配列スライス  $.items[0:3]
  ワイルドカード $.content.*

できないこと:
  集計（count / sum / avg）
  複数ドキュメントをまたぐ変換
  演算（進捗率の計算など）
```

---

## JSONata の特徴

```
JSON クエリ + 変換 + 演算を単一の式言語で表現できる。
クエリ結果をそのまま新しい JSON 構造に変換できる。

できること（jsonpath-plus にない機能）:
  集計    $count(docs[status = "VALIDATED"])
  演算    $count(done) / $count(total) * 100
  変換    docs.{ "id": documentId, "title": content.title }
  条件    status = "VALIDATED" ? "完了" : "未完了"
  結合    複数配列の JOIN 的な操作

注意点:
  - 独自の式言語のため学習コストがある
  - 大規模データ（数千件〜）でパフォーマンスが劣化する報告あり
  - RFC 標準ではなく独自仕様
```

---

## has-udd の QuerySkills で必要な操作の分類

| 操作 | 具体例 | 必要ライブラリ |
|---|---|---|
| 単件取得 | documentId 指定で1件取得 | jsonpath-plus |
| フィルタ | VALIDATED な UsecaseSpec 全件 | jsonpath-plus |
| 逆引き | refs.pbiRef = "pbi-001" な Spec[] | jsonpath-plus |
| フィールド取得 | content.acceptanceCriteria[] | jsonpath-plus |
| **件数集計** | VALIDATED な PBISpec が何件か | **JSONata** |
| **進捗演算** | Sprint の完了率（done/total）| **JSONata** |
| **横断集計** | 複数 SBISpec の status を集計 | **JSONata** |

→ 単純なフィルタ・抽出は jsonpath-plus で十分。集計・演算が必要な箇所は JSONata。

---

## クエリ機能の選定方針（確定）

```
基本クエリ（抽出・フィルタ）: JsonPath（RFC 9535）
集計・演算:                   実装言語標準の配列操作で代替（JSONata は不採用）

理由:
  QuerySkills のアクセスパターン（論点2で確定予定）を見ると
  集計は filter().length 等の標準操作で代替できる範囲に収まる見込み。
  JSONata は独自構文で学習コストが高く、依存を増やす価値がない。
```

---

## JMESPath（参考）

AWS が策定した RFC 標準のクエリ言語。
`jmespath.js` ライブラリで TypeScript 対応あり。
jsonpath-plus より厳密な標準準拠だが機能は同程度。
has-udd では jsonpath-plus の方が拡張性・情報量ともに優位。

---

## 参考リンク

- [jsonpath-plus npm](https://www.npmjs.com/package/jsonpath-plus)
- [jsonpath-plus 公式ドキュメント](https://jsonpath-plus.github.io/JSONPath/docs/ts/)
- [JSONata 公式ドキュメント](https://docs.jsonata.org/)
- [JSONata, JSONPath, JMESPath 比較](https://medium.com/@khileshsahu2007/jsonata-jsonpath-and-jmespath-exploring-capabilities-and-limitations-bf491348022d)
- [npm trends 比較](https://npmtrends.com/jsonata-vs-jsonpath-vs-jsonpath-plus-vs-object-mapper)
