//! @spec 01-spec.md#7 — 二次索引（GSI）: 同一 txn 維持・sparse・後付けバックフィル・
//! index 指定 query。索引は常に強整合（ローカル特権・DynamoDB の結果整合の上位互換）。

use loom_core::application::usecases::{
    create_table, delete_item, delete_table, put_item, query, update_item, update_table,
    KeyConditionInput, QueryOptions, UpdateInput,
};
use loom_core::domain::{
    AttributeValue, DbError, IndexDef, Item, KeySchema, Number, Projection, TableDef,
};
use loom_testkit::InMemoryStorage;
use std::collections::BTreeMap;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.into())
}
fn n(v: &str) -> AttributeValue {
    AttributeValue::N(Number(v.into()))
}

fn by_status_index() -> IndexDef {
    IndexDef {
        name: "byStatus".into(),
        key: KeySchema {
            pk: "status".into(),
            sk: Some("amount".into()),
        },
        projection: Projection::KeysOnly,
    }
}

fn orders_def(indexes: Vec<IndexDef>) -> TableDef {
    TableDef {
        name: "orders".into(),
        key: KeySchema {
            pk: "userId".into(),
            sk: Some("orderId".into()),
        },
        indexes,
        ttl_attr: None,
    }
}

fn order(uid: &str, oid: &str, status: Option<&str>, amount: &str) -> Item {
    let mut it = Item::new();
    it.insert("userId".into(), s(uid));
    it.insert("orderId".into(), s(oid));
    if let Some(st) = status {
        it.insert("status".into(), s(st));
    }
    it.insert("amount".into(), n(amount));
    it
}

fn kc(expression: &str, values: &[(&str, AttributeValue)]) -> KeyConditionInput {
    KeyConditionInput {
        expression: expression.into(),
        names: BTreeMap::new(),
        values: values
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    }
}

fn index_query() -> QueryOptions {
    QueryOptions {
        index: Some("byStatus".into()),
        ..QueryOptions::default()
    }
}

fn order_ids(items: &[Item]) -> Vec<String> {
    items
        .iter()
        .map(|it| match it.get("orderId") {
            Some(AttributeValue::S(v)) => v.clone(),
            other => panic!("unexpected orderId {other:?}"),
        })
        .collect()
}

/// 索引つきテーブル＋データ一式。
/// open: o1(30), o2(10), o3(20) / shipped: o4(99) / status なし: o5
fn seeded() -> InMemoryStorage {
    let e = InMemoryStorage::new();
    create_table(&e, &orders_def(vec![by_status_index()])).expect("create");
    for (oid, status, amount) in [
        ("o1", Some("open"), "30"),
        ("o2", Some("open"), "10"),
        ("o3", Some("open"), "20"),
        ("o4", Some("shipped"), "99"),
        ("o5", None, "77"), // sparse: 索引に載らない
    ] {
        put_item(&e, "orders", &order("u1", oid, status, amount), None).expect("put");
    }
    e
}

/// 索引 query は「索引 pk 等価」で全属性の item を返し、isk（N）の数値順に並ぶ
#[test]
fn query_gsi_returns_full_items_sorted_by_numeric_isk() {
    let e = seeded();
    let page = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("open"))]),
        &index_query(),
    )
    .expect("query");
    // amount 数値順: o2(10), o3(20), o1(30)
    assert_eq!(order_ids(&page.items), ["o2", "o3", "o1"]);
    // 索引経由でも全属性が返る（ローカルは main 参照が安価・強整合）
    assert_eq!(page.items[0].get("userId"), Some(&s("u1")));
}

#[test]
fn query_gsi_with_isk_range_condition() {
    let e = seeded();
    let page = query(
        &e,
        "orders",
        &kc(
            "status = :s AND amount >= :min",
            &[(":s", s("open")), (":min", n("20"))],
        ),
        &index_query(),
    )
    .expect("query");
    assert_eq!(order_ids(&page.items), ["o3", "o1"]); // 20, 30
}

/// sparse index: 索引キー属性を持たない item は索引に載らない
#[test]
fn sparse_items_are_absent_from_index() {
    let e = seeded();
    for status in ["open", "shipped"] {
        let page = query(
            &e,
            "orders",
            &kc("status = :s", &[(":s", s(status))]),
            &index_query(),
        )
        .expect("query");
        assert!(
            !order_ids(&page.items).contains(&"o5".to_string()),
            "o5 (no status) must not appear in the index"
        );
    }
}

/// put 上書きで索引キーが変わったら、旧エントリが消え新エントリが載る（同一 txn 維持）
#[test]
fn put_overwrite_moves_index_entry() {
    let e = seeded();
    // o1 を open → shipped に
    put_item(
        &e,
        "orders",
        &order("u1", "o1", Some("shipped"), "30"),
        None,
    )
    .expect("put");

    let open = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("open"))]),
        &index_query(),
    )
    .expect("query");
    assert_eq!(order_ids(&open.items), ["o2", "o3"]); // o1 が消えた

    let shipped = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("shipped"))]),
        &index_query(),
    )
    .expect("query");
    assert_eq!(order_ids(&shipped.items), ["o1", "o4"]); // 30 < 99
}

/// update_item で索引属性を変更しても索引が追随する
#[test]
fn update_item_maintains_index() {
    let e = seeded();
    update_item(
        &e,
        "orders",
        &s("u1"),
        Some(&s("o2")),
        &UpdateInput {
            expression: "SET status = :new".into(),
            names: BTreeMap::new(),
            values: [(":new".to_string(), s("shipped"))].into(),
        },
        None,
    )
    .expect("update");

    let open = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("open"))]),
        &index_query(),
    )
    .expect("query");
    assert_eq!(order_ids(&open.items), ["o3", "o1"]);
}

/// delete_item で索引からも消える
#[test]
fn delete_item_removes_index_entry() {
    let e = seeded();
    delete_item(&e, "orders", &s("u1"), Some(&s("o3")), None).expect("delete");
    let page = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("open"))]),
        &index_query(),
    )
    .expect("query");
    assert_eq!(order_ids(&page.items), ["o2", "o1"]);
}

/// 後付け GSI（差別化）: 既存データがバックフィルされる
#[test]
fn update_table_adds_index_with_backfill() {
    let e = InMemoryStorage::new();
    create_table(&e, &orders_def(vec![])).expect("create"); // 索引なしで開始
    for (oid, amount) in [("o1", "30"), ("o2", "10")] {
        put_item(&e, "orders", &order("u1", oid, Some("open"), amount), None).expect("put");
    }
    // 後付けで追加 → 既存データが索引に載る
    update_table(&e, "orders", &[by_status_index()], &[]).expect("update_table");
    let page = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("open"))]),
        &index_query(),
    )
    .expect("query");
    assert_eq!(order_ids(&page.items), ["o2", "o1"]); // バックフィル済み・数値順

    // 追加後の書込も維持される
    put_item(&e, "orders", &order("u1", "o9", Some("open"), "5"), None).expect("put");
    let page = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("open"))]),
        &index_query(),
    )
    .expect("query");
    assert_eq!(order_ids(&page.items), ["o9", "o2", "o1"]);
}

/// 索引の削除: 以後の index query は ResourceNotFound
#[test]
fn update_table_removes_index() {
    let e = seeded();
    update_table(&e, "orders", &[], &["byStatus".to_string()]).expect("update_table");
    let r = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("open"))]),
        &index_query(),
    );
    assert!(matches!(r, Err(DbError::ResourceNotFound(_))), "got {r:?}");
}

/// delete_table は索引エントリも掃除する（再作成で蘇らない）
#[test]
fn delete_table_cleans_index_entries() {
    let e = seeded();
    delete_table(&e, "orders").expect("delete_table");
    create_table(&e, &orders_def(vec![by_status_index()])).expect("re-create");
    let page = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("open"))]),
        &index_query(),
    )
    .expect("query");
    assert!(
        page.items.is_empty(),
        "old index entries must not resurrect"
    );
}

/// 索引 query のページング（limit + LEK）
#[test]
fn index_query_pagination() {
    let e = seeded();
    let mut collected = Vec::new();
    let mut start: Option<Vec<u8>> = None;
    loop {
        let page = query(
            &e,
            "orders",
            &kc("status = :s", &[(":s", s("open"))]),
            &QueryOptions {
                index: Some("byStatus".into()),
                limit: Some(1),
                exclusive_start_key: start.clone(),
                ..QueryOptions::default()
            },
        )
        .expect("query");
        collected.extend(order_ids(&page.items));
        match page.last_evaluated_key {
            Some(k) => start = Some(k),
            None => break,
        }
        assert!(collected.len() <= 5, "must terminate");
    }
    assert_eq!(collected, ["o2", "o3", "o1"]);
}

#[test]
fn invalid_index_usage_is_rejected() {
    let e = seeded();
    // 存在しない索引
    let r = query(
        &e,
        "orders",
        &kc("status = :s", &[(":s", s("open"))]),
        &QueryOptions {
            index: Some("ghost".into()),
            ..QueryOptions::default()
        },
    );
    assert!(matches!(r, Err(DbError::ResourceNotFound(_))), "got {r:?}");
    // 索引 query なのにテーブル pk を参照
    let r = query(
        &e,
        "orders",
        &kc("userId = :u", &[(":u", s("u1"))]),
        &index_query(),
    );
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
    // 重複追加・存在しない索引の削除
    let r = update_table(&e, "orders", &[by_status_index()], &[]);
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
    let r = update_table(&e, "orders", &[], &["ghost".to_string()]);
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
}
