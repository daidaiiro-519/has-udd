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
//!   正規化し、負数は指数・仮数・終端をビット反転相当で反転して順序を逆転させる。
//!   38 有効桁・E ∈ [-129, 126]（= 1E-130〜9.9…E+125、spec §2.2/§11）。

use super::attribute::{AttributeValue, Number};
use super::error::DbError;

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

// 正規化指数 E ∈ [EXP_MIN, EXP_MAX] を 1 バイトへ（bias 後 0x00..=0xFF）
const EXP_MIN: i64 = -129;
const EXP_MAX: i64 = 126;
const EXP_BIAS: i64 = 129;

const MAX_SIGNIFICANT_DIGITS: usize = 38;

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
// N（10進・順序保存）
// ---------------------------------------------------------------------------

/// 正規化済み 10 進数: 値 = 0.digits × 10^exp（digits 先頭・末尾に 0 なし、空 = ゼロ）。
struct Decimal {
    neg: bool,
    digits: Vec<u8>,
    exp: i64,
}

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

/// 10 進文字列（平叙形＋指数表記）を正規形へ。38 桁・指数範囲は spec §2.2/§11 で検証。
fn parse_decimal(s: &str) -> Result<Decimal, DbError> {
    let err = |msg: &str| DbError::Validation(format!("invalid number {s:?}: {msg}"));
    let bytes = s.as_bytes();
    let mut i = 0;

    let mut neg = false;
    if let Some(&c) = bytes.first() {
        if c == b'+' || c == b'-' {
            neg = c == b'-';
            i = 1;
        }
    }

    let mut digits: Vec<u8> = Vec::new();
    let mut int_len: i64 = 0;
    let mut seen_dot = false;
    let mut seen_digit = false;
    while i < bytes.len() {
        match bytes[i] {
            c @ b'0'..=b'9' => {
                digits.push(c - b'0');
                if !seen_dot {
                    int_len += 1;
                }
                seen_digit = true;
                i += 1;
            }
            b'.' if !seen_dot => {
                seen_dot = true;
                i += 1;
            }
            b'e' | b'E' => break,
            _ => return Err(err("unexpected character")),
        }
    }
    if !seen_digit {
        return Err(err("no digits"));
    }

    // 指数部（任意）
    let mut exp_shift: i64 = 0;
    if i < bytes.len() {
        i += 1; // 'e' / 'E'
        let mut exp_neg = false;
        if let Some(&c) = bytes.get(i) {
            if c == b'+' || c == b'-' {
                exp_neg = c == b'-';
                i += 1;
            }
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            // 指数の絶対値は高々 3 桁で足りる（範囲検証で弾く）。桁数だけ先に制限。
            if i - start >= 6 {
                return Err(err("exponent out of range"));
            }
            exp_shift = exp_shift * 10 + i64::from(bytes[i] - b'0');
            i += 1;
        }
        if i == start || i != bytes.len() {
            return Err(err("malformed exponent"));
        }
        if exp_neg {
            exp_shift = -exp_shift;
        }
    }

    // 正規化: 値 = 0.digits × 10^exp
    let mut exp = int_len + exp_shift;
    let lead = digits.iter().take_while(|&&d| d == 0).count();
    digits.drain(..lead);
    exp -= lead as i64;
    while digits.last() == Some(&0) {
        digits.pop();
    }
    if digits.is_empty() {
        return Ok(Decimal {
            neg: false,
            digits,
            exp: 0,
        });
    }
    if digits.len() > MAX_SIGNIFICANT_DIGITS {
        return Err(err("exceeds 38 significant digits"));
    }
    if !(EXP_MIN..=EXP_MAX).contains(&exp) {
        return Err(err("magnitude out of range (1E-130..9.9E+125)"));
    }
    Ok(Decimal { neg, digits, exp })
}

/// 正規形 → canonical な平叙 10 進文字列（"0"・"1.23"・"-0.005"・"12300" 等）。
fn format_decimal(neg: bool, digits: &[u8], exp: i64) -> String {
    if digits.is_empty() {
        return "0".into();
    }
    let mut s = String::new();
    if neg {
        s.push('-');
    }
    let n = digits.len() as i64;
    let push = |s: &mut String, ds: &[u8]| {
        for &d in ds {
            s.push((b'0' + d) as char);
        }
    };
    if exp >= n {
        push(&mut s, digits);
        for _ in 0..(exp - n) {
            s.push('0');
        }
    } else if exp >= 1 {
        push(&mut s, &digits[..exp as usize]);
        s.push('.');
        push(&mut s, &digits[exp as usize..]);
    } else {
        s.push_str("0.");
        for _ in 0..(-exp) {
            s.push('0');
        }
        push(&mut s, digits);
    }
    s
}
