//! N 型（10 進・38 有効桁・spec §2.2）の解析・整形・数値比較。
//!
//! `key_codec`（順序保存エンコード）と `expr::eval`（比較演算）が共有する。

use super::attribute::Number;
use super::error::DbError;
use std::cmp::Ordering;

const MAX_SIGNIFICANT_DIGITS: usize = 38;
// 正規化指数 E（値 = 0.digits × 10^E）の範囲 = 1E-130〜9.9…E+125（spec §2.2/§11）
const EXP_MIN: i64 = -129;
const EXP_MAX: i64 = 126;

/// 正規化済み 10 進数: 値 = 0.digits × 10^exp（digits 先頭・末尾に 0 なし、空 = ゼロ）。
pub(crate) struct Decimal {
    pub neg: bool,
    pub digits: Vec<u8>,
    pub exp: i64,
}

/// 2 つの N を数値として比較する（"1200" vs "999"、"1.0" vs "1" 等を正しく扱う）。
pub fn compare(a: &Number, b: &Number) -> Result<Ordering, DbError> {
    let x = parse_decimal(&a.0)?;
    let y = parse_decimal(&b.0)?;
    let sx = sign(&x);
    let sy = sign(&y);
    if sx != sy {
        return Ok(sx.cmp(&sy));
    }
    if sx == 0 {
        return Ok(Ordering::Equal);
    }
    // 同符号の非ゼロ: 大きさ = (指数, 仮数の辞書式)。正規化済みなので prefix 比較で正しい。
    let mag = x.exp.cmp(&y.exp).then_with(|| x.digits.cmp(&y.digits));
    Ok(if sx < 0 { mag.reverse() } else { mag })
}

fn sign(d: &Decimal) -> i8 {
    if d.digits.is_empty() {
        0
    } else if d.neg {
        -1
    } else {
        1
    }
}

/// 10 進文字列（平叙形＋指数表記）を正規形へ。38 桁・指数範囲を検証。
pub(crate) fn parse_decimal(s: &str) -> Result<Decimal, DbError> {
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
pub(crate) fn format_decimal(neg: bool, digits: &[u8], exp: i64) -> String {
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
