//! @spec 01-spec.md#13 — オンディスク形式のバージョニング。
//!
//! 初回 open で現行版を meta に記録し、以後は一致を検証する。未知（将来）の版の
//! ファイルは**黙って壊さず**明示エラー。互換変更の自動読替え・`migrate` は
//! 非互換版が生まれたときに追加する。

use crate::application::meta;
use crate::domain::DbError;
use crate::ports::StorageEngine;

/// 現行のオンディスク形式バージョン。非互換変更で上げる（crate のメジャー版と連動）。
pub const FORMAT_VERSION: u64 = 1;

const FORMAT_KEY: &[u8] = b"format_version";

/// format_version を検証し（未記録なら記録し）、そのバージョンを返す。
/// DB open 直後（Bridge 構築時など）に呼ぶ。
pub fn ensure_format<E: StorageEngine>(engine: &E) -> Result<u64, DbError> {
    let mut txn = engine.begin_write()?;
    match txn.get(meta::META_TABLE, FORMAT_KEY)? {
        Some(bytes) => {
            let version =
                u64::from_be_bytes(bytes.as_slice().try_into().map_err(|_| {
                    DbError::Serialization("corrupt format_version in meta".into())
                })?);
            if version != FORMAT_VERSION {
                return Err(DbError::Validation(format!(
                    "unsupported on-disk format version {version} \
                     (this build supports {FORMAT_VERSION})"
                )));
            }
            Ok(version) // 変更なし（txn drop）
        }
        None => {
            txn.put(meta::META_TABLE, FORMAT_KEY, &FORMAT_VERSION.to_be_bytes())?;
            txn.commit()?;
            Ok(FORMAT_VERSION)
        }
    }
}
