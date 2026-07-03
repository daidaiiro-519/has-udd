//! @spec 01-spec.md#5.4 — ProjectionExpression（取得属性をパスのリストで指定）。
//!
//! 適合方針（DynamoDB 準拠）:
//! - 存在しないパスは黙って省く（1 つも無ければ空 item — 項目自体は返る）
//! - 入れ子パス（addr.city）は構造を保って返す・リスト添字は詰めて返す
//! - パス重複（a と a.b の同時指定）は ValidationError
//! - get / query / scan すべてで使える（JOIN は select が §10.6 で既存）

use loom_core::application::usecases::{
    create_table, get_item, put_item, query, scan, ExprInput, ProjectionInput, QueryOptions,
    ScanOptions,
};
use loom_core::domain::expr::{parse_projection, project, ExprContext};
use loom_core::domain::{AttributeValue, DbError, Item, KeySchema, Number, TableDef};
use loom_testkit::InMemoryStorage;
use std::collections::BTreeMap;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.into())
}
fn n(v: &str) -> AttributeValue {
    AttributeValue::N(Number(v.into()))
}

/// { id, name, amount, addr: {city, zip}, tags: ["red","blue","green"] }
fn sample_item(id: &str) -> Item {
    let mut addr = BTreeMap::new();
    addr.insert("city".to_string(), s("tokyo"));
    addr.insert("zip".to_string(), s("100-0001"));
    let mut it = Item::new();
    it.insert("id".into(), s(id));
    it.insert("name".into(), s("loomdb"));
    it.insert("amount".into(), n("1200"));
    it.insert("addr".into(), AttributeValue::M(addr));
    it.insert(
        "tags".into(),
        AttributeValue::L(vec![s("red"), s("blue"), s("green")]),
    );
    it
}

fn project_str(expr: &str, item: &Item, names: &[(&str, &str)]) -> Result<Item, DbError> {
    let paths = parse_projection(expr)?;
    let names: BTreeMap<String, String> = names
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let values = BTreeMap::new();
    project(
        &paths,
        item,
        &ExprContext {
            names: &names,
            values: &values,
        },
    )
}

#[test]
fn projects_top_level_nested_and_indexed_paths() {
    let it = sample_item("d1");

    // トップレベル: 指定した属性だけが返る
    let out = project_str("name, amount", &it, &[]).expect("project");
    assert_eq!(out.len(), 2);
    assert_eq!(out["name"], s("loomdb"));
    assert_eq!(out["amount"], n("1200"));

    // 入れ子: 構造を保つ（addr.city だけ・zip は含まない）
    let out = project_str("addr.city", &it, &[]).expect("project");
    let AttributeValue::M(addr) = &out["addr"] else {
        panic!("addr must be M, got {:?}", out.get("addr"));
    };
    assert_eq!(addr.len(), 1);
    assert_eq!(addr["city"], s("tokyo"));

    // リスト添字: 指定した要素だけを**詰めて**返す（DynamoDB 準拠）
    let out = project_str("tags[0], tags[2]", &it, &[]).expect("project");
    assert_eq!(
        out["tags"],
        AttributeValue::L(vec![s("red"), s("green")]) // [1] が抜けて詰まる
    );

    // #name プレースホルダ
    let out = project_str("#n", &it, &[("#n", "name")]).expect("project");
    assert_eq!(out.len(), 1);
    assert_eq!(out["name"], s("loomdb"));
}

#[test]
fn missing_paths_are_silently_omitted() {
    let it = sample_item("d1");
    let out = project_str("name, ghost, addr.nope, tags[9]", &it, &[]).expect("project");
    assert_eq!(out.len(), 1); // name だけ
                              // 1 つも一致しなければ空 item（エラーではない）
    let out = project_str("ghost", &it, &[]).expect("project");
    assert!(out.is_empty());
}

#[test]
fn invalid_projections_are_validation_errors() {
    let it = sample_item("d1");
    for bad in [
        "",                // 空
        "name,",           // 末尾カンマ
        "a AND b",         // パス以外のトークン
        "addr, addr.city", // 重複（プレフィックス関係）
        "name, name",      // 完全重複
    ] {
        let r = project_str(bad, &it, &[]);
        assert!(matches!(r, Err(DbError::Validation(_))), "{bad:?}: {r:?}");
    }
    // 未知の #name も ValidationError
    let r = project_str("#nope", &it, &[]);
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
}

fn seeded_engine() -> InMemoryStorage {
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
    for id in ["d1", "d2"] {
        put_item(&e, "docs", &sample_item(id), None).expect("put");
    }
    e
}

fn projection(expr: &str) -> ProjectionInput {
    ProjectionInput {
        expression: expr.into(),
        names: BTreeMap::new(),
        values: BTreeMap::new(),
    }
}

#[test]
fn get_query_scan_apply_projection() {
    let e = seeded_engine();

    // get_item
    let got = get_item(
        &e,
        "docs",
        &s("d1"),
        None,
        Some(&projection("name, addr.city")),
    )
    .expect("get")
    .expect("item");
    assert_eq!(got.len(), 2);
    assert!(got.contains_key("name") && got.contains_key("addr"));

    // query（キー属性も明示しなければ返らない = DynamoDB 準拠）
    let page = query(
        &e,
        "docs",
        &ExprInput {
            expression: "id = :i".into(),
            names: BTreeMap::new(),
            values: [(":i".to_string(), s("d1"))].into_iter().collect(),
        },
        &QueryOptions {
            projection: Some(projection("amount")),
            ..QueryOptions::default()
        },
    )
    .expect("query");
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].len(), 1);
    assert_eq!(page.items[0]["amount"], n("1200"));

    // scan
    let page = scan(
        &e,
        "docs",
        &ScanOptions {
            projection: Some(projection("id")),
            ..ScanOptions::default()
        },
    )
    .expect("scan");
    assert_eq!(page.items.len(), 2);
    assert!(page.items.iter().all(|it| it.len() == 1));
}
