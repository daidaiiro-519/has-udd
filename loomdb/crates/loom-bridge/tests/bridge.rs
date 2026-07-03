//! loom-bridge — 言語バインディング（loom-node / loom-py）とワイヤ層が共有する
//! JSON API ブリッジのテスト。
//!
//! 設計方針:
//! - JS オブジェクト / Python dict 相当の JSON がそのまま item（型記法 {"S":..} 不要）
//! - values/names は keyCondition・filter・update・condition で **共有**
//!   （DocumentClient 風・使う分だけ参照される）
//! - 数値規則: 整数は正確に N / 浮動小数は最短表現 / 64bit に収まらない・
//!   f64 で表現できない N は **文字列にフォールバック**（精度を黙って壊さない）
//! - LastEvaluatedKey は不透明トークン（hex 文字列）

use loom_bridge::Bridge;
use loom_core::domain::DbError;
use loom_testkit::InMemoryStorage;
use serde_json::{json, Value};

fn bridge() -> Bridge<InMemoryStorage> {
    let b = Bridge::new(InMemoryStorage::new());
    b.create_table(&json!({
        "name": "orders",
        "key": { "pk": "userId", "sk": "orderId" },
        "indexes": [{ "name": "byStatus", "key": { "pk": "status", "sk": "amount" } }]
    }))
    .expect("create_table");
    b
}

fn seed(b: &Bridge<InMemoryStorage>) {
    for (oid, status, amount) in [
        ("o1", "open", 30),
        ("o2", "open", 10),
        ("o3", "shipped", 99),
    ] {
        b.put(
            "orders",
            &json!({ "userId": "u1", "orderId": oid, "status": status, "amount": amount }),
            None,
        )
        .expect("put");
    }
}

/// JS オブジェクト相当の JSON がそのまま item として round-trip する（入れ子込み）
#[test]
fn plain_json_round_trips() {
    let b = bridge();
    let item = json!({
        "userId": "u1", "orderId": "o1",
        "amount": 1200,
        "ratio": 0.5,
        "active": true,
        "note": null,
        "tags": ["red", "blue"],
        "addr": { "city": "tokyo", "zip": "100-0001" }
    });
    b.put("orders", &item, None).expect("put");
    let got = b
        .get("orders", &json!({ "userId": "u1", "orderId": "o1" }))
        .expect("get")
        .expect("item exists");
    assert_eq!(got, item);
}

#[test]
fn get_missing_returns_none() {
    let b = bridge();
    let got = b
        .get("orders", &json!({ "userId": "u1", "orderId": "nope" }))
        .expect("get");
    assert_eq!(got, None);
}

/// 数値の精度規則: 巨大な N は number に入れず文字列で返す（黙って精度を壊さない）
#[test]
fn big_numbers_come_back_as_strings() {
    let b = bridge();
    // update の ADD で f64 に収まらない大きさの N を作る
    b.put(
        "orders",
        &json!({ "userId": "u1", "orderId": "big", "n": 9_007_199_254_740_993i64 }),
        None,
    )
    .expect("put"); // 2^53+1: f64 では表現できないが i64 なら正確
    let got = b
        .get("orders", &json!({ "userId": "u1", "orderId": "big" }))
        .expect("get")
        .unwrap();
    assert_eq!(got["n"], json!(9_007_199_254_740_993i64)); // i64 に収まる → number のまま

    // 38 桁級は i64/f64 のどちらにも収まらない → 文字列で返る
    b.update(
        "orders",
        &json!({ "userId": "u1", "orderId": "big" }),
        &json!({ "update": "SET huge = :h", "values": { ":h": "12345678901234567890123456789012345678" } }),
    )
    .expect("update");
    // 文字列で入れたものは S のまま
    let got = b
        .get("orders", &json!({ "userId": "u1", "orderId": "big" }))
        .expect("get")
        .unwrap();
    assert_eq!(got["huge"], json!("12345678901234567890123456789012345678"));
}

/// 条件付き put: 不成立は ConditionalCheckFailed
#[test]
fn conditional_put() {
    let b = bridge();
    let item = json!({ "userId": "u1", "orderId": "o1", "v": 1 });
    b.put(
        "orders",
        &item,
        Some(&json!({ "condition": "attribute_not_exists(userId)" })),
    )
    .expect("first put");
    let err = b
        .put(
            "orders",
            &item,
            Some(&json!({ "condition": "attribute_not_exists(userId)" })),
        )
        .expect_err("second must fail");
    assert!(matches!(err, DbError::ConditionalCheckFailed));
}

/// update は ALL_NEW を返す。数値の ADD は JSON number で書ける（原子カウンタ）
#[test]
fn update_returns_all_new_and_counts() {
    let b = bridge();
    for _ in 0..2 {
        b.update(
            "orders",
            &json!({ "userId": "u1", "orderId": "page" }),
            &json!({ "update": "ADD hits :one", "values": { ":one": 1 } }),
        )
        .expect("update");
    }
    let got = b
        .get("orders", &json!({ "userId": "u1", "orderId": "page" }))
        .expect("get")
        .unwrap();
    assert_eq!(got["hits"], json!(2));
}

/// delete は旧 item を返す（無ければ None）
#[test]
fn delete_returns_old() {
    let b = bridge();
    seed(&b);
    let old = b
        .delete("orders", &json!({ "userId": "u1", "orderId": "o1" }), None)
        .expect("delete")
        .expect("old item");
    assert_eq!(old["amount"], json!(30));
    let again = b
        .delete("orders", &json!({ "userId": "u1", "orderId": "o1" }), None)
        .expect("delete");
    assert_eq!(again, None);
}

/// query: values は keyCondition と filter で共有（DocumentClient 風）
#[test]
fn query_with_shared_values_and_filter() {
    let b = bridge();
    seed(&b);
    let page = b
        .query(
            "orders",
            &json!({
                "keyCondition": "userId = :u",
                "filter": "amount >= :min",
                "values": { ":u": "u1", ":min": 20 }
            }),
        )
        .expect("query");
    let items = page["items"].as_array().expect("items");
    let ids: Vec<&str> = items
        .iter()
        .map(|i| i["orderId"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["o1", "o3"]); // 30, 99（sk 昇順）
}

/// query: 降順・limit・不透明トークンでのページング
#[test]
fn query_pagination_with_opaque_token() {
    let b = bridge();
    seed(&b);
    let mut collected: Vec<String> = Vec::new();
    let mut start: Option<Value> = None;
    loop {
        let mut params = json!({
            "keyCondition": "userId = :u",
            "values": { ":u": "u1" },
            "scanForward": false,
            "limit": 1
        });
        if let Some(t) = &start {
            params["startKey"] = t.clone();
        }
        let page = b.query("orders", &params).expect("query");
        for item in page["items"].as_array().unwrap() {
            collected.push(item["orderId"].as_str().unwrap().to_string());
        }
        match page.get("lastEvaluatedKey") {
            Some(t) if !t.is_null() => start = Some(t.clone()),
            _ => break,
        }
        assert!(collected.len() <= 4, "must terminate");
    }
    assert_eq!(collected, ["o3", "o2", "o1"]); // orderId 降順
}

/// query: index 指定（isk = N の数値順で返る）
#[test]
fn query_via_index() {
    let b = bridge();
    seed(&b);
    let page = b
        .query(
            "orders",
            &json!({
                "index": "byStatus",
                "keyCondition": "#s = :s",
                "names": { "#s": "status" },
                "values": { ":s": "open" }
            }),
        )
        .expect("query");
    let ids: Vec<&str> = page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["orderId"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["o2", "o1"]); // amount 10, 30
}

/// JOIN も JSON で宣言して実行できる（spec §10.5-B）
#[test]
fn join_via_bridge() {
    let b = bridge();
    seed(&b);
    b.create_table(&json!({ "name": "users", "key": { "pk": "id" } }))
        .expect("create users");
    b.put("users", &json!({ "id": "u1", "name": "Alice" }), None)
        .expect("put user");

    let result = b
        .join(&json!({
            "root": { "table": "orders", "alias": "o" },
            "steps": [
                { "table": "users", "alias": "u", "kind": "inner",
                  "on": [{ "left": "o.userId", "right": "u.id" }] }
            ],
            "filter": "o.amount >= :min",
            "values": { ":min": 20 },
            "select": ["o.orderId", "u.name"]
        }))
        .expect("join");
    let rows = result["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 2); // o1(30), o3(99)
    for row in rows {
        assert_eq!(row["u.name"], json!("Alice"));
        assert!(row.get("o.orderId").is_some());
    }
    assert!(result["warnings"].as_array().unwrap().is_empty());
}

/// テーブル管理: list / updateTable(後付け索引) / deleteTable
#[test]
fn table_management() {
    let b = bridge();
    assert_eq!(b.list_tables().expect("list"), vec!["orders"]);

    seed(&b);
    b.update_table(
        "orders",
        &json!({ "add": [{ "name": "byAmount", "key": { "pk": "amount" } }] }),
    )
    .expect("update_table");
    // 後付け索引がバックフィルされている
    let page = b
        .query(
            "orders",
            &json!({
                "index": "byAmount",
                "keyCondition": "amount = :a",
                "values": { ":a": 30 }
            }),
        )
        .expect("query");
    assert_eq!(page["items"].as_array().unwrap().len(), 1);

    b.delete_table("orders").expect("delete");
    assert!(b.list_tables().expect("list").is_empty());
}

// ---------------------------------------------------------------------------
// transact / batch / sweep（§4.4・§8 のブリッジ公開）
// ---------------------------------------------------------------------------

/// transact_write: put/update/delete/conditionCheck の 4 種が 1 txn で全部通る
#[test]
fn transact_write_applies_all_ops() {
    let b = bridge();
    seed(&b);
    b.transact_write(&json!([
        { "put": { "table": "orders",
                   "item": { "userId": "u1", "orderId": "o9", "status": "open", "amount": 1 } } },
        { "update": { "table": "orders",
                      "key": { "userId": "u1", "orderId": "o1" },
                      "update": "ADD amount :d", "values": { ":d": 5 } } },
        { "delete": { "table": "orders",
                      "key": { "userId": "u1", "orderId": "o2" } } },
        { "conditionCheck": { "table": "orders",
                              "key": { "userId": "u1", "orderId": "o3" },
                              "condition": "amount = :a", "values": { ":a": 99 } } }
    ]))
    .expect("transact_write");

    let get = |oid: &str| {
        b.get("orders", &json!({ "userId": "u1", "orderId": oid }))
            .expect("get")
    };
    assert_eq!(get("o9").unwrap()["amount"], json!(1)); // put された
    assert_eq!(get("o1").unwrap()["amount"], json!(35)); // 30 + 5
    assert_eq!(get("o2"), None); // delete された
}

/// 条件不成立は TransactionCanceled（理由コード配列・該当位置のみ Failed）で
/// **全操作ロールバック**
#[test]
fn transact_write_cancels_all_on_condition_failure() {
    let b = bridge();
    seed(&b);
    let err = b
        .transact_write(&json!([
            { "put": { "table": "orders",
                       "item": { "userId": "u1", "orderId": "o9", "amount": 1 } } },
            { "conditionCheck": { "table": "orders",
                                  "key": { "userId": "u1", "orderId": "o3" },
                                  "condition": "amount = :a", "values": { ":a": -1 } } }
        ]))
        .expect_err("must cancel");
    match &err {
        DbError::TransactionCanceled(reasons) => {
            assert_eq!(
                reasons,
                &vec!["None".to_string(), "ConditionalCheckFailed".into()]
            );
        }
        other => panic!("expected TransactionCanceled, got {other:?}"),
    }
    // put もロールバックされている
    let got = b
        .get("orders", &json!({ "userId": "u1", "orderId": "o9" }))
        .expect("get");
    assert_eq!(got, None);
}

/// transact_get / batch_get: 単一スナップショットで複数キー → 順序保存・欠損は null
#[test]
fn transact_get_returns_items_and_nulls_in_order() {
    let b = bridge();
    seed(&b);
    let keys = json!([
        { "table": "orders", "key": { "userId": "u1", "orderId": "o3" } },
        { "table": "orders", "key": { "userId": "u1", "orderId": "ghost" } },
        { "table": "orders", "key": { "userId": "u1", "orderId": "o1" } }
    ]);
    let got = b.transact_get(&keys).expect("transact_get");
    let items = got.as_array().expect("array");
    assert_eq!(items[0]["amount"], json!(99));
    assert!(items[1].is_null());
    assert_eq!(items[2]["amount"], json!(30));
    // ローカルでは batch_get も同一意味論
    assert_eq!(b.batch_get(&keys).expect("batch_get"), got);
}

/// batch_write: puts / deletes を無制限に流せる冪等ループ
#[test]
fn batch_write_puts_and_deletes() {
    let b = bridge();
    seed(&b);
    b.batch_write(&json!({
        "puts": [
            { "table": "orders",
              "item": { "userId": "u2", "orderId": "b1", "amount": 7 } },
            { "table": "orders",
              "item": { "userId": "u2", "orderId": "b2", "amount": 8 } }
        ],
        "deletes": [
            { "table": "orders", "key": { "userId": "u1", "orderId": "o2" } },
            { "table": "orders", "key": { "userId": "u1", "orderId": "ghost" } }
        ]
    }))
    .expect("batch_write");
    assert!(b
        .get("orders", &json!({ "userId": "u2", "orderId": "b1" }))
        .expect("get")
        .is_some());
    assert_eq!(
        b.get("orders", &json!({ "userId": "u1", "orderId": "o2" }))
            .expect("get"),
        None
    );
}

/// TTL: 失効項目は読取時点で隠れ、sweep_expired が物理削除数を返す（§8）
#[test]
fn sweep_expired_via_bridge() {
    let b = Bridge::new(InMemoryStorage::new());
    b.create_table(&json!({
        "name": "sessions", "key": { "pk": "id" }, "ttlAttr": "expiresAt"
    }))
    .expect("create_table");
    b.engine().set_now(1_000);
    b.put("sessions", &json!({ "id": "old", "expiresAt": 500 }), None)
        .expect("put");
    b.put(
        "sessions",
        &json!({ "id": "live", "expiresAt": 2_000 }),
        None,
    )
    .expect("put");

    // 読取時失効（sweep 前でも見えない）
    assert_eq!(
        b.get("sessions", &json!({ "id": "old" })).expect("get"),
        None
    );
    // 物理削除は sweep で
    assert_eq!(b.sweep_expired("sessions", 10).expect("sweep"), 1);
    assert!(b
        .get("sessions", &json!({ "id": "live" }))
        .expect("get")
        .is_some());
}

/// 不正な形の transact 操作は Validation
#[test]
fn malformed_transact_ops_are_validation_errors() {
    let b = bridge();
    seed(&b);
    for bad in [
        json!({ "not": "an array" }),
        json!([{ "teleport": { "table": "orders" } }]), // 未知の操作
        json!([{ "put": { "table": "orders" } }]),      // item がない
        json!([{ "update": { "table": "orders",
                             "key": { "userId": "u1", "orderId": "o1" } } }]), // update 式がない
    ] {
        let r = b.transact_write(&bad);
        assert!(matches!(r, Err(DbError::Validation(_))), "{bad}: got {r:?}");
    }
}

/// 不正な形の JSON は Validation
#[test]
fn malformed_requests_are_validation_errors() {
    let b = bridge();
    for bad in [
        json!({ "name": "x" }),           // key がない
        json!({ "key": { "pk": "id" } }), // name がない
        json!("just a string"),           // オブジェクトですらない
    ] {
        assert!(
            matches!(b.create_table(&bad), Err(DbError::Validation(_))),
            "must reject {bad}"
        );
    }
    // key に pk 属性が欠けている get
    seed(&b);
    let r = b.get("orders", &json!({ "orderId": "o1" }));
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
}

/// 集合型: `$ss`/`$ns`/`$bs` で書き、正規化されて round-trip・ADD/DELETE・contains
#[test]
fn sets_via_bridge() {
    let b = bridge();
    b.put(
        "orders",
        &json!({
            "userId": "u1", "orderId": "s1",
            "tags":   { "$ss": ["red", "blue", "red"] },      // 重複は除去される
            "scores": { "$ns": [2, "1.0"] },                  // 数値文字列も可・正規化
            "blobs":  { "$bs": ["01", "00ff"] }
        }),
        None,
    )
    .expect("put");
    let got = b
        .get("orders", &json!({ "userId": "u1", "orderId": "s1" }))
        .expect("get")
        .unwrap();
    assert_eq!(got["tags"], json!({ "$ss": ["blue", "red"] })); // 整列＋一意
    assert_eq!(got["scores"], json!({ "$ns": [1, 2] })); // "1.0" → 1 に正規化
    assert_eq!(got["blobs"], json!({ "$bs": ["00ff", "01"] }));

    // ADD = 集合和 / DELETE = 集合差（空になったら属性ごと削除）
    let after = b
        .update(
            "orders",
            &json!({ "userId": "u1", "orderId": "s1" }),
            &json!({
                "update": "ADD tags :t DELETE scores :s",
                "values": { ":t": { "$ss": ["green"] }, ":s": { "$ns": [1, 2] } }
            }),
        )
        .expect("update");
    assert_eq!(after["tags"], json!({ "$ss": ["blue", "green", "red"] }));
    assert!(after.get("scores").is_none());

    // filter でも contains が集合の要素判定になる
    let page = b
        .scan(
            "orders",
            &json!({ "filter": "contains(tags, :v)", "values": { ":v": "green" } }),
        )
        .expect("scan");
    assert_eq!(page["items"].as_array().unwrap().len(), 1);

    // 空集合は Validation
    let r = b.put(
        "orders",
        &json!({ "userId": "u1", "orderId": "s2", "tags": { "$ss": [] } }),
        None,
    );
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
}
