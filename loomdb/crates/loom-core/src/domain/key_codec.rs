//! 順序保存エンコード（spec §2.3）。
//!
//! 各値は「型タグ + 本体 + 終端」の自己区切り形式。複数値（pk, sk）を連結しても
//! **タプル順序と encode 後の辞書式順序が一致**する。物理レイアウト（テーブル名の
//! 前置など）はストレージアダプタの責務なので、ここにはテーブル名を含めない。
//!
//! - **S/B**: タグ + ペイロード（0x00 → 0x00 0xFF にエスケープ）+ 終端 0x00。
//!   終端 0x00 の直後に来得るバイトは「次の値の型タグ（< 0xFF）」か入力終端のみ
//!   なので、エスケープ済み 0x00（= 00 FF）との比較で順序が壊れない
//!   （FoundationDB tuple layer と同方式）。
//! - **N**: タグ + 符号クラス（負 < ゼロ < 正）+ 指数 + 仮数。値 = 0.d1d2… × 10^E に
//!   正規化（`domain::number`）し、負数は指数・仮数・終端をビット反転相当で反転して
//!   順序を逆転させる。38 有効桁・E ∈ [-129, 126]（spec §2.2/§11）。

use super::attribute::{AttributeValue, Number};
use super::error::DbError;
use super::number::{format_decimal, parse_decimal};

// 型タグ。すべて 0xFF 未満であることが順序保存の前提（モジュール冒頭コメント参照）。
const TAG_N: u8 = 0x08;
const TAG_S: u8 = 0x10;
const TAG_B: u8 = 0x18;

const TERM: u8 = 0x00; // S/B/正の N の終端
const NEG_TERM: u8 = 0xFF; // 負の N の終端（反転世界では 0x00 相当）

// 符号クラス: 負 < ゼロ < 正
const CLASS_NEG: u8 = 0x01;
const CLASS_ZERO: u8 = 0x02;
const CLASS_POS: u8 = 0x03;

// 正規化指数（number::parse_decimal が [-129, 126] を保証）を 1 バイトへ
const EXP_BIAS: i64 = 129;

/// pk (+ sk) を 1 本の順序保存キーに結合する。
pub fn encode_key(pk: &AttributeValue, sk: Option<&AttributeValue>) -> Result<Vec<u8>, DbError> {
    let mut out = encode_value(pk)?;
    if let Some(sk) = sk {
        out.extend_from_slice(&encode_value(sk)?);
    }
    Ok(out)
}

/// 単一の値を順序保存バイト列にする。キーに使えるのは S / N / B のみ（spec §2.3）。
pub fn encode_value(v: &AttributeValue) -> Result<Vec<u8>, DbError> {
    match v {
        AttributeValue::S(s) => Ok(encode_bytes(TAG_S, s.as_bytes())),
        AttributeValue::B(b) => Ok(encode_bytes(TAG_B, b)),
        AttributeValue::N(n) => encode_number(n),
        other => Err(DbError::Validation(format!(
            "key attribute must be S/N/B, got {other:?}"
        ))),
    }
}

/// `encode_key` の逆変換。(pk, sk?) を返す。
pub fn decode_key(bytes: &[u8]) -> Result<(AttributeValue, Option<AttributeValue>), DbError> {
    let (pk, used) = decode_first(bytes)?;
    if used == bytes.len() {
        return Ok((pk, None));
    }
    let (sk, used2) = decode_first(&bytes[used..])?;
    if used + used2 != bytes.len() {
        return Err(DbError::Validation("trailing bytes after sk".into()));
    }
    Ok((pk, Some(sk)))
}

/// 先頭の 1 値をデコードし (値, 消費バイト数) を返す。
pub fn decode_first(bytes: &[u8]) -> Result<(AttributeValue, usize), DbError> {
    match bytes.first() {
        Some(&TAG_S) => {
            let (payload, used) = decode_bytes(&bytes[1..])?;
            let s = String::from_utf8(payload)
                .map_err(|e| DbError::Validation(format!("invalid UTF-8 in S key: {e}")))?;
            Ok((AttributeValue::S(s), 1 + used))
        }
        Some(&TAG_B) => {
            let (payload, used) = decode_bytes(&bytes[1..])?;
            Ok((AttributeValue::B(payload), 1 + used))
        }
        Some(&TAG_N) => decode_number(&bytes[1..]).map(|(v, used)| (v, 1 + used)),
        _ => Err(DbError::Validation(
            "unknown or missing key type tag".into(),
        )),
    }
}

// ---------------------------------------------------------------------------
// S / B
// ---------------------------------------------------------------------------

fn encode_bytes(tag: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 2);
    out.push(tag);
    for &b in payload {
        out.push(b);
        if b == 0x00 {
            out.push(0xFF); // 0x00 → 0x00 0xFF（終端 0x00 との衝突回避）
        }
    }
    out.push(TERM);
    out
}

fn decode_bytes(bytes: &[u8]) -> Result<(Vec<u8>, usize), DbError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            0x00 => {
                if bytes.get(i + 1) == Some(&0xFF) {
                    out.push(0x00); // エスケープ解除
                    i += 2;
                } else {
                    return Ok((out, i + 1)); // 終端
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    Err(DbError::Validation("unterminated S/B key payload".into()))
}

// ---------------------------------------------------------------------------
// N（10進・順序保存）— 正規化・整形は domain::number に委譲
// ---------------------------------------------------------------------------

fn encode_number(n: &Number) -> Result<Vec<u8>, DbError> {
    let d = parse_decimal(&n.0)?;
    let mut out = vec![TAG_N];
    if d.digits.is_empty() {
        out.push(CLASS_ZERO);
        out.push(TERM);
        return Ok(out);
    }
    let exp_byte = (d.exp + EXP_BIAS) as u8; // parse_decimal が範囲を保証
    if !d.neg {
        out.push(CLASS_POS);
        out.push(exp_byte);
        // 仮数: 桁 d → d+1（1..=10）。終端 0x00 より必ず大きい ⇒
        // 「短い仮数（例 0.12）< 長い仮数（例 0.123）」が辞書式でも成り立つ。
        for &dg in &d.digits {
            out.push(dg + 1);
        }
        out.push(TERM);
    } else {
        // 負数は同絶対値の並びを丸ごと反転（バイト値 b → 0xFF-b 相当）して順序を逆転。
        // 終端は 0xFF ⇒ どの反転桁（≤ 0xFE）より大きく、
        // 「長い仮数（-0.123）< 短い仮数（-0.12）」が辞書式でも成り立つ。
        out.push(CLASS_NEG);
        out.push(0xFF - exp_byte);
        for &dg in &d.digits {
            out.push(0xFF - (dg + 1));
        }
        out.push(NEG_TERM);
    }
    Ok(out)
}

fn decode_number(bytes: &[u8]) -> Result<(AttributeValue, usize), DbError> {
    let err = || DbError::Validation("malformed N key encoding".into());
    let class = *bytes.first().ok_or_else(err)?;
    match class {
        CLASS_ZERO => {
            if bytes.get(1) != Some(&TERM) {
                return Err(err());
            }
            Ok((AttributeValue::N(Number("0".into())), 2))
        }
        CLASS_POS | CLASS_NEG => {
            let neg = class == CLASS_NEG;
            let raw_exp = *bytes.get(1).ok_or_else(err)?;
            let exp = i64::from(if neg { 0xFF - raw_exp } else { raw_exp }) - EXP_BIAS;
            let term = if neg { NEG_TERM } else { TERM };
            let mut digits = Vec::new();
            let mut i = 2;
            loop {
                let b = *bytes.get(i).ok_or_else(err)?;
                i += 1;
                if b == term {
                    break;
                }
                let dg = if neg { 0xFF - b } else { b };
                if !(1..=10).contains(&dg) {
                    return Err(err());
                }
                digits.push(dg - 1);
            }
            if digits.is_empty() {
                return Err(err());
            }
            let s = format_decimal(neg, &digits, exp);
            Ok((AttributeValue::N(Number(s)), i))
        }
        _ => Err(err()),
    }
}
