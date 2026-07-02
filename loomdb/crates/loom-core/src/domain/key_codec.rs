//! 順序保存エンコード（spec §2.3）。
//!
//! 論理キーは pk (+ sk) を 0x00 区切りで結合する。**物理レイアウト**（テーブル名の
//! 前置など）はストレージアダプタの責務なので、ここにはテーブル名を含めない。
//!
//! S / B は素直にバイト列化。N は本サンプルでは簡易実装（下記 TODO）。

use super::attribute::{AttributeValue, Number};
use super::error::DbError;

const SEP: u8 = 0x00;

/// pk (+ sk) を 1 本の順序保存キーに結合する。
pub fn encode_key(pk: &AttributeValue, sk: Option<&AttributeValue>) -> Result<Vec<u8>, DbError> {
    let mut out = encode_value(pk)?;
    out.push(SEP);
    if let Some(sk) = sk {
        out.extend_from_slice(&encode_value(sk)?);
    }
    Ok(out)
}

/// 単一の値を順序保存バイト列にする。キーに使えるのは S / N / B のみ（spec §2.3）。
pub fn encode_value(v: &AttributeValue) -> Result<Vec<u8>, DbError> {
    match v {
        AttributeValue::S(s) => Ok(escape(s.as_bytes())),
        AttributeValue::B(b) => Ok(escape(b)),
        AttributeValue::N(n) => encode_number(n),
        other => Err(DbError::Validation(format!(
            "key attribute must be S/N/B, got {other:?}"
        ))),
    }
}

/// 区切りの 0x00 と衝突しないよう、データ中の 0x00 を 0x00 0xFF にエスケープする。
fn escape(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for &b in bytes {
        out.push(b);
        if b == SEP {
            out.push(0xFF);
        }
    }
    out
}

/// TODO(spec §2.3): 符号・指数・仮数を辞書式=数値順にする完全実装へ置き換える。
/// 本サンプルは「非負整数のみ・固定幅ゼロ埋め」の簡易版で、順序保存を部分的にしか満たさない。
fn encode_number(n: &Number) -> Result<Vec<u8>, DbError> {
    let s = n.0.trim();
    match s.parse::<u64>() {
        // 20 桁ゼロ埋めなら u64 の辞書式順序が数値順序に一致する。
        Ok(u) => Ok(format!("{u:020}").into_bytes()),
        Err(_) => Err(DbError::Validation(format!(
            "sample encoder supports only non-negative integers for N, got {s:?}"
        ))),
    }
}
