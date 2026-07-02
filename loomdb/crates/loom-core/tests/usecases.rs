//! @spec 01-spec.md#4.2 — put_item / get_item の単体テスト（in-memory fake 使用）。
//! テーブルは DynamoDB 同様、事前に create_table してから名前で参照する。

use loom_core::application::usecases::{create_table, get_item, put_item};
use loom_core::domain::{AttributeValue, DbError, Item, KeySchema, Number, TableDef};
use loom_testkit::InMemoryStorage;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.to_string())
}

fn engine_with_orders() -> InMemoryStorage {
    let engine = InMemoryStorage::new();
    create_table(
        &engine,
        &TableDef {
            name: "orders".into(),
            key: KeySchema {
                pk: "userId".into(),
                sk: Some("orderId".into()),
            },
            indexes: vec![],
            ttl_attr: None,
        },
    )
    .expect("create_table");
    engine
}

fn order_item(uid: &str, oid: &str, amount: &str) -> Item {
    let mut item = Item::new();
    item.insert("userId".into(), s(uid));
    item.insert("orderId".into(), s(oid));
    item.insert("amount".into(), AttributeValue::N(Number(amount.into())));
    item
}

#[test]
fn put_then_get_round_trips() {
    let engine = engine_with_orders();
    let item = order_item("u1", "o100", "1200");

    put_item(&engine, "orders", &item, None).expect("put_item");
    let got = get_item(&engine, "orders", &s("u1"), Some(&s("o100"))).expect("get_item");
    assert_eq!(got, Some(item));
}

#[test]
fn get_missing_returns_none() {
    let engine = engine_with_orders();
    let got = get_item(&engine, "orders", &s("u1"), Some(&s("nope"))).expect("get_item");
    assert_eq!(got, None);
}

#[test]
fn different_sk_are_different_items() {
    let engine = engine_with_orders();
    put_item(&engine, "orders", &order_item("u1", "o100", "1"), None).expect("put");
    put_item(&engine, "orders", &order_item("u1", "o101", "2"), None).expect("put");

    let a = get_item(&engine, "orders", &s("u1"), Some(&s("o100"))).expect("get");
    let b = get_item(&engine, "orders", &s("u1"), Some(&s("o101"))).expect("get");
    assert_ne!(a, b);
    assert!(a.is_some() && b.is_some());
}

#[test]
fn put_without_key_attribute_is_validation_error() {
    let engine = engine_with_orders();
    let mut item = Item::new();
    item.insert("userId".into(), s("u1")); // orderId (sk) が無い

    let err = put_item(&engine, "orders", &item, None).expect_err("must fail");
    assert!(
        matches!(err, DbError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
