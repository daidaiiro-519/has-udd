//! 二次索引の意味論（spec §3/§7）。ステートレス純関数。
//!
//! 索引エントリのキー = `enc(ipk) ++ enc(isk?) ++ <主キーのエンコード済みバイト列>`。
//! 各値は自己区切り（key_codec）なので連結でタプル順序が保たれ、末尾の主キーが
//! 「同じ索引キーを持つ複数 item」の一意性を担保する。
//!
//! 値は v1 では空（KEYS_ONLY 相当）で、読み取りは main から全属性を引く
//! （ローカルは同一ファイル・同一 txn で安価かつ常に強整合。射影の格納は
//! サイズ最適化として後続 — spec §7）。

use super::attribute::Item;
use super::error::DbError;
use super::key_codec;
use super::table::IndexDef;

/// 索引の物理配置に使う論理テーブル名（spec §3: `idx:{table}:{index}`）。
/// `:` は利用者のテーブル名に使えないため衝突しない。
pub fn index_table_name(table: &str, index: &str) -> String {
    format!("idx:{table}:{index}")
}

/// item から索引エントリのキーを導出する。
/// 索引キー属性が欠けていれば `None`（= sparse index・spec §7）。
pub fn index_entry_key(
    idx: &IndexDef,
    item: &Item,
    main_key: &[u8],
) -> Result<Option<Vec<u8>>, DbError> {
    let Some(ipk) = item.get(&idx.key.pk) else {
        return Ok(None);
    };
    let mut key = key_codec::encode_value(ipk)?;
    if let Some(sk_attr) = &idx.key.sk {
        let Some(isk) = item.get(sk_attr) else {
            return Ok(None);
        };
        key.extend_from_slice(&key_codec::encode_value(isk)?);
    }
    key.extend_from_slice(main_key);
    Ok(Some(key))
}
