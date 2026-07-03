//! @spec 01-spec.md#5.3 / #5.5 — UpdateExpression の適用（純関数・表駆動）。
//!
//! 適合方針: 右辺の読取はすべて「元の item」に対して行う（宣言順に依存しない）。
//! 不正パス・型不一致・未知プレースホルダは `ValidationError`。

use loom_core::domain::expr::{apply_update, parse_update, ExprContext};
use loom_core::domain::{AttributeValue, DbError, Item, Number};
use std::collections::BTreeMap;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.into())
}
fn n(v: &str) -> AttributeValue {
    AttributeValue::N(Number(v.into()))
}

/// { amount:1200, name:"loomdb", addr:{city:"tokyo"}, tags:["red","blue"], hits:41 }
fn sample_item() -> Item {
    let mut addr = BTreeMap::new();
    addr.insert("city".to_string(), s("tokyo"));
    let mut it = Item::new();
    it.insert("amount".into(), n("1200"));
    it.insert("name".into(), s("loomdb"));
    it.insert("addr".into(), AttributeValue::M(addr));
    it.insert("tags".into(), AttributeValue::L(vec![s("red"), s("blue")]));
    it.insert("hits".into(), n("41"));
    it
}

fn apply(expr: &str, item: &Item, values: &[(&str, AttributeValue)]) -> Result<Item, DbError> {
    let ast = parse_update(expr)?;
    let names = BTreeMap::new();
    let values: BTreeMap<String, AttributeValue> = values
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    apply_update(
        &ast,
        item,
        &ExprContext {
            names: &names,
            values: &values,
        },
    )
}

#[test]
fn set_new_and_overwrite() {
    let out = apply(
        "SET title = :t, amount = :a",
        &sample_item(),
        &[(":t", s("hello")), (":a", n("5"))],
    )
    .unwrap();
    assert_eq!(out.get("title"), Some(&s("hello")));
    assert_eq!(out.get("amount"), Some(&n("5")));
    assert_eq!(out.get("name"), Some(&s("loomdb"))); // 触っていない属性は不変
}

#[test]
fn set_nested_map_value() {
    let out = apply("SET addr.city = :c", &sample_item(), &[(":c", s("osaka"))]).unwrap();
    let AttributeValue::M(addr) = out.get("addr").unwrap() else {
        panic!("addr must stay a map");
    };
    assert_eq!(addr.get("city"), Some(&s("osaka")));

    // 親が存在しないパスへの SET は不正（DynamoDB 準拠）
    let err = apply("SET missing.x = :c", &sample_item(), &[(":c", s("v"))]).unwrap_err();
    assert!(matches!(err, DbError::Validation(_)));
}

#[test]
fn set_list_replace_and_append_beyond_end() {
    let out = apply("SET tags[0] = :v", &sample_item(), &[(":v", s("green"))]).unwrap();
    assert_eq!(
        out.get("tags"),
        Some(&AttributeValue::L(vec![s("green"), s("blue")]))
    );
    // 範囲外の添字は末尾追加（DynamoDB 準拠）
    let out = apply("SET tags[9] = :v", &sample_item(), &[(":v", s("green"))]).unwrap();
    assert_eq!(
        out.get("tags"),
        Some(&AttributeValue::L(vec![s("red"), s("blue"), s("green")]))
    );
}

#[test]
fn set_arithmetic_is_decimal() {
    let out = apply(
        "SET amount = amount + :d",
        &sample_item(),
        &[(":d", n("0.5"))],
    )
    .unwrap();
    assert_eq!(out.get("amount"), Some(&n("1200.5")));

    let out = apply(
        "SET amount = amount - :d",
        &sample_item(),
        &[(":d", n("1300"))],
    )
    .unwrap();
    assert_eq!(out.get("amount"), Some(&n("-100")));

    // 2 進浮動小数では 0.30000000000000004 になる例が、10 進なら正確
    let out = apply(
        "SET x = :a + :b",
        &sample_item(),
        &[(":a", n("0.1")), (":b", n("0.2"))],
    )
    .unwrap();
    assert_eq!(out.get("x"), Some(&n("0.3")));

    // 存在しないパスの読取・N 以外との加算は Validation
    assert!(matches!(
        apply("SET x = ghost + :d", &sample_item(), &[(":d", n("1"))]),
        Err(DbError::Validation(_))
    ));
    assert!(matches!(
        apply("SET x = name + :d", &sample_item(), &[(":d", n("1"))]),
        Err(DbError::Validation(_))
    ));
}

#[test]
fn set_if_not_exists() {
    // 既存 → 現値を維持
    let out = apply(
        "SET amount = if_not_exists(amount, :zero)",
        &sample_item(),
        &[(":zero", n("0"))],
    )
    .unwrap();
    assert_eq!(out.get("amount"), Some(&n("1200")));
    // 欠落 → 既定値
    let out = apply(
        "SET score = if_not_exists(score, :zero)",
        &sample_item(),
        &[(":zero", n("0"))],
    )
    .unwrap();
    assert_eq!(out.get("score"), Some(&n("0")));
}

#[test]
fn set_list_append() {
    let more = AttributeValue::L(vec![s("green")]);
    let out = apply(
        "SET tags = list_append(tags, :more)",
        &sample_item(),
        &[(":more", more.clone())],
    )
    .unwrap();
    assert_eq!(
        out.get("tags"),
        Some(&AttributeValue::L(vec![s("red"), s("blue"), s("green")]))
    );
    // 先頭連結
    let out = apply(
        "SET tags = list_append(:more, tags)",
        &sample_item(),
        &[(":more", more)],
    )
    .unwrap();
    assert_eq!(
        out.get("tags"),
        Some(&AttributeValue::L(vec![s("green"), s("red"), s("blue")]))
    );
    // リスト以外は Validation
    assert!(matches!(
        apply(
            "SET tags = list_append(name, :m)",
            &sample_item(),
            &[(":m", AttributeValue::L(vec![]))]
        ),
        Err(DbError::Validation(_))
    ));
}

#[test]
fn remove_attribute_nested_and_list() {
    let out = apply("REMOVE name", &sample_item(), &[]).unwrap();
    assert!(!out.contains_key("name"));

    let out = apply("REMOVE addr.city", &sample_item(), &[]).unwrap();
    let AttributeValue::M(addr) = out.get("addr").unwrap() else {
        panic!("addr must stay a map");
    };
    assert!(!addr.contains_key("city"));

    // リスト要素の除去は詰める
    let out = apply("REMOVE tags[0]", &sample_item(), &[]).unwrap();
    assert_eq!(out.get("tags"), Some(&AttributeValue::L(vec![s("blue")])));

    // 存在しないパスの REMOVE は no-op
    let out = apply("REMOVE ghost, ghost.deep", &sample_item(), &[]).unwrap();
    assert_eq!(out, sample_item());
}

#[test]
fn add_is_atomic_counter() {
    // 欠落 → 0 起点（DynamoDB 準拠）
    let out = apply("ADD counter :one", &sample_item(), &[(":one", n("1"))]).unwrap();
    assert_eq!(out.get("counter"), Some(&n("1")));
    // 既存 → 加算
    let out = apply("ADD hits :one", &sample_item(), &[(":one", n("1"))]).unwrap();
    assert_eq!(out.get("hits"), Some(&n("42")));
    // 負数も可
    let out = apply("ADD amount :d", &sample_item(), &[(":d", n("-200"))]).unwrap();
    assert_eq!(out.get("amount"), Some(&n("1000")));
    // N 以外への ADD・入れ子パスは Validation
    assert!(matches!(
        apply("ADD name :one", &sample_item(), &[(":one", n("1"))]),
        Err(DbError::Validation(_))
    ));
    assert!(matches!(
        apply("ADD addr.city :one", &sample_item(), &[(":one", n("1"))]),
        Err(DbError::Validation(_))
    ));
}

#[test]
fn delete_requires_set_operands() {
    // DELETE は集合差専用: L 属性 × S オペランドは ValidationError（tests/sets.rs も参照）
    let err = apply("DELETE tags :v", &sample_item(), &[(":v", s("red"))]).unwrap_err();
    assert!(matches!(err, DbError::Validation(_)));
}

#[test]
fn combined_clauses() {
    let out = apply(
        "SET title = :t REMOVE name ADD hits :one",
        &sample_item(),
        &[(":t", s("x")), (":one", n("1"))],
    )
    .unwrap();
    assert_eq!(out.get("title"), Some(&s("x")));
    assert!(!out.contains_key("name"));
    assert_eq!(out.get("hits"), Some(&n("42")));
}

/// 右辺の読取は「元の item」に対して行う（宣言順に依存しない）
#[test]
fn reads_see_original_item() {
    let out = apply(
        "SET copied = amount, amount = :zero",
        &sample_item(),
        &[(":zero", n("0"))],
    )
    .unwrap();
    assert_eq!(out.get("copied"), Some(&n("1200")));
    assert_eq!(out.get("amount"), Some(&n("0")));
}

#[test]
fn invalid_updates_are_validation_errors() {
    for (expr, values) in [
        ("SET x = :missing", vec![]),                    // 未知 :value
        ("SET x = ", vec![]),                            // 途中で終わる
        ("BOGUS x :v", vec![(":v", n("1"))]),            // 不明な句
        ("SET x = :v SET y = :v", vec![(":v", n("1"))]), // 句の重複
        ("", vec![]),                                    // 空
    ] {
        let r = apply(expr, &sample_item(), &values);
        assert!(
            matches!(r, Err(DbError::Validation(_))),
            "expr {expr:?}: expected Validation, got {r:?}"
        );
    }
}
