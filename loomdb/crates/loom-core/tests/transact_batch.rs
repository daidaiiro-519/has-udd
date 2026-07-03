//! @spec 01-spec.md#4.4 / #6 — transact_write（all-or-nothing）・transact_get・batch。
//!
//! test-standard §必須プロパティ「トランザクション原子性」:
//! transact_write の途中で 1 操作を失敗させると、**どの項目も索引も変更されていない**。
//! 差別化: 操作数は無制限（DynamoDB の 25/100 件制限は撤廃・spec §11）。

use loom_core::application::usecases::{
    batch_get, batch_write, create_table, get_item, put_item, query, scan, transact_get,
    transact_write, ConditionInput, KeyConditionInput, KeyRef, QueryOptions, ScanOptions,
    TransactWriteOp, UpdateInput,
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

fn doc(id: &str, status: &str) -> Item {
    let mut it = Item::new();
    it.insert("id".into(), s(id));
    it.insert("status".into(), s(status));
    it
}

fn cond(expression: &str) -> ConditionInput {
    ConditionInput {
        expression: expression.into(),
        names: BTreeMap::new(),
        values: BTreeMap::new(),
    }
}

/// docs テーブル（pk=id・GSI byStatus）
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
            indexes: vec![IndexDef {
                name: "byStatus".into(),
                key: KeySchema {
                    pk: "status".into(),
                    sk: None,
                },
                projection: Projection::KeysOnly,
            }],
            ttl_attr: None,
        },
    )
    .expect("create");
    e
}

fn put_op(id: &str, status: &str, condition: Option<ConditionInput>) -> TransactWriteOp {
    TransactWriteOp::Put {
        table: "docs".into(),
        item: doc(id, status),
        condition,
    }
}

fn dump(e: &InMemoryStorage) -> Vec<Item> {
    scan(e, "docs", &ScanOptions::default())
        .expect("scan")
        .items
}

/// 原子性: 途中の条件失敗で、項目も索引も一切変わらない（必須プロパティ）
#[test]
fn failed_transact_changes_nothing() {
    let e = engine();
    put_item(&e, "docs", &doc("seed", "open"), None).expect("seed");
    let before_items = dump(&e);

    let err = transact_write(
        &e,
        &[
            put_op("a", "open", None),
            // seed は既に存在する → この条件は失敗する
            put_op("seed", "closed", Some(cond("attribute_not_exists(id)"))),
            put_op("b", "open", None),
        ],
    )
    .expect_err("must cancel");

    // 理由コード配列（失敗した op の位置に ConditionalCheckFailed）
    match err {
        DbError::TransactionCanceled(reasons) => {
            assert_eq!(reasons, ["None", "ConditionalCheckFailed", "None"]);
        }
        other => panic!("expected TransactionCanceled, got {other:?}"),
    }

    // 項目が一切変わっていない
    assert_eq!(dump(&e), before_items);
    // 索引も変わっていない（a/b が byStatus=open に現れない）
    let page = query(
        &e,
        "docs",
        &KeyConditionInput {
            expression: "status = :s".into(),
            names: BTreeMap::new(),
            values: [(":s".to_string(), s("open"))].into(),
        },
        &QueryOptions {
            index: Some("byStatus".into()),
            ..QueryOptions::default()
        },
    )
    .expect("query");
    assert_eq!(page.items.len(), 1); // seed のみ
}

/// 成功時: Put / Update / Delete / ConditionCheck が 1 txn で全部適用される
#[test]
fn successful_transact_applies_everything() {
    let e = engine();
    put_item(&e, "docs", &doc("victim", "open"), None).expect("seed");
    put_item(&e, "docs", &doc("counter", "open"), None).expect("seed");
    put_item(&e, "docs", &doc("anchor", "open"), None).expect("seed");

    transact_write(
        &e,
        &[
            put_op("fresh", "open", Some(cond("attribute_not_exists(id)"))),
            TransactWriteOp::Update {
                table: "docs".into(),
                pk: s("counter"),
                sk: None,
                update: UpdateInput {
                    expression: "ADD hits :one".into(),
                    names: BTreeMap::new(),
                    values: [(":one".to_string(), n("1"))].into(),
                },
                condition: None,
            },
            TransactWriteOp::Delete {
                table: "docs".into(),
                pk: s("victim"),
                sk: None,
                condition: Some(cond("attribute_exists(id)")),
            },
            // 同一 txn 内の他の item と重複しない item への純粋チェック
            // （同一 item への複数操作は DynamoDB 同様に禁止）
            TransactWriteOp::ConditionCheck {
                table: "docs".into(),
                pk: s("anchor"),
                sk: None,
                condition: cond("attribute_exists(id)"),
            },
        ],
    )
    .expect("transact");

    assert!(get_item(&e, "docs", &s("fresh"), None, None)
        .expect("get")
        .is_some());
    assert!(get_item(&e, "docs", &s("victim"), None, None)
        .expect("get")
        .is_none());
    let counter = get_item(&e, "docs", &s("counter"), None, None)
        .expect("get")
        .unwrap();
    assert_eq!(counter.get("hits"), Some(&n("1")));
}

/// ConditionCheck 単体の失敗もキャンセルになる
#[test]
fn condition_check_failure_cancels() {
    let e = engine();
    let err = transact_write(
        &e,
        &[
            put_op("x", "open", None),
            TransactWriteOp::ConditionCheck {
                table: "docs".into(),
                pk: s("ghost"),
                sk: None,
                condition: cond("attribute_exists(id)"),
            },
        ],
    )
    .expect_err("must cancel");
    assert!(matches!(err, DbError::TransactionCanceled(_)));
    assert!(get_item(&e, "docs", &s("x"), None, None)
        .expect("get")
        .is_none());
}

/// 同一 item への複数操作は拒否（DynamoDB 準拠）
#[test]
fn duplicate_items_are_rejected() {
    let e = engine();
    let err = transact_write(
        &e,
        &[put_op("a", "open", None), put_op("a", "closed", None)],
    )
    .expect_err("must reject");
    assert!(matches!(err, DbError::Validation(_)));
}

/// 差別化: 操作数は無制限（DynamoDB の 100 件制限を超える 150 件が 1 txn で通る）
#[test]
fn unlimited_operations() {
    let e = engine();
    let ops: Vec<TransactWriteOp> = (0..150)
        .map(|i| put_op(&format!("item{i:03}"), "open", None))
        .collect();
    transact_write(&e, &ops).expect("150 ops in one txn");
    assert_eq!(dump(&e).len(), 150);
}

/// transact_get / batch_get: 一貫スナップショットで順序どおり返る（無ければ None）
#[test]
fn transact_get_returns_snapshot() {
    let e = engine();
    put_item(&e, "docs", &doc("a", "open"), None).expect("put");
    put_item(&e, "docs", &doc("b", "open"), None).expect("put");

    let keys = [
        KeyRef {
            table: "docs".into(),
            pk: s("b"),
            sk: None,
        },
        KeyRef {
            table: "docs".into(),
            pk: s("ghost"),
            sk: None,
        },
        KeyRef {
            table: "docs".into(),
            pk: s("a"),
            sk: None,
        },
    ];
    let got = transact_get(&e, &keys).expect("transact_get");
    assert_eq!(got.len(), 3);
    assert_eq!(got[0].as_ref().and_then(|i| i.get("id")), Some(&s("b")));
    assert!(got[1].is_none());
    assert_eq!(got[2].as_ref().and_then(|i| i.get("id")), Some(&s("a")));

    // batch_get はローカルでは同じ意味論（同一スナップショット・上位互換）
    let got2 = batch_get(&e, &keys).expect("batch_get");
    assert_eq!(got, got2);
}

/// batch_write: puts と deletes を冪等に適用（UnprocessedItems は常に空 = Ok）
#[test]
fn batch_write_applies_puts_and_deletes() {
    let e = engine();
    put_item(&e, "docs", &doc("old", "open"), None).expect("put");

    batch_write(
        &e,
        &[
            ("docs".to_string(), doc("n1", "open")),
            ("docs".to_string(), doc("n2", "open")),
        ],
        &[KeyRef {
            table: "docs".into(),
            pk: s("old"),
            sk: None,
        }],
    )
    .expect("batch_write");

    assert!(get_item(&e, "docs", &s("n1"), None, None)
        .expect("get")
        .is_some());
    assert!(get_item(&e, "docs", &s("n2"), None, None)
        .expect("get")
        .is_some());
    assert!(get_item(&e, "docs", &s("old"), None, None)
        .expect("get")
        .is_none());
}

/// transact 内の Update でもキー属性の変更は拒否される
#[test]
fn transact_update_protects_key_attributes() {
    let e = engine();
    put_item(&e, "docs", &doc("a", "open"), None).expect("put");
    let err = transact_write(
        &e,
        &[TransactWriteOp::Update {
            table: "docs".into(),
            pk: s("a"),
            sk: None,
            update: UpdateInput {
                expression: "SET id = :v".into(),
                names: BTreeMap::new(),
                values: [(":v".to_string(), s("evil"))].into(),
            },
            condition: None,
        }],
    )
    .expect_err("must reject");
    assert!(matches!(err, DbError::Validation(_)));
}
