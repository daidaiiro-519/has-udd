//! @spec 01-spec.md#4.2 — update_item（条件付き更新・upsert・原子カウンタ）。

use loom_core::application::usecases::{
    create_table, get_item, put_item, update_item, ConditionInput, UpdateInput,
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

fn upd(expression: &str, values: &[(&str, AttributeValue)]) -> UpdateInput {
    UpdateInput {
        expression: expression.into(),
        names: BTreeMap::new(),
        values: values
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect(),
    }
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

/// 存在しない item への update は upsert（キー属性入りで新規作成・DynamoDB 準拠）
#[test]
fn update_missing_item_upserts_with_key() {
    let e = engine();
    let new = update_item(
        &e,
        "docs",
        &s("a"),
        None,
        &upd("SET title = :t", &[(":t", s("hello"))]),
        None,
    )
    .expect("update");
    assert_eq!(new.get("id"), Some(&s("a"))); // キー属性が入る
    assert_eq!(new.get("title"), Some(&s("hello")));
    // 実際に格納されている
    let got = get_item(&e, "docs", &s("a"), None).expect("get").unwrap();
    assert_eq!(got, new);
}

/// 戻り値は ALL_NEW 相当（適用後の item 全体）
#[test]
fn update_returns_all_new() {
    let e = engine();
    let mut item = Item::new();
    item.insert("id".into(), s("a"));
    item.insert("count".into(), n("1"));
    put_item(&e, "docs", &item, None).expect("seed");

    let new = update_item(
        &e,
        "docs",
        &s("a"),
        None,
        &upd(
            "SET title = :t ADD count :one",
            &[(":t", s("x")), (":one", n("1"))],
        ),
        None,
    )
    .expect("update");
    assert_eq!(new.get("id"), Some(&s("a")));
    assert_eq!(new.get("title"), Some(&s("x")));
    assert_eq!(new.get("count"), Some(&n("2")));
}

/// 原子カウンタ: ADD を 2 回 → 2（ストレージを通して）
#[test]
fn atomic_counter_through_storage() {
    let e = engine();
    for _ in 0..2 {
        update_item(
            &e,
            "docs",
            &s("page"),
            None,
            &upd("ADD hits :one", &[(":one", n("1"))]),
            None,
        )
        .expect("update");
    }
    let got = get_item(&e, "docs", &s("page"), None)
        .expect("get")
        .unwrap();
    assert_eq!(got.get("hits"), Some(&n("2")));
}

/// 条件付き更新: 不成立なら ConditionalCheckFailed で何も変わらない
#[test]
fn conditional_update() {
    let e = engine();
    let mut item = Item::new();
    item.insert("id".into(), s("a"));
    item.insert("version".into(), n("1"));
    put_item(&e, "docs", &item, None).expect("seed");

    // version=1 を期待 → 成功
    update_item(
        &e,
        "docs",
        &s("a"),
        None,
        &upd("SET version = :new", &[(":new", n("2"))]),
        Some(&cond("version = :expected", &[(":expected", n("1"))])),
    )
    .expect("cas success");

    // もう一度 version=1 を期待 → 失敗・状態不変
    let err = update_item(
        &e,
        "docs",
        &s("a"),
        None,
        &upd("SET version = :new", &[(":new", n("9"))]),
        Some(&cond("version = :expected", &[(":expected", n("1"))])),
    )
    .expect_err("stale cas must fail");
    assert!(matches!(err, DbError::ConditionalCheckFailed));
    let got = get_item(&e, "docs", &s("a"), None).expect("get").unwrap();
    assert_eq!(got.get("version"), Some(&n("2")));
}

/// キー属性の変更は禁止（DynamoDB 準拠）
#[test]
fn updating_key_attributes_is_rejected() {
    let e = engine();
    for expr in ["SET id = :v", "REMOVE id", "ADD id :v"] {
        let err = update_item(
            &e,
            "docs",
            &s("a"),
            None,
            &upd(expr, &[(":v", n("1"))]),
            None,
        )
        .expect_err("must reject");
        assert!(
            matches!(err, DbError::Validation(_)),
            "expr {expr:?}: got {err:?}"
        );
    }
}

/// テーブルスキーマと sk の有無が食い違う呼び出しは Validation
#[test]
fn sort_key_mismatch_is_rejected() {
    let e = engine(); // docs は sk なし
    let err = update_item(
        &e,
        "docs",
        &s("a"),
        Some(&s("wrong")),
        &upd("SET x = :v", &[(":v", n("1"))]),
        None,
    )
    .expect_err("must reject");
    assert!(matches!(err, DbError::Validation(_)));
}
