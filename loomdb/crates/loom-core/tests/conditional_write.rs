//! @spec 01-spec.md#4.2 — 条件付き書込（put/delete）と楽観ロック（§7 楽観ロック）。

use loom_core::application::usecases::{
    create_table, delete_item, get_item, put_item, ConditionInput,
};
use loom_core::domain::{AttributeValue, DbError, Item, KeySchema, Number, TableDef};
use loom_testkit::InMemoryStorage;
use std::collections::BTreeMap;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.into())
}
fn n(v: &str) -> AttributeValue {
    AttributeValue::N(Number(v.into()))
}

fn engine() -> InMemoryStorage {
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
    .expect("create_table");
    e
}

fn doc(id: &str, version: &str) -> Item {
    let mut it = Item::new();
    it.insert("id".into(), s(id));
    it.insert("version".into(), n(version));
    it
}

fn cond(expression: &str, values: &[(&str, AttributeValue)]) -> ConditionInput {
    ConditionInput {
        expression: expression.into(),
        names: BTreeMap::new(),
        values: values
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    }
}

/// attribute_not_exists(pk) イディオム: 「無ければ作る」が 2 回目は失敗する
#[test]
fn put_if_not_exists_idiom() {
    let e = engine();
    let c = cond("attribute_not_exists(id)", &[]);
    put_item(&e, "docs", &doc("a", "1"), Some(&c)).expect("first put");
    let err = put_item(&e, "docs", &doc("a", "9"), Some(&c)).expect_err("second must fail");
    assert!(matches!(err, DbError::ConditionalCheckFailed));
    // 失敗した put は何も変えていない（ロールバック）
    let got = get_item(&e, "docs", &s("a"), None, None)
        .expect("get")
        .unwrap();
    assert_eq!(got.get("version"), Some(&n("1")));
}

/// 楽観ロック: version 一致でのみ置換できる
#[test]
fn optimistic_lock_via_version_condition() {
    let e = engine();
    put_item(&e, "docs", &doc("a", "1"), None).expect("seed");

    // version=1 を期待 → 成功して version=2 に
    put_item(
        &e,
        "docs",
        &doc("a", "2"),
        Some(&cond("version = :expected", &[(":expected", n("1"))])),
    )
    .expect("cas success");

    // もう一度 version=1 を期待 → 失敗（すでに 2）
    let err = put_item(
        &e,
        "docs",
        &doc("a", "3"),
        Some(&cond("version = :expected", &[(":expected", n("1"))])),
    )
    .expect_err("stale cas must fail");
    assert!(matches!(err, DbError::ConditionalCheckFailed));

    let got = get_item(&e, "docs", &s("a"), None, None)
        .expect("get")
        .unwrap();
    assert_eq!(got.get("version"), Some(&n("2")));
}

/// delete: 条件なしで存在しないキー → Ok(None)（DynamoDB 準拠の no-op）
#[test]
fn delete_missing_without_condition_is_noop() {
    let e = engine();
    let old = delete_item(&e, "docs", &s("ghost"), None, None).expect("delete");
    assert_eq!(old, None);
}

/// delete: attribute_exists 条件は、存在しなければ ConditionalCheckFailed
#[test]
fn delete_missing_with_exists_condition_fails() {
    let e = engine();
    let err = delete_item(
        &e,
        "docs",
        &s("ghost"),
        None,
        Some(&cond("attribute_exists(id)", &[])),
    )
    .expect_err("must fail");
    assert!(matches!(err, DbError::ConditionalCheckFailed));
}

/// delete: 条件成立なら削除して旧 item を返す（ALL_OLD 相当）
#[test]
fn conditional_delete_returns_old_item() {
    let e = engine();
    put_item(&e, "docs", &doc("a", "7"), None).expect("seed");

    let old = delete_item(
        &e,
        "docs",
        &s("a"),
        None,
        Some(&cond("version = :v", &[(":v", n("7"))])),
    )
    .expect("delete");
    assert_eq!(old, Some(doc("a", "7")));
    assert_eq!(
        get_item(&e, "docs", &s("a"), None, None).expect("get"),
        None
    );
}

/// delete: 条件不成立なら何も消えない
#[test]
fn failed_conditional_delete_changes_nothing() {
    let e = engine();
    put_item(&e, "docs", &doc("a", "7"), None).expect("seed");
    let err = delete_item(
        &e,
        "docs",
        &s("a"),
        None,
        Some(&cond("version = :v", &[(":v", n("999"))])),
    )
    .expect_err("must fail");
    assert!(matches!(err, DbError::ConditionalCheckFailed));
    assert!(get_item(&e, "docs", &s("a"), None, None)
        .expect("get")
        .is_some());
}
