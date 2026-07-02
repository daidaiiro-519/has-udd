//! `StorageEngine` port の契約テストスイート（test-standard §契約）。
//!
//! **アダプタ非依存**: どの実装（in-memory fake / loom-redb / 将来の LMDB）も
//! この同一スイートを通ることで、port の意味論と差替可能性を担保する。
//! 使い方: 各アダプタの tests から `run_all(|| 新しい空のエンジン)` を呼ぶ。

use loom_core::ports::StorageEngine;

/// 全契約を実行する。factory は**毎回新しい空のエンジン**を返すこと。
pub fn run_all<E, F>(new_engine: F)
where
    E: StorageEngine,
    F: Fn() -> E,
{
    get_missing_returns_none(&new_engine);
    put_commit_get(&new_engine);
    read_your_writes_in_txn(&new_engine);
    drop_without_commit_rolls_back(&new_engine);
    overwrite_last_wins(&new_engine);
    delete_removes_and_missing_delete_is_ok(&new_engine);
    scan_prefix_filters_and_sorts(&new_engine);
    write_txn_scan_sees_own_writes(&new_engine);
    tables_are_isolated(&new_engine);
    read_snapshot_is_stable(&new_engine);
}

fn write_txn_scan_sees_own_writes<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    let mut w = e.begin_write().expect("begin_write");
    w.put("t", b"b", b"2").expect("put");
    w.put("t", b"a", b"1").expect("put");
    let hits = w.scan_prefix("t", b"").expect("scan_prefix");
    let keys: Vec<&[u8]> = hits.iter().map(|(k, _)| k.as_slice()).collect();
    assert_eq!(
        keys,
        vec![b"a".as_slice(), b"b"],
        "contract[write_scan]: write txn 内の走査は未 commit の自分の書込を昇順で見る"
    );
}

fn get_missing_returns_none<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    let r = e.begin_read().expect("begin_read");
    assert_eq!(
        r.get("t", b"nope").expect("get"),
        None,
        "contract[get_missing]: 未書込キーは None"
    );
}

fn put_commit_get<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    let mut w = e.begin_write().expect("begin_write");
    w.put("t", b"k", b"v1").expect("put");
    w.commit().expect("commit");
    let r = e.begin_read().expect("begin_read");
    assert_eq!(
        r.get("t", b"k").expect("get"),
        Some(b"v1".to_vec()),
        "contract[put_commit_get]: commit 後の読取で見える"
    );
}

fn read_your_writes_in_txn<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    let mut w = e.begin_write().expect("begin_write");
    w.put("t", b"k", b"v1").expect("put");
    assert_eq!(
        w.get("t", b"k").expect("get"),
        Some(b"v1".to_vec()),
        "contract[read_your_writes]: 同一 write txn 内で自分の書込が見える"
    );
}

fn drop_without_commit_rolls_back<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    {
        let mut w = e.begin_write().expect("begin_write");
        w.put("t", b"k", b"v1").expect("put");
        // commit せず drop = ロールバック（architecture §3）
    }
    let r = e.begin_read().expect("begin_read");
    assert_eq!(
        r.get("t", b"k").expect("get"),
        None,
        "contract[rollback]: commit しなければ何も残らない"
    );
}

fn overwrite_last_wins<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    let mut w = e.begin_write().expect("begin_write");
    w.put("t", b"k", b"v1").expect("put");
    w.put("t", b"k", b"v2").expect("put");
    w.commit().expect("commit");
    let r = e.begin_read().expect("begin_read");
    assert_eq!(
        r.get("t", b"k").expect("get"),
        Some(b"v2".to_vec()),
        "contract[overwrite]: 同一キーへの put は上書き"
    );
}

fn delete_removes_and_missing_delete_is_ok<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    let mut w = e.begin_write().expect("begin_write");
    w.put("t", b"k", b"v").expect("put");
    w.commit().expect("commit");

    let mut w = e.begin_write().expect("begin_write");
    w.delete("t", b"k").expect("delete");
    w.delete("t", b"ghost")
        .expect("contract[delete_missing]: 存在しないキーの delete はエラーにしない");
    w.commit().expect("commit");

    let r = e.begin_read().expect("begin_read");
    assert_eq!(
        r.get("t", b"k").expect("get"),
        None,
        "contract[delete]: delete 後は読めない"
    );
}

fn scan_prefix_filters_and_sorts<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    let mut w = e.begin_write().expect("begin_write");
    // わざと昇順でない順序で入れる
    for (k, v) in [
        (b"b".to_vec(), b"3".to_vec()),
        (b"a".to_vec(), b"1".to_vec()),
        (b"ab".to_vec(), b"2".to_vec()),
        (b"c\x00d".to_vec(), b"4".to_vec()),
        (b"aa\xff".to_vec(), b"5".to_vec()),
    ] {
        w.put("t", &k, &v).expect("put");
    }
    w.commit().expect("commit");

    let r = e.begin_read().expect("begin_read");
    let hits = r.scan_prefix("t", b"a").expect("scan_prefix");
    let keys: Vec<&[u8]> = hits.iter().map(|(k, _)| k.as_slice()).collect();
    assert_eq!(
        keys,
        vec![b"a".as_slice(), b"aa\xff", b"ab"],
        "contract[scan_prefix]: prefix 一致のみ・キー昇順"
    );
    assert_eq!(
        hits.iter().map(|(_, v)| v.as_slice()).collect::<Vec<_>>(),
        vec![b"1".as_slice(), b"5", b"2"],
        "contract[scan_prefix]: 値もキーに対応"
    );

    let all = r.scan_prefix("t", b"").expect("scan_prefix empty");
    assert_eq!(all.len(), 5, "contract[scan_all]: 空 prefix は全件");
    let mut sorted = all.clone();
    sorted.sort();
    assert_eq!(all, sorted, "contract[scan_all]: 全件走査も昇順");
}

fn tables_are_isolated<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    let mut w = e.begin_write().expect("begin_write");
    w.put("t1", b"k", b"from-t1").expect("put");
    w.put("t2", b"k", b"from-t2").expect("put");
    // 名前が prefix 関係にあるテーブル同士も混ざらないこと
    w.put("a", b"zz", b"in-a").expect("put");
    w.put("ab", b"zz", b"in-ab").expect("put");
    w.commit().expect("commit");

    let r = e.begin_read().expect("begin_read");
    assert_eq!(
        r.get("t1", b"k").expect("get"),
        Some(b"from-t1".to_vec()),
        "contract[isolation]: t1 のキーは t1 の値"
    );
    assert_eq!(
        r.get("t2", b"k").expect("get"),
        Some(b"from-t2".to_vec()),
        "contract[isolation]: t2 のキーは t2 の値"
    );
    assert_eq!(
        r.scan_prefix("a", b"").expect("scan").len(),
        1,
        "contract[isolation]: テーブル a の走査に ab の項目が混ざらない"
    );
}

fn read_snapshot_is_stable<E: StorageEngine>(f: &impl Fn() -> E) {
    let e = f();
    let mut w = e.begin_write().expect("begin_write");
    w.put("t", b"k", b"old").expect("put");
    w.commit().expect("commit");

    let r = e.begin_read().expect("begin_read"); // ← この時点のスナップショット
    let mut w = e.begin_write().expect("begin_write");
    w.put("t", b"k", b"new").expect("put");
    w.commit().expect("commit");

    assert_eq!(
        r.get("t", b"k").expect("get"),
        Some(b"old".to_vec()),
        "contract[snapshot]: 先に開いた read txn は後の commit を見ない（MVCC）"
    );
    let r2 = e.begin_read().expect("begin_read");
    assert_eq!(
        r2.get("t", b"k").expect("get"),
        Some(b"new".to_vec()),
        "contract[snapshot]: 新しい read txn は最新を見る"
    );
}
