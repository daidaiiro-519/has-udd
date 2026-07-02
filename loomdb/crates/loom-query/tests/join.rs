//! @spec 01-spec.md#10 — JOIN 実行器（N テーブル多段 index-nested-loop・left-deep）。
//!
//! test-standard §必須プロパティ「JOIN の正しさ」:
//! - 参照実装（素朴な多重ループ）と結果一致
//! - INNER はマッチ 0 件のタプルを出さない / LEFT は左タプルを残し当該入力属性が欠落
//! - 索引あり経路と scan フォールバック経路で同一結果（フォールバックは warnings で通知）
//! - N テーブル多段・エイリアス（自己結合含む）

use loom_core::application::usecases::{create_table, put_item, ConditionInput};
use loom_core::domain::{AttributeValue, DbError, IndexDef, Item, KeySchema, Projection, TableDef};
use loom_query::{execute, InputRef, JoinEq, JoinKind, JoinQuery, JoinRow, JoinStep};
use loom_testkit::InMemoryStorage;
use proptest::prelude::*;
use std::collections::BTreeMap;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.into())
}
fn n(v: i64) -> AttributeValue {
    AttributeValue::N(loom_core::domain::Number(v.to_string()))
}

fn item(pairs: &[(&str, AttributeValue)]) -> Item {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

fn table(name: &str, pk: &str, indexes: Vec<IndexDef>) -> TableDef {
    TableDef {
        name: name.into(),
        key: KeySchema {
            pk: pk.into(),
            sk: None,
        },
        indexes,
        ttl_attr: None,
    }
}

fn by_user_index() -> IndexDef {
    IndexDef {
        name: "byUser".into(),
        key: KeySchema {
            pk: "userId".into(),
            sk: None,
        },
        projection: Projection::KeysOnly,
    }
}

fn input(table: &str, alias: &str) -> InputRef {
    InputRef {
        table: table.into(),
        alias: alias.into(),
        index: None,
    }
}

fn step(table: &str, alias: &str, kind: JoinKind, on: &[(&str, &str)]) -> JoinStep {
    JoinStep {
        input: input(table, alias),
        kind,
        on: on
            .iter()
            .map(|(l, r)| JoinEq {
                left: l.to_string(),
                right: r.to_string(),
            })
            .collect(),
    }
}

fn jq(root: InputRef, steps: Vec<JoinStep>) -> JoinQuery {
    JoinQuery {
        root,
        steps,
        filter: None,
        select: vec![],
    }
}

fn get_s(row: &JoinRow, key: &str) -> Option<String> {
    match row.get(key) {
        Some(AttributeValue::S(v)) => Some(v.clone()),
        _ => None,
    }
}

/// users(id) / orders(id, userId, amount)。o4 は存在しない u9 を指す（dangling）。
fn seeded(with_orders_index: bool) -> InMemoryStorage {
    let e = InMemoryStorage::new();
    create_table(&e, &table("users", "id", vec![])).expect("create users");
    let idx = if with_orders_index {
        vec![by_user_index()]
    } else {
        vec![]
    };
    create_table(&e, &table("orders", "id", idx)).expect("create orders");
    for (id, name) in [("u1", "Alice"), ("u2", "Bob"), ("u3", "Carol")] {
        put_item(
            &e,
            "users",
            &item(&[("id", s(id)), ("name", s(name))]),
            None,
        )
        .expect("put");
    }
    for (id, uid, amount) in [
        ("o1", "u1", 30),
        ("o2", "u1", 10),
        ("o3", "u2", 99),
        ("o4", "u9", 7), // dangling
    ] {
        put_item(
            &e,
            "orders",
            &item(&[("id", s(id)), ("userId", s(uid)), ("amount", n(amount))]),
            None,
        )
        .expect("put");
    }
    e
}

fn pairs(rows: &[JoinRow]) -> Vec<(Option<String>, Option<String>)> {
    let mut out: Vec<_> = rows
        .iter()
        .map(|r| (get_s(r, "o.id"), get_s(r, "u.name")))
        .collect();
    out.sort();
    out
}

/// INNER: 右にマッチしない o4 は出ない。u.id は users の pk → pk 直引き経路
#[test]
fn inner_join_via_pk_probe() {
    let e = seeded(false);
    let q = jq(
        input("orders", "o"),
        vec![step("users", "u", JoinKind::Inner, &[("o.userId", "u.id")])],
    );
    let page = execute(&e, &q).expect("join");
    assert_eq!(
        pairs(&page.rows),
        [
            (Some("o1".into()), Some("Alice".into())),
            (Some("o2".into()), Some("Alice".into())),
            (Some("o3".into()), Some("Bob".into())),
        ]
    );
    assert!(page.warnings.is_empty(), "pk probe must not warn");
}

/// LEFT: o4 も残り、u.* が欠落する
#[test]
fn left_join_keeps_unmatched() {
    let e = seeded(false);
    let q = jq(
        input("orders", "o"),
        vec![step("users", "u", JoinKind::Left, &[("o.userId", "u.id")])],
    );
    let page = execute(&e, &q).expect("join");
    assert_eq!(page.rows.len(), 4);
    let o4 = page
        .rows
        .iter()
        .find(|r| get_s(r, "o.id").as_deref() == Some("o4"))
        .expect("o4 must be kept");
    assert!(get_s(o4, "u.name").is_none(), "u.* must be absent");
}

/// 1対多の展開: users → orders（orders.userId に GSI）= 索引経路・警告なし
#[test]
fn one_to_many_via_index_probe() {
    let e = seeded(true);
    let q = jq(
        input("users", "u"),
        vec![step(
            "orders",
            "o",
            JoinKind::Inner,
            &[("u.id", "o.userId")],
        )],
    );
    let page = execute(&e, &q).expect("join");
    assert_eq!(
        pairs(&page.rows),
        [
            (Some("o1".into()), Some("Alice".into())),
            (Some("o2".into()), Some("Alice".into())),
            (Some("o3".into()), Some("Bob".into())),
        ]
    );
    assert!(page.warnings.is_empty(), "index probe must not warn");
}

/// 索引なしでも同一結果（scan フォールバック）＋ warnings で通知（spec §10.3）
#[test]
fn scan_fallback_same_result_with_warning() {
    let indexed = execute(
        &seeded(true),
        &jq(
            input("users", "u"),
            vec![step(
                "orders",
                "o",
                JoinKind::Inner,
                &[("u.id", "o.userId")],
            )],
        ),
    )
    .expect("join");
    let fallback = execute(
        &seeded(false),
        &jq(
            input("users", "u"),
            vec![step(
                "orders",
                "o",
                JoinKind::Inner,
                &[("u.id", "o.userId")],
            )],
        ),
    )
    .expect("join");
    assert_eq!(pairs(&indexed.rows), pairs(&fallback.rows));
    assert!(!fallback.warnings.is_empty(), "fallback must warn");
}

/// 3 テーブル多段（left-deep）: orders → users(INNER) → profiles(LEFT)
#[test]
fn three_table_left_deep() {
    let e = seeded(false);
    create_table(&e, &table("profiles", "userId", vec![])).expect("create profiles");
    put_item(
        &e,
        "profiles",
        &item(&[("userId", s("u1")), ("plan", s("pro"))]),
        None,
    )
    .expect("put");

    let q = jq(
        input("orders", "o"),
        vec![
            step("users", "u", JoinKind::Inner, &[("o.userId", "u.id")]),
            step("profiles", "p", JoinKind::Left, &[("u.id", "p.userId")]),
        ],
    );
    let page = execute(&e, &q).expect("join");
    assert_eq!(page.rows.len(), 3); // o4 は step1 の INNER で落ちる
    for row in &page.rows {
        match get_s(row, "o.id").as_deref() {
            Some("o1") | Some("o2") => assert_eq!(get_s(row, "p.plan").as_deref(), Some("pro")),
            Some("o3") => assert!(get_s(row, "p.plan").is_none(), "u2 has no profile"),
            other => panic!("unexpected row {other:?}"),
        }
    }
}

/// 自己結合（同一テーブル×エイリアス2つ）: 部下 → 上司
#[test]
fn self_join_with_aliases() {
    let e = InMemoryStorage::new();
    create_table(&e, &table("employees", "id", vec![])).expect("create");
    for (id, mgr) in [("e1", None), ("e2", Some("e1")), ("e3", Some("e1"))] {
        let mut it = item(&[("id", s(id))]);
        if let Some(m) = mgr {
            it.insert("managerId".into(), s(m));
        }
        put_item(&e, "employees", &it, None).expect("put");
    }
    let q = jq(
        input("employees", "e"),
        vec![step(
            "employees",
            "m",
            JoinKind::Inner,
            &[("e.managerId", "m.id")],
        )],
    );
    let page = execute(&e, &q).expect("join");
    let mut got: Vec<_> = page
        .rows
        .iter()
        .map(|r| (get_s(r, "e.id").unwrap(), get_s(r, "m.id").unwrap()))
        .collect();
    got.sort();
    assert_eq!(
        got,
        [("e2".into(), "e1".to_string()), ("e3".into(), "e1".into())]
    ); // e1 は managerId 欠落 → INNER で落ちる
}

/// 複合キー結合（on が AND で 2 本）
#[test]
fn composite_on_conditions() {
    let e = InMemoryStorage::new();
    create_table(&e, &table("bookings", "id", vec![])).expect("create");
    create_table(&e, &table("slots", "id", vec![])).expect("create");
    put_item(
        &e,
        "bookings",
        &item(&[("id", s("b1")), ("day", s("mon")), ("hour", n(9))]),
        None,
    )
    .expect("put");
    for (id, day, hour) in [("s1", "mon", 9), ("s2", "mon", 10), ("s3", "tue", 9)] {
        put_item(
            &e,
            "slots",
            &item(&[("id", s(id)), ("day", s(day)), ("hour", n(hour))]),
            None,
        )
        .expect("put");
    }
    let q = jq(
        input("bookings", "b"),
        vec![step(
            "slots",
            "sl",
            JoinKind::Inner,
            &[("b.day", "sl.day"), ("b.hour", "sl.hour")],
        )],
    );
    let page = execute(&e, &q).expect("join");
    assert_eq!(page.rows.len(), 1);
    assert_eq!(get_s(&page.rows[0], "sl.id").as_deref(), Some("s1"));
}

/// 結合後フィルタは alias 修飾パス（§10.2）で書ける
#[test]
fn post_join_filter_with_alias_paths() {
    let e = seeded(false);
    let mut q = jq(
        input("orders", "o"),
        vec![step("users", "u", JoinKind::Inner, &[("o.userId", "u.id")])],
    );
    q.filter = Some(ConditionInput {
        expression: "u.name = :n AND o.amount >= :min".into(),
        names: BTreeMap::new(),
        values: [(":n".to_string(), s("Alice")), (":min".to_string(), n(20))].into(),
    });
    let page = execute(&e, &q).expect("join");
    assert_eq!(page.rows.len(), 1);
    assert_eq!(get_s(&page.rows[0], "o.id").as_deref(), Some("o1")); // 30 のみ
}

/// select 射影: 指定した alias.attr だけが返る
#[test]
fn select_projects_columns() {
    let e = seeded(false);
    let mut q = jq(
        input("orders", "o"),
        vec![step("users", "u", JoinKind::Inner, &[("o.userId", "u.id")])],
    );
    q.select = vec!["o.id".into(), "u.name".into()];
    let page = execute(&e, &q).expect("join");
    for row in &page.rows {
        assert_eq!(row.len(), 2, "only selected columns: {row:?}");
        assert!(row.contains_key("o.id") && row.contains_key("u.name"));
    }
}

#[test]
fn invalid_join_queries_are_rejected() {
    let e = seeded(false);
    // エイリアス重複
    let q = jq(
        input("orders", "o"),
        vec![step("users", "o", JoinKind::Inner, &[("o.userId", "o.id")])],
    );
    assert!(matches!(execute(&e, &q), Err(DbError::Validation(_))));
    // on.left が未知のエイリアス
    let q = jq(
        input("orders", "o"),
        vec![step("users", "u", JoinKind::Inner, &[("x.userId", "u.id")])],
    );
    assert!(matches!(execute(&e, &q), Err(DbError::Validation(_))));
    // on.right のエイリアスが step と不一致
    let q = jq(
        input("orders", "o"),
        vec![step("users", "u", JoinKind::Inner, &[("o.userId", "z.id")])],
    );
    assert!(matches!(execute(&e, &q), Err(DbError::Validation(_))));
    // 未知テーブル
    let q = jq(
        input("ghosts", "g"),
        vec![step("users", "u", JoinKind::Inner, &[("g.x", "u.id")])],
    );
    assert!(matches!(execute(&e, &q), Err(DbError::ResourceNotFound(_))));
    // on なし
    let q = jq(
        input("orders", "o"),
        vec![JoinStep {
            input: input("users", "u"),
            kind: JoinKind::Inner,
            on: vec![],
        }],
    );
    assert!(matches!(execute(&e, &q), Err(DbError::Validation(_))));
}

// ---------------------------------------------------------------------------
// 参照実装との一致（property）: orders JOIN users を素朴な二重ループと比較
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    #[test]
    fn join_matches_naive_reference(
        user_count in 0usize..5,
        order_uids in proptest::collection::vec(0u8..8, 0..12),
        left in any::<bool>(),
    ) {
        let e = InMemoryStorage::new();
        create_table(&e, &table("users", "id", vec![])).expect("create");
        create_table(&e, &table("orders", "id", vec![])).expect("create");
        let users: Vec<String> = (0..user_count).map(|i| format!("u{i}")).collect();
        for uid in &users {
            put_item(&e, "users", &item(&[("id", s(uid))]), None).expect("put");
        }
        let orders: Vec<(String, String)> = order_uids
            .iter()
            .enumerate()
            .map(|(i, uid)| (format!("o{i}"), format!("u{uid}")))
            .collect();
        for (oid, uid) in &orders {
            put_item(&e, "orders", &item(&[("id", s(oid)), ("userId", s(uid))]), None)
                .expect("put");
        }

        let kind = if left { JoinKind::Left } else { JoinKind::Inner };
        let q = jq(
            input("orders", "o"),
            vec![step("users", "u", kind, &[("o.userId", "u.id")])],
        );
        let page = execute(&e, &q).expect("join");
        let mut got: Vec<(Option<String>, Option<String>)> = page
            .rows
            .iter()
            .map(|r| (get_s(r, "o.id"), get_s(r, "u.id")))
            .collect();
        got.sort();

        // 参照実装: 素朴な二重ループ
        let mut expected: Vec<(Option<String>, Option<String>)> = Vec::new();
        for (oid, uid) in &orders {
            let matches: Vec<&String> = users.iter().filter(|u| *u == uid).collect();
            if matches.is_empty() {
                if left {
                    expected.push((Some(oid.clone()), None));
                }
            } else {
                for m in matches {
                    expected.push((Some(oid.clone()), Some(m.clone())));
                }
            }
        }
        expected.sort();
        prop_assert_eq!(got, expected);
    }
}
