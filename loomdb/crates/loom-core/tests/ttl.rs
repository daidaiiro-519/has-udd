//! @spec 01-spec.md#8 — TTL: 読取時失効（論理削除）と sweep_expired（物理削除）。
//!
//! 失効規則: ttl 属性（N・epoch 秒）が **now 以下**なら失効。属性なし・N 以外は対象外。
//! テストは testkit の固定時計（set_now）で決定的に検証する。

use loom_core::application::usecases::{
    create_table, get_item, put_item, query, scan, sweep_expired, transact_get, KeyConditionInput,
    KeyRef, QueryOptions, ScanOptions,
};
use loom_core::domain::{AttributeValue, IndexDef, Item, KeySchema, Number, Projection, TableDef};
use loom_testkit::InMemoryStorage;
use std::collections::BTreeMap;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.into())
}
fn n(v: i64) -> AttributeValue {
    AttributeValue::N(Number(v.to_string()))
}

/// events テーブル（pk=id・GSI byStatus・TTL 属性 expiresAt）
fn engine() -> InMemoryStorage {
    let e = InMemoryStorage::new();
    e.set_now(1000);
    create_table(
        &e,
        &TableDef {
            name: "events".into(),
            key: KeySchema {
                pk: "id".into(),
                sk: None,
            },
            indexes: vec![IndexDef {
                name: "byStatus".into(),
                key: KeySchema {
                    pk: "status".into(),
                    sk: None,
                },
                projection: Projection::KeysOnly,
            }],
            ttl_attr: Some("expiresAt".into()),
        },
    )
    .expect("create");
    e
}

fn ev(id: &str, expires_at: Option<i64>) -> Item {
    let mut it = Item::new();
    it.insert("id".into(), s(id));
    it.insert("status".into(), s("open"));
    if let Some(t) = expires_at {
        it.insert("expiresAt".into(), n(t));
    }
    it
}

/// get: 失効した item は None（ttl <= now で失効・属性なしは対象外）
#[test]
fn get_hides_expired_items() {
    let e = engine();
    put_item(&e, "events", &ev("a", Some(1500)), None).expect("put");
    put_item(&e, "events", &ev("forever", None), None).expect("put");

    e.set_now(1499);
    assert!(get_item(&e, "events", &s("a"), None)
        .expect("get")
        .is_some());

    e.set_now(1500); // ちょうど期限 = 失効
    assert!(get_item(&e, "events", &s("a"), None)
        .expect("get")
        .is_none());
    // TTL 属性を持たない item は永続
    assert!(get_item(&e, "events", &s("forever"), None)
        .expect("get")
        .is_some());
}

/// TTL 属性が N 以外なら失効対象にしない
#[test]
fn non_numeric_ttl_is_ignored() {
    let e = engine();
    let mut it = ev("weird", None);
    it.insert("expiresAt".into(), s("not-a-number"));
    put_item(&e, "events", &it, None).expect("put");
    e.set_now(999_999);
    assert!(get_item(&e, "events", &s("weird"), None)
        .expect("get")
        .is_some());
}

/// query / scan: 失効 item は存在しない扱い（limit のカウントにも入らない）
#[test]
fn query_and_scan_hide_expired() {
    let e = engine();
    put_item(&e, "events", &ev("a", Some(1200)), None).expect("put"); // 失効予定
    put_item(&e, "events", &ev("b", None), None).expect("put");
    put_item(&e, "events", &ev("c", None), None).expect("put");
    e.set_now(2000);

    let page = scan(&e, "events", &ScanOptions::default()).expect("scan");
    let ids: Vec<&AttributeValue> = page.items.iter().filter_map(|i| i.get("id")).collect();
    assert_eq!(ids, [&s("b"), &s("c")]);

    // limit=2 でも失効分を数えず live 2 件が返る
    let page = scan(
        &e,
        "events",
        &ScanOptions {
            limit: Some(2),
            ..ScanOptions::default()
        },
    )
    .expect("scan");
    assert_eq!(page.items.len(), 2);

    // 索引経由の query も同様
    let page = query(
        &e,
        "events",
        &KeyConditionInput {
            expression: "status = :st".into(),
            names: BTreeMap::new(),
            values: [(":st".to_string(), s("open"))].into(),
        },
        &QueryOptions {
            index: Some("byStatus".into()),
            ..QueryOptions::default()
        },
    )
    .expect("query");
    assert_eq!(page.items.len(), 2);
}

/// transact_get も失効を隠す
#[test]
fn transact_get_hides_expired() {
    let e = engine();
    put_item(&e, "events", &ev("a", Some(1200)), None).expect("put");
    e.set_now(2000);
    let got = transact_get(
        &e,
        &[KeyRef {
            table: "events".into(),
            pk: s("a"),
            sk: None,
        }],
    )
    .expect("transact_get");
    assert_eq!(got, vec![None]);
}

/// sweep_expired: budget 件まで物理削除し、削除数を返す（索引も同一 txn で掃除）
#[test]
fn sweep_deletes_up_to_budget() {
    let e = engine();
    put_item(&e, "events", &ev("x1", Some(1100)), None).expect("put");
    put_item(&e, "events", &ev("x2", Some(1100)), None).expect("put");
    put_item(&e, "events", &ev("live", Some(9999)), None).expect("put");
    e.set_now(2000);

    assert_eq!(sweep_expired(&e, "events", 1).expect("sweep"), 1); // budget で打ち切り
    assert_eq!(sweep_expired(&e, "events", 10).expect("sweep"), 1); // 残り 1 件
    assert_eq!(sweep_expired(&e, "events", 10).expect("sweep"), 0); // もう無い

    // live は残っている
    assert!(get_item(&e, "events", &s("live"), None)
        .expect("get")
        .is_some());
    // 索引エントリも物理削除済み: 失効前の時刻に巻き戻しても x1/x2 は出ない
    e.set_now(1000);
    let page = query(
        &e,
        "events",
        &KeyConditionInput {
            expression: "status = :st".into(),
            names: BTreeMap::new(),
            values: [(":st".to_string(), s("open"))].into(),
        },
        &QueryOptions {
            index: Some("byStatus".into()),
            ..QueryOptions::default()
        },
    )
    .expect("query");
    let ids: Vec<&AttributeValue> = page.items.iter().filter_map(|i| i.get("id")).collect();
    assert_eq!(ids, [&s("live")]);
}

/// TTL 属性を持たないテーブルでは sweep は何もしない
#[test]
fn sweep_on_table_without_ttl_is_noop() {
    let e = InMemoryStorage::new();
    create_table(
        &e,
        &TableDef {
            name: "plain".into(),
            key: KeySchema {
                pk: "id".into(),
                sk: None,
            },
            indexes: vec![],
            ttl_attr: None,
        },
    )
    .expect("create");
    put_item(&e, "plain", &ev("a", Some(1)), None).expect("put");
    e.set_now(999_999);
    assert_eq!(sweep_expired(&e, "plain", 10).expect("sweep"), 0);
    assert!(get_item(&e, "plain", &s("a"), None).expect("get").is_some());
}
