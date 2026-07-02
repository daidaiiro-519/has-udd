//! @spec 01-spec.md#4.1 — テーブル操作（create/describe/list/delete）の単体テスト。

use loom_core::application::usecases::{
    create_table, delete_table, describe_table, get_item, list_tables, put_item,
};
use loom_core::domain::{AttributeValue, DbError, Item, KeySchema, TableDef};
use loom_testkit::InMemoryStorage;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.to_string())
}

fn def(name: &str) -> TableDef {
    TableDef {
        name: name.into(),
        key: KeySchema {
            pk: "id".into(),
            sk: None,
        },
        indexes: vec![],
        ttl_attr: None,
    }
}

fn item(id: &str) -> Item {
    let mut it = Item::new();
    it.insert("id".into(), s(id));
    it
}

#[test]
fn create_then_describe_round_trips() {
    let engine = InMemoryStorage::new();
    let d = def("orders");
    create_table(&engine, &d).expect("create_table");
    let got = describe_table(&engine, "orders").expect("describe_table");
    assert_eq!(got, d);
}

#[test]
fn create_duplicate_is_resource_in_use() {
    let engine = InMemoryStorage::new();
    create_table(&engine, &def("orders")).expect("create");
    let err = create_table(&engine, &def("orders")).expect_err("duplicate must fail");
    assert!(
        matches!(err, DbError::ResourceInUse(_)),
        "expected ResourceInUse, got {err:?}"
    );
}

#[test]
fn describe_missing_is_resource_not_found() {
    let engine = InMemoryStorage::new();
    let err = describe_table(&engine, "ghost").expect_err("must fail");
    assert!(
        matches!(err, DbError::ResourceNotFound(_)),
        "expected ResourceNotFound, got {err:?}"
    );
}

#[test]
fn list_tables_returns_sorted_names() {
    let engine = InMemoryStorage::new();
    for name in ["zebra", "alpha", "middle"] {
        create_table(&engine, &def(name)).expect("create");
    }
    let names = list_tables(&engine).expect("list_tables");
    assert_eq!(names, vec!["alpha", "middle", "zebra"]);
}

#[test]
fn delete_table_removes_definition_and_items() {
    let engine = InMemoryStorage::new();
    create_table(&engine, &def("orders")).expect("create");
    put_item(&engine, "orders", &item("a"), None).expect("put");

    delete_table(&engine, "orders").expect("delete_table");
    assert!(matches!(
        describe_table(&engine, "orders"),
        Err(DbError::ResourceNotFound(_))
    ));

    // 同名で作り直しても旧データが蘇らないこと（DynamoDB の DeleteTable 準拠）
    create_table(&engine, &def("orders")).expect("re-create");
    let got = get_item(&engine, "orders", &s("a"), None).expect("get");
    assert_eq!(got, None, "old items must not resurrect");
}

#[test]
fn delete_missing_table_is_resource_not_found() {
    let engine = InMemoryStorage::new();
    assert!(matches!(
        delete_table(&engine, "ghost"),
        Err(DbError::ResourceNotFound(_))
    ));
}

#[test]
fn invalid_table_names_are_rejected() {
    let engine = InMemoryStorage::new();
    let long = "x".repeat(256);
    for bad in ["", "ab", "bad name", "loom:meta", "日本語", &long] {
        let err = create_table(&engine, &def(bad)).expect_err("must reject");
        assert!(
            matches!(err, DbError::Validation(_)),
            "name {bad:?}: expected Validation, got {err:?}"
        );
    }
    // 境界は受理（3 文字・255 文字・許可記号）
    for ok in ["abc", "a-b_c.d"] {
        create_table(&engine, &def(ok)).expect("must accept");
    }
    create_table(&engine, &def(&"y".repeat(255))).expect("must accept 255 chars");
}

#[test]
fn item_ops_on_missing_table_are_resource_not_found() {
    let engine = InMemoryStorage::new();
    assert!(matches!(
        put_item(&engine, "ghost", &item("a"), None),
        Err(DbError::ResourceNotFound(_))
    ));
    assert!(matches!(
        get_item(&engine, "ghost", &s("a"), None),
        Err(DbError::ResourceNotFound(_))
    ));
}
