//! LoomDB デモ CLI。core（ユースケース）＋ redb（アダプタ）を通して put/get を実演し、
//! ヘキサゴナル構成が end-to-end で疎通することを示す。
//!
//! 実行: `cargo run -p loom-cli`

use loom_core::domain::{AttributeValue, KeySchema, Number, TableDef};
use loom_core::{
    application::usecases::{
        create_table, get_item, list_tables, put_item, query, KeyConditionInput, QueryOptions,
    },
    Item,
};
use loom_query::{InputRef, JoinEq, JoinKind, JoinQuery, JoinStep};
use loom_redb::RedbStorage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 一時 DB ファイルを用意（デモ用に毎回作り直す）。
    let dir = std::env::temp_dir().join("loomdb-demo");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("demo.redb");
    let _ = std::fs::remove_file(&path);

    let engine = RedbStorage::create(&path)?;

    // DynamoDB 同様、まずテーブルを作成する（pk=userId, sk=orderId）。
    create_table(
        &engine,
        &TableDef {
            name: "orders".into(),
            key: KeySchema {
                pk: "userId".into(),
                sk: Some("orderId".into()),
            },
            indexes: vec![],
            ttl_attr: None,
        },
    )?;
    println!("create: orders (tables = {:?})", list_tables(&engine)?);

    // 以降はテーブル名で参照する。2 件書き込む。
    for (uid, oid, amount) in [("u1", "o100", "1200"), ("u1", "o101", "3400")] {
        let mut item: Item = Item::new();
        item.insert("userId".into(), AttributeValue::S(uid.into()));
        item.insert("orderId".into(), AttributeValue::S(oid.into()));
        item.insert("amount".into(), AttributeValue::N(Number(amount.into())));
        put_item(&engine, "orders", &item, None)?;
        println!("put   : {uid}/{oid} amount={amount}");
    }

    // 主キーで 1 件取得。
    let got = get_item(
        &engine,
        "orders",
        &AttributeValue::S("u1".into()),
        Some(&AttributeValue::S("o100".into())),
    )?;
    println!("get   : u1/o100 -> {got:?}");

    // Query: u1 の注文を sk（orderId）昇順で。
    let page = query(
        &engine,
        "orders",
        &KeyConditionInput {
            expression: "userId = :u".into(),
            names: Default::default(),
            values: [(":u".to_string(), AttributeValue::S("u1".into()))].into(),
        },
        &QueryOptions::default(),
    )?;
    println!(
        "query : userId = u1 -> {} 件 (orderId 昇順)",
        page.items.len()
    );

    // JOIN はデータ構造まで（実行器は骨子）。ここでは構造の組み立てだけ示す。
    let join = JoinQuery {
        root: InputRef {
            table: "orders".into(),
            alias: "o".into(),
            index: None,
        },
        steps: vec![JoinStep {
            input: InputRef {
                table: "users".into(),
                alias: "u".into(),
                index: Some("byId".into()),
            },
            kind: JoinKind::Inner,
            on: vec![JoinEq {
                left: "o.userId".into(),
                right: "u.id".into(),
            }],
        }],
        select: vec!["o.orderId".into(), "u.name".into()],
    };
    println!(
        "join  : {} 段のプラン（実行器は未実装・骨子）",
        join.steps.len()
    );

    Ok(())
}
