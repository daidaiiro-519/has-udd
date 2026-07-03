//! @spec 01-spec.md#2.2 / #5.3 / #5.5 — 集合型 SS / NS / BS。
//!
//! 適合方針（DynamoDB 準拠）:
//! - 集合は空にできない・要素は一意（NS は**数値として**一意: "1.0" と "1" は同一）
//! - 等価は順序に依存しない（正規化: 整列＋重複除去を構築時に保証）
//! - ADD = 集合和（欠落属性は新規作成）・DELETE = 集合差（空になったら属性ごと削除）
//! - contains は要素判定・size は要素数・attribute_type は "SS"/"NS"/"BS"
//! - 集合はキー属性に使えない

use loom_core::application::usecases::{create_table, get_item, put_item, update_item, ExprInput};
use loom_core::domain::expr::{eval, parse_condition, ExprContext};
use loom_core::domain::{AttributeValue, DbError, Item, KeySchema, Number, TableDef};
use loom_testkit::InMemoryStorage;
use std::collections::BTreeMap;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.into())
}
fn n(v: &str) -> AttributeValue {
    AttributeValue::N(Number(v.into()))
}
fn ss(elems: &[&str]) -> AttributeValue {
    AttributeValue::string_set(elems.iter().map(|e| e.to_string()).collect()).expect("ss")
}
fn ns(elems: &[&str]) -> AttributeValue {
    AttributeValue::number_set(elems.iter().map(|e| Number(e.to_string())).collect()).expect("ns")
}
fn bs(elems: &[&[u8]]) -> AttributeValue {
    AttributeValue::binary_set(elems.iter().map(|e| e.to_vec()).collect()).expect("bs")
}

#[test]
fn constructors_normalize_and_reject_empty() {
    // 整列＋重複除去（入力順に依存しない = 等価が構造比較で成立する）
    assert_eq!(ss(&["b", "a", "b"]), ss(&["a", "b"]));
    assert_eq!(bs(&[b"\x02", b"\x01", b"\x02"]), bs(&[b"\x01", b"\x02"]));
    // NS は数値として一意（"1.0" と "1" は同じ要素）・数値順に整列
    assert_eq!(ns(&["10", "2", "1.0", "1"]), ns(&["1", "2", "10"]));

    // 空集合は作れない
    assert!(matches!(
        AttributeValue::string_set(vec![]),
        Err(DbError::Validation(_))
    ));
    assert!(matches!(
        AttributeValue::number_set(vec![]),
        Err(DbError::Validation(_))
    ));
    assert!(matches!(
        AttributeValue::binary_set(vec![]),
        Err(DbError::Validation(_))
    ));
    // NS の要素は正しい数値でなければならない
    assert!(matches!(
        AttributeValue::number_set(vec![Number("abc".into())]),
        Err(DbError::Validation(_))
    ));
}

fn engine_with_docs() -> InMemoryStorage {
    let e = InMemoryStorage::new();
    create_table(
        &e,
        &TableDef {
            name: "docs".into(),
            key: KeySchema {
                pk: "id".into(),
                sk: None,
            },
            indexes: vec![],
            ttl_attr: None,
        },
    )
    .expect("create");
    e
}

#[test]
fn sets_round_trip_through_storage() {
    let e = engine_with_docs();
    let mut it = Item::new();
    it.insert("id".into(), s("d1"));
    it.insert("tags".into(), ss(&["red", "blue"]));
    it.insert("scores".into(), ns(&["1", "2.5"]));
    it.insert("blobs".into(), bs(&[b"\x00\xff", b"\x01"]));
    put_item(&e, "docs", &it, None).expect("put");
    let got = get_item(&e, "docs", &s("d1"), None, None)
        .expect("get")
        .unwrap();
    assert_eq!(got, it);
}

#[test]
fn sets_cannot_be_key_attributes() {
    let e = engine_with_docs();
    let mut it = Item::new();
    it.insert("id".into(), ss(&["not-a-key"]));
    let r = put_item(&e, "docs", &it, None);
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
}

fn eval_str(expr: &str, item: &Item, values: &[(&str, AttributeValue)]) -> Result<bool, DbError> {
    let ast = parse_condition(expr)?;
    let names = BTreeMap::new();
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

/// contains / size / attribute_type / 等価が集合で動く（表駆動）
#[test]
fn condition_functions_on_sets() {
    let mut it = Item::new();
    it.insert("tags".into(), ss(&["red", "blue"]));
    it.insert("scores".into(), ns(&["1", "2.5"]));
    it.insert("blobs".into(), bs(&[b"\x01"]));

    #[allow(clippy::type_complexity)]
    let cases: &[(&str, Vec<(&str, AttributeValue)>, bool)] = &[
        // contains = 要素判定
        ("contains(tags, :v)", vec![(":v", s("red"))], true),
        ("contains(tags, :v)", vec![(":v", s("green"))], false),
        // NS は数値等価（"1.0" は要素 "1" に一致）
        ("contains(scores, :v)", vec![(":v", n("1.0"))], true),
        ("contains(scores, :v)", vec![(":v", n("3"))], false),
        (
            "contains(blobs, :v)",
            vec![(":v", AttributeValue::B(vec![1]))],
            true,
        ),
        // 型不一致の needle は偽（エラーにしない）
        ("contains(tags, :v)", vec![(":v", n("1"))], false),
        // size = 要素数
        ("size(tags) = :n", vec![(":n", n("2"))], true),
        // attribute_type
        ("attribute_type(tags, :t)", vec![(":t", s("SS"))], true),
        ("attribute_type(scores, :t)", vec![(":t", s("NS"))], true),
        ("attribute_type(blobs, :t)", vec![(":t", s("BS"))], true),
        ("attribute_type(tags, :t)", vec![(":t", s("L"))], false),
        // 等価は順序非依存＋NS は数値等価
        ("tags = :v", vec![(":v", ss(&["blue", "red"]))], true),
        ("scores = :v", vec![(":v", ns(&["2.50", "1.0"]))], true),
        ("tags = :v", vec![(":v", ss(&["red"]))], false),
        // 集合に順序比較は無い（型不一致扱いで偽）
        ("tags < :v", vec![(":v", ss(&["zzz"]))], false),
    ];
    for (expr, values, expected) in cases {
        let got = eval_str(expr, &it, values).unwrap_or_else(|e| panic!("{expr}: {e}"));
        assert_eq!(got, *expected, "expr: {expr}");
    }
}

fn update(
    e: &InMemoryStorage,
    expr: &str,
    values: &[(&str, AttributeValue)],
) -> Result<Item, DbError> {
    update_item(
        e,
        "docs",
        &s("d1"),
        None,
        &ExprInput {
            expression: expr.into(),
            names: BTreeMap::new(),
            values: values
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        },
        None,
    )
}

/// ADD = 集合和。欠落属性には新規作成・NS は数値として重複除去
#[test]
fn add_performs_set_union() {
    let e = engine_with_docs();
    let mut it = Item::new();
    it.insert("id".into(), s("d1"));
    it.insert("tags".into(), ss(&["red"]));
    put_item(&e, "docs", &it, None).expect("put");

    let out = update(&e, "ADD tags :t", &[(":t", ss(&["blue", "red"]))]).expect("add");
    assert_eq!(out["tags"], ss(&["blue", "red"]));

    // 欠落属性への ADD は集合を新規作成
    let out = update(&e, "ADD scores :s", &[(":s", ns(&["2", "1.0"]))]).expect("add new");
    assert_eq!(out["scores"], ns(&["1", "2"]));

    // 既存 NS との和も数値として一意（"1" と "1.0" は同じ）
    let out = update(&e, "ADD scores :s", &[(":s", ns(&["1.0", "3"]))]).expect("add ns");
    assert_eq!(out["scores"], ns(&["1", "2", "3"]));

    // 型不一致（SS の属性に NS を ADD）は ValidationError
    let r = update(&e, "ADD tags :t", &[(":t", ns(&["1"]))]);
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
    // N の属性に集合を ADD も ValidationError
    let r = update(&e, "ADD tags :t", &[(":t", n("1"))]);
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
}

/// DELETE = 集合差。空になったら属性ごと削除・欠落属性には no-op
#[test]
fn delete_performs_set_difference() {
    let e = engine_with_docs();
    let mut it = Item::new();
    it.insert("id".into(), s("d1"));
    it.insert("tags".into(), ss(&["red", "blue", "green"]));
    it.insert("scores".into(), ns(&["1", "2"]));
    put_item(&e, "docs", &it, None).expect("put");

    let out = update(&e, "DELETE tags :t", &[(":t", ss(&["blue", "absent"]))]).expect("delete");
    assert_eq!(out["tags"], ss(&["green", "red"]));

    // 全要素を消すと属性ごと消える（空集合は存在しない）
    let out = update(&e, "DELETE scores :s", &[(":s", ns(&["2.0", "1"]))]).expect("delete all");
    assert!(!out.contains_key("scores"));

    // 欠落属性への DELETE は no-op
    let out = update(&e, "DELETE ghost :t", &[(":t", ss(&["x"]))]).expect("delete missing");
    assert!(!out.contains_key("ghost"));

    // 集合以外のオペランドは ValidationError
    let r = update(&e, "DELETE tags :t", &[(":t", s("red"))]);
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
    // 型不一致（SS の属性から NS を引く）も ValidationError
    let r = update(&e, "DELETE tags :t", &[(":t", ns(&["1"]))]);
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
}
