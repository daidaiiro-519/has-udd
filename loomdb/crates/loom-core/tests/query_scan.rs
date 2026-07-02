//! @spec 01-spec.md#4.3 / #5.1 — query（KeyCondition・ページング・Filter）と scan。
//!
//! 適合の要点: 結果は sk 順・**Limit は Filter 適用「前」**に効く（DynamoDB 準拠）。

use loom_core::application::usecases::{
    create_table, put_item, query, scan, ConditionInput, KeyConditionInput, QueryOptions,
    ScanOptions,
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

fn filter(expression: &str, values: &[(&str, AttributeValue)]) -> ConditionInput {
    ConditionInput {
        expression: expression.into(),
        names: BTreeMap::new(),
        values: values
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    }
}

/// orders: pk=userId(S), sk=orderId(S)。u1 に 5 件・u2 に 1 件。
fn seeded_engine() -> InMemoryStorage {
    let e = InMemoryStorage::new();
    create_table(
        &e,
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
    .expect("create");
    for (uid, oid, amount) in [
        ("u1", "o104", "50"),
        ("u1", "o100", "10"),
        ("u1", "o102", "30"),
        ("u1", "o101", "20"),
        ("u1", "o103", "40"),
        ("u2", "o200", "99"),
    ] {
        let mut it = Item::new();
        it.insert("userId".into(), s(uid));
        it.insert("orderId".into(), s(oid));
        it.insert("amount".into(), n(amount));
        put_item(&e, "orders", &it, None).expect("put");
    }
    e
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

#[test]
fn query_partition_sorted_ascending() {
    let e = seeded_engine();
    let page = query(
        &e,
        "orders",
        &kc("userId = :u", &[(":u", s("u1"))]),
        &QueryOptions::default(),
    )
    .expect("query");
    assert_eq!(
        order_ids(&page.items),
        ["o100", "o101", "o102", "o103", "o104"]
    );
    assert!(page.last_evaluated_key.is_none()); // 全件返した
}

#[test]
fn query_descending_with_scan_forward_false() {
    let e = seeded_engine();
    let page = query(
        &e,
        "orders",
        &kc("userId = :u", &[(":u", s("u1"))]),
        &QueryOptions {
            scan_forward: false,
            ..QueryOptions::default()
        },
    )
    .expect("query");
    assert_eq!(
        order_ids(&page.items),
        ["o104", "o103", "o102", "o101", "o100"]
    );
}

#[test]
fn query_sk_conditions() {
    let e = seeded_engine();
    let cases: Vec<(&str, Vec<(&str, AttributeValue)>, Vec<&str>)> = vec![
        (
            "userId = :u AND orderId > :o",
            vec![(":u", s("u1")), (":o", s("o102"))],
            vec!["o103", "o104"],
        ),
        (
            "userId = :u AND orderId BETWEEN :a AND :b",
            vec![(":u", s("u1")), (":a", s("o101")), (":b", s("o103"))],
            vec!["o101", "o102", "o103"],
        ),
        (
            "userId = :u AND begins_with(orderId, :p)",
            vec![(":u", s("u1")), (":p", s("o10"))],
            vec!["o100", "o101", "o102", "o103", "o104"],
        ),
        (
            "userId = :u AND orderId = :o",
            vec![(":u", s("u1")), (":o", s("o102"))],
            vec!["o102"],
        ),
    ];
    for (expr, values, expected) in cases {
        let page = query(&e, "orders", &kc(expr, &values), &QueryOptions::default())
            .unwrap_or_else(|err| panic!("{expr}: {err}"));
        assert_eq!(order_ids(&page.items), expected, "expr: {expr}");
    }
}

/// N 型 sk は数値順に並ぶ（順序保存エンコードの成果。文字列順なら 10 < 2 になる）
#[test]
fn query_numeric_sk_sorts_numerically() {
    let e = InMemoryStorage::new();
    create_table(
        &e,
        &TableDef {
            name: "scores".into(),
            key: KeySchema {
                pk: "game".into(),
                sk: Some("score".into()),
            },
            indexes: vec![],
            ttl_attr: None,
        },
    )
    .expect("create");
    for v in ["30", "2", "10"] {
        let mut it = Item::new();
        it.insert("game".into(), s("g1"));
        it.insert("score".into(), n(v));
        put_item(&e, "scores", &it, None).expect("put");
    }
    let page = query(
        &e,
        "scores",
        &kc("game = :g", &[(":g", s("g1"))]),
        &QueryOptions::default(),
    )
    .expect("query");
    let scores: Vec<_> = page
        .items
        .iter()
        .map(|it| it.get("score").cloned().unwrap())
        .collect();
    assert_eq!(scores, [n("2"), n("10"), n("30")]); // 数値順

    let page = query(
        &e,
        "scores",
        &kc(
            "game = :g AND score >= :min",
            &[(":g", s("g1")), (":min", n("10"))],
        ),
        &QueryOptions::default(),
    )
    .expect("query");
    assert_eq!(page.items.len(), 2); // 10, 30
}

/// limit + exclusive_start_key で全件を重複なく回収できる
#[test]
fn query_pagination_round_trip() {
    let e = seeded_engine();
    let mut collected = Vec::new();
    let mut start: Option<Vec<u8>> = None;
    let mut guard = 0;
    loop {
        let page = query(
            &e,
            "orders",
            &kc("userId = :u", &[(":u", s("u1"))]),
            &QueryOptions {
                limit: Some(2),
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
        guard += 1;
        assert!(guard < 10, "pagination must terminate");
    }
    assert_eq!(collected, ["o100", "o101", "o102", "o103", "o104"]);
}

/// Limit は Filter の「前」に効く（spec §4.3・DynamoDB 準拠の落とし穴）
#[test]
fn limit_applies_before_filter() {
    let e = seeded_engine();
    // filter は amount >= 40（o103, o104 のみ一致）。limit 2 なら最初のページは
    // o100, o101 を読んで filter で全滅 → items は空だが LEK は返る。
    let first = query(
        &e,
        "orders",
        &kc("userId = :u", &[(":u", s("u1"))]),
        &QueryOptions {
            limit: Some(2),
            filter: Some(filter("amount >= :min", &[(":min", n("40"))])),
            ..QueryOptions::default()
        },
    )
    .expect("query");
    assert!(first.items.is_empty(), "page1 must be empty after filter");
    assert!(first.last_evaluated_key.is_some(), "but more data remains");

    // ページを回しきれば一致分だけがすべて集まる
    let mut collected = Vec::new();
    let mut start: Option<Vec<u8>> = None;
    loop {
        let page = query(
            &e,
            "orders",
            &kc("userId = :u", &[(":u", s("u1"))]),
            &QueryOptions {
                limit: Some(2),
                exclusive_start_key: start.clone(),
                filter: Some(filter("amount >= :min", &[(":min", n("40"))])),
                ..QueryOptions::default()
            },
        )
        .expect("query");
        collected.extend(order_ids(&page.items));
        match page.last_evaluated_key {
            Some(k) => start = Some(k),
            None => break,
        }
    }
    assert_eq!(collected, ["o103", "o104"]);
}

#[test]
fn key_condition_with_name_placeholders() {
    let e = seeded_engine();
    let mut input = kc("#u = :u", &[(":u", s("u2"))]);
    input.names.insert("#u".into(), "userId".into());
    let page = query(&e, "orders", &input, &QueryOptions::default()).expect("query");
    assert_eq!(order_ids(&page.items), ["o200"]);
}

#[test]
fn scan_returns_everything_and_paginates() {
    let e = seeded_engine();
    let page = scan(&e, "orders", &ScanOptions::default()).expect("scan");
    assert_eq!(page.items.len(), 6);
    assert!(page.last_evaluated_key.is_none());

    // limit=4 → 2 ページで全件
    let first = scan(
        &e,
        "orders",
        &ScanOptions {
            limit: Some(4),
            ..ScanOptions::default()
        },
    )
    .expect("scan");
    assert_eq!(first.items.len(), 4);
    let lek = first.last_evaluated_key.expect("must have LEK");
    let second = scan(
        &e,
        "orders",
        &ScanOptions {
            limit: Some(4),
            exclusive_start_key: Some(lek),
            ..ScanOptions::default()
        },
    )
    .expect("scan");
    assert_eq!(second.items.len(), 2);
}

#[test]
fn scan_with_filter() {
    let e = seeded_engine();
    let page = scan(
        &e,
        "orders",
        &ScanOptions {
            filter: Some(filter("amount >= :min", &[(":min", n("50"))])),
            ..ScanOptions::default()
        },
    )
    .expect("scan");
    assert_eq!(page.items.len(), 2); // u1/o104(50) と u2/o200(99)
}

#[test]
fn invalid_key_conditions_are_rejected() {
    let e = seeded_engine();
    // pk 名の不一致 / pk に不等号 / sk 名の不一致 / 未知テーブル
    for (expr, values) in [
        ("wrong = :u", vec![(":u", s("u1"))]),
        ("userId > :u", vec![(":u", s("u1"))]),
        (
            "userId = :u AND wrong > :o",
            vec![(":u", s("u1")), (":o", s("x"))],
        ),
    ] {
        let r = query(&e, "orders", &kc(expr, &values), &QueryOptions::default());
        assert!(
            matches!(r, Err(DbError::Validation(_))),
            "expr {expr:?}: got {r:?}"
        );
    }
    assert!(matches!(
        query(
            &e,
            "ghost",
            &kc("userId = :u", &[(":u", s("u1"))]),
            &QueryOptions::default()
        ),
        Err(DbError::ResourceNotFound(_))
    ));

    // sk なしテーブルへの sk 条件
    create_table(
        &e,
        &TableDef {
            name: "flat".into(),
            key: KeySchema {
                pk: "id".into(),
                sk: None,
            },
            indexes: vec![],
            ttl_attr: None,
        },
    )
    .expect("create");
    let r = query(
        &e,
        "flat",
        &kc("id = :u AND extra > :x", &[(":u", s("a")), (":x", s("b"))]),
        &QueryOptions::default(),
    );
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
}
