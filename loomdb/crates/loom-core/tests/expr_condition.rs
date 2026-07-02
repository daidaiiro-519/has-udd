//! @spec 01-spec.md#5.2 / #5.5 — Condition/Filter 式のパーサ＋評価器（表駆動）。
//!
//! 適合方針: 型不一致・属性欠落の比較は「偽」、構文誤り・未知プレースホルダは
//! `ValidationError`（DynamoDB 準拠）。

use loom_core::domain::expr::{eval, parse_condition, ExprContext};
use loom_core::domain::{AttributeValue, DbError, Item, Number};
use std::collections::BTreeMap;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.into())
}
fn n(v: &str) -> AttributeValue {
    AttributeValue::N(Number(v.into()))
}

/// テスト用アイテム:
/// { status:"active", amount:1200, name:"loomdb", flag:true,
///   addr:{city:"tokyo"}, tags:["red","blue"], bin: b"\x01\x02" }
fn sample_item() -> Item {
    let mut addr = BTreeMap::new();
    addr.insert("city".to_string(), s("tokyo"));
    let mut it = Item::new();
    it.insert("status".into(), s("active"));
    it.insert("amount".into(), n("1200"));
    it.insert("name".into(), s("loomdb"));
    it.insert("flag".into(), AttributeValue::Bool(true));
    it.insert("addr".into(), AttributeValue::M(addr));
    it.insert("tags".into(), AttributeValue::L(vec![s("red"), s("blue")]));
    it.insert("bin".into(), AttributeValue::B(vec![1, 2]));
    it
}

fn eval_str(
    expr: &str,
    item: &Item,
    names: &[(&str, &str)],
    values: &[(&str, AttributeValue)],
) -> Result<bool, DbError> {
    let ast = parse_condition(expr)?;
    let names: BTreeMap<String, String> = names
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let values: BTreeMap<String, AttributeValue> = values
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    eval(
        &ast,
        item,
        &ExprContext {
            names: &names,
            values: &values,
        },
    )
}

/// (式, 値プレースホルダ, 期待値) の表駆動
#[test]
fn condition_truth_table() {
    let item = sample_item();
    #[allow(clippy::type_complexity)]
    let cases: Vec<(&str, Vec<(&str, AttributeValue)>, bool)> = vec![
        // 比較（S）
        ("status = :v", vec![(":v", s("active"))], true),
        ("status = :v", vec![(":v", s("inactive"))], false),
        ("status <> :v", vec![(":v", s("inactive"))], true),
        ("status < :v", vec![(":v", s("b"))], true), // "active" < "b"
        // 比較（N・数値比較 — 文字列比較では壊れるケースを含む）
        ("amount > :v", vec![(":v", n("999"))], true), // 文字列なら "1200" < "999"
        ("amount = :v", vec![(":v", n("1200.00"))], true), // 数値としての等価
        ("amount <= :v", vec![(":v", n("1200"))], true),
        ("amount < :v", vec![(":v", n("1200"))], false),
        // 比較（B）
        (
            "bin < :v",
            vec![(":v", AttributeValue::B(vec![1, 3]))],
            true,
        ),
        // 型不一致 → 偽（エラーにしない）
        ("amount = :v", vec![(":v", s("1200"))], false),
        ("status < :v", vec![(":v", n("1"))], false),
        // 属性欠落 → 偽 / NOT で反転すれば真
        ("ghost = :v", vec![(":v", s("x"))], false),
        ("ghost <> :v", vec![(":v", s("x"))], false),
        ("NOT ghost = :v", vec![(":v", s("x"))], true),
        // BETWEEN（両端含む・N は数値順: 文字列順だと "5" > "40" で壊れる）
        (
            "amount BETWEEN :lo AND :hi",
            vec![(":lo", n("2")), (":hi", n("4000"))],
            true,
        ),
        (
            "amount BETWEEN :lo AND :hi",
            vec![(":lo", n("1200")), (":hi", n("1200"))],
            true,
        ),
        (
            "amount BETWEEN :lo AND :hi",
            vec![(":lo", n("1201")), (":hi", n("9999"))],
            false,
        ),
        // IN
        (
            "status IN (:a, :b)",
            vec![(":a", s("active")), (":b", s("archived"))],
            true,
        ),
        (
            "status IN (:a, :b)",
            vec![(":a", s("x")), (":b", s("y"))],
            false,
        ),
        // AND / OR / NOT と優先順位（AND が OR より強い）
        (
            "status = :x OR status = :a AND amount > :big",
            vec![(":x", s("nope")), (":a", s("active")), (":big", n("1"))],
            true,
        ),
        (
            "(status = :x OR status = :a) AND ghost = :x",
            vec![(":x", s("nope")), (":a", s("active"))],
            false,
        ),
        // 入れ子パス（M / L 添字）
        ("addr.city = :v", vec![(":v", s("tokyo"))], true),
        ("tags[1] = :v", vec![(":v", s("blue"))], true),
        ("tags[9] = :v", vec![(":v", s("blue"))], false),
        // 関数
        ("attribute_exists(status)", vec![], true),
        ("attribute_exists(ghost)", vec![], false),
        ("attribute_not_exists(ghost)", vec![], true),
        ("attribute_type(amount, :t)", vec![(":t", s("N"))], true),
        ("attribute_type(amount, :t)", vec![(":t", s("S"))], false),
        ("begins_with(name, :p)", vec![(":p", s("loom"))], true),
        ("begins_with(name, :p)", vec![(":p", s("dyn"))], false),
        ("contains(name, :v)", vec![(":v", s("omd"))], true), // 部分文字列
        ("contains(tags, :v)", vec![(":v", s("red"))], true), // リスト要素
        ("contains(tags, :v)", vec![(":v", s("green"))], false),
        // size（S は UTF-8 バイト長）
        ("size(name) = :v", vec![(":v", n("6"))], true),
        ("size(tags) = :v", vec![(":v", n("2"))], true),
        ("size(name) > :v", vec![(":v", n("100"))], false),
    ];
    for (expr, values, expected) in cases {
        let got = eval_str(expr, &item, &[], &values)
            .unwrap_or_else(|e| panic!("{expr}: unexpected error {e}"));
        assert_eq!(got, expected, "expr: {expr}");
    }
}

/// マルチバイト文字列の size はバイト長（DynamoDB 準拠）
#[test]
fn size_is_utf8_byte_length() {
    let mut item = Item::new();
    item.insert("jp".into(), s("あいう")); // 3 文字 × 3 バイト
    assert!(eval_str("size(jp) = :v", &item, &[], &[(":v", n("9"))]).unwrap());
}

/// #name プレースホルダの解決（予約語や記号入り属性名のため）
#[test]
fn name_placeholders_resolve() {
    let item = sample_item();
    assert!(eval_str(
        "#s = :v",
        &item,
        &[("#s", "status")],
        &[(":v", s("active"))],
    )
    .unwrap());
}

/// 未知プレースホルダ・構文誤りは ValidationError
#[test]
fn invalid_inputs_are_validation_errors() {
    let item = sample_item();
    for (expr, names, values) in [
        ("#u = :v", vec![], vec![(":v", s("x"))]), // 未知 #name
        ("status = :missing", vec![], vec![]),     // 未知 :value
        ("status = ", vec![], vec![]),             // 途中で終わる
        ("status == :v", vec![], vec![(":v", s("x"))]), // 不正演算子
        ("BETWEEN :a AND :b", vec![], vec![]),     // operand 欠落
        ("status = :v extra", vec![], vec![(":v", s("x"))]), // 余分なトークン
    ] {
        let names: Vec<(&str, &str)> = names;
        let r = eval_str(expr, &item, &names, &values);
        assert!(
            matches!(r, Err(DbError::Validation(_))),
            "expr {expr:?}: expected Validation, got {r:?}"
        );
    }
}
