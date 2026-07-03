//! @spec 01-spec.md#13 — 運用 API（format_version / stats / compact）。
//!
//! - `ensure_format`: 初回 open で format_version を meta に記録・未知の版は明示エラー
//! - `stats(table)`: item_count は **O(1)**（書込パスで維持されるカウンタ）・
//!   storage_bytes は物理サイズ
//! - `compact()`: 対応しないエンジン（in-memory fake）は false

use loom_core::application::meta::META_TABLE;
use loom_core::application::usecases::{
    batch_write, create_table, delete_item, delete_table, ensure_format, put_item, stats,
    sweep_expired, transact_write, update_item, ExprInput, KeyRef, TransactWriteOp, FORMAT_VERSION,
};
use loom_core::domain::{AttributeValue, DbError, Item, KeySchema, Number, TableDef};
use loom_core::ports::StorageEngine;
use loom_testkit::InMemoryStorage;
use std::collections::BTreeMap;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.into())
}
fn n(v: i64) -> AttributeValue {
    AttributeValue::N(Number(v.to_string()))
}

fn doc(id: &str) -> Item {
    let mut it = Item::new();
    it.insert("id".into(), s(id));
    it.insert("v".into(), n(1));
    it
}

fn docs_table(ttl: Option<&str>) -> TableDef {
    TableDef {
        name: "docs".into(),
        key: KeySchema {
            pk: "id".into(),
            sk: None,
        },
        indexes: vec![],
        ttl_attr: ttl.map(|a| a.to_string()),
    }
}

/// 初回 open で現行版を記録し、以後は同じ版を返す。未知の版は明示エラー（§13）
#[test]
fn ensure_format_records_and_rejects_unknown_versions() {
    let e = InMemoryStorage::new();
    assert_eq!(ensure_format(&e).expect("first"), FORMAT_VERSION);
    assert_eq!(ensure_format(&e).expect("second"), FORMAT_VERSION); // 冪等

    // 将来の版のファイルを開いたら明示エラー（黙って壊さない）
    let mut txn = e.begin_write().expect("txn");
    txn.put(META_TABLE, b"format_version", &99u64.to_be_bytes())
        .expect("tamper");
    txn.commit().expect("commit");
    let r = ensure_format(&e);
    assert!(matches!(r, Err(DbError::Validation(_))), "got {r:?}");
}

/// item_count は put（新規/上書き）・update（upsert）・delete で正しく増減する
#[test]
fn stats_item_count_tracks_writes() {
    let e = InMemoryStorage::new();
    create_table(&e, &docs_table(None)).expect("create");
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 0);

    put_item(&e, "docs", &doc("a"), None).expect("put a");
    put_item(&e, "docs", &doc("b"), None).expect("put b");
    put_item(&e, "docs", &doc("a"), None).expect("overwrite a"); // 上書きは増えない
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 2);

    // update の upsert は +1・既存への update は不変
    let upd = ExprInput {
        expression: "SET v = :v".into(),
        names: BTreeMap::new(),
        values: [(":v".to_string(), n(9))].into_iter().collect(),
    };
    update_item(&e, "docs", &s("c"), None, &upd, None).expect("upsert c");
    update_item(&e, "docs", &s("a"), None, &upd, None).expect("update a");
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 3);

    // delete は -1・存在しないキーの delete は不変
    delete_item(&e, "docs", &s("b"), None, None).expect("delete b");
    delete_item(&e, "docs", &s("ghost"), None, None).expect("delete ghost");
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 2);

    // storage_bytes は書込後は非ゼロ
    assert!(stats(&e, "docs").expect("stats").storage_bytes > 0);

    // 未知テーブルは ResourceNotFound
    assert!(matches!(
        stats(&e, "ghost"),
        Err(DbError::ResourceNotFound(_))
    ));
}

/// transact / batch / sweep でもカウンタが破綻しない（ロールバック時は不変）
#[test]
fn stats_item_count_survives_transact_batch_sweep() {
    let e = InMemoryStorage::new();
    create_table(&e, &docs_table(Some("ttl"))).expect("create");
    put_item(&e, "docs", &doc("a"), None).expect("put");

    transact_write(
        &e,
        &[
            TransactWriteOp::Put {
                table: "docs".into(),
                item: doc("t1"),
                condition: None,
            },
            TransactWriteOp::Update {
                table: "docs".into(),
                pk: s("t2"),
                sk: None,
                update: ExprInput {
                    expression: "SET v = :v".into(),
                    names: BTreeMap::new(),
                    values: [(":v".to_string(), n(1))].into_iter().collect(),
                },
                condition: None,
            },
            TransactWriteOp::Delete {
                table: "docs".into(),
                pk: s("a"),
                sk: None,
                condition: None,
            },
        ],
    )
    .expect("transact");
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 2); // +2 -1

    // 失敗した transact は全ロールバック → カウンタも不変
    let r = transact_write(
        &e,
        &[
            TransactWriteOp::Put {
                table: "docs".into(),
                item: doc("t3"),
                condition: None,
            },
            TransactWriteOp::ConditionCheck {
                table: "docs".into(),
                pk: s("t1"),
                sk: None,
                condition: ExprInput {
                    expression: "v = :bad".into(),
                    names: BTreeMap::new(),
                    values: [(":bad".to_string(), n(-1))].into_iter().collect(),
                },
            },
        ],
    );
    assert!(r.is_err());
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 2);

    // batch_write（put 2 新規 + delete 1）
    batch_write(
        &e,
        &[
            ("docs".to_string(), doc("b1")),
            ("docs".to_string(), doc("b2")),
        ],
        &[KeyRef {
            table: "docs".into(),
            pk: s("t1"),
            sk: None,
        }],
    )
    .expect("batch");
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 3);

    // TTL sweep で物理削除された分も減る
    e.set_now(1_000);
    let mut expired = doc("x");
    expired.insert("ttl".into(), n(500));
    put_item(&e, "docs", &expired, None).expect("put expired");
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 4);
    assert_eq!(sweep_expired(&e, "docs", 10).expect("sweep"), 1);
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 3);

    // delete_table でカウンタも消える → 再作成後は 0
    delete_table(&e, "docs").expect("drop");
    create_table(&e, &docs_table(None)).expect("recreate");
    assert_eq!(stats(&e, "docs").expect("stats").item_count, 0);
}

/// compact は対応しないエンジン（in-memory fake）では false
#[test]
fn compact_is_false_on_unsupported_engines() {
    let mut e = InMemoryStorage::new();
    assert!(!e.compact().expect("compact"));
}
