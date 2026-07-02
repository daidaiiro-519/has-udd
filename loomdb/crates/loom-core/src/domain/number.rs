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

/// 任意精度 10 進の加算（原子カウンタ・SET の +/- の基盤）。
/// 結果が 38 桁・範囲制約を破る場合は `ValidationError`。
pub fn add(a: &Number, b: &Number) -> Result<Number, DbError> {
    let x = parse_decimal(&a.0)?;
    let y = parse_decimal(&b.0)?;
    let r = add_decimal(&x, &y);
    let s = format_decimal(r.neg, &r.digits, r.exp);
    parse_decimal(&s)?; // 38 桁・指数範囲の検証を一元化
    Ok(Number(s))
}

/// 減算 = 符号反転して加算。
pub fn sub(a: &Number, b: &Number) -> Result<Number, DbError> {
    let mut y = parse_decimal(&b.0)?;
    y.neg = !y.neg && !y.digits.is_empty();
    let x = parse_decimal(&a.0)?;
    let r = add_decimal(&x, &y);
    let s = format_decimal(r.neg, &r.digits, r.exp);
    parse_decimal(&s)?;
    Ok(Number(s))
}

/// 正規形同士の加算。係数を共通スケールに揃えて桁ベクトルで筆算する。
fn add_decimal(x: &Decimal, y: &Decimal) -> Decimal {
    let clone = |d: &Decimal| Decimal {
        neg: d.neg,
        digits: d.digits.clone(),
        exp: d.exp,
    };
    if x.digits.is_empty() {
        return clone(y);
    }
    if y.digits.is_empty() {
        return clone(x);
    }
    // 値 = ±係数 × 10^scale（係数 = digits を整数と見る、scale = exp - 桁数）
    let scale_x = x.exp - x.digits.len() as i64;
    let scale_y = y.exp - y.digits.len() as i64;
    let scale = scale_x.min(scale_y);
    let mut cx = x.digits.clone();
    cx.extend(std::iter::repeat_n(0u8, (scale_x - scale) as usize));
    let mut cy = y.digits.clone();
    cy.extend(std::iter::repeat_n(0u8, (scale_y - scale) as usize));

    let (neg, mag) = if x.neg == y.neg {
        (x.neg, add_mag(&cx, &cy))
    } else {
        match cmp_mag(&cx, &cy) {
            std::cmp::Ordering::Equal => {
                return Decimal {
                    neg: false,
                    digits: Vec::new(),
                    exp: 0,
                }
            }
            std::cmp::Ordering::Greater => (x.neg, sub_mag(&cx, &cy)),
            std::cmp::Ordering::Less => (y.neg, sub_mag(&cy, &cx)),
        }
    };
    normalize(neg, mag, scale)
}

/// 係数×10^scale → 正規形（0.digits × 10^exp）。
fn normalize(neg: bool, mut mag: Vec<u8>, scale: i64) -> Decimal {
    let lead = mag.iter().take_while(|&&d| d == 0).count();
    mag.drain(..lead);
    if mag.is_empty() {
        return Decimal {
            neg: false,
            digits: mag,
            exp: 0,
        };
    }
    let exp = scale + mag.len() as i64; // 末尾ゼロ除去は exp に影響しない
    while mag.last() == Some(&0) {
        mag.pop();
    }
    Decimal {
        neg,
        digits: mag,
        exp,
    }
}

fn add_mag(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
    let mut carry = 0u8;
    let (mut i, mut j) = (a.len(), b.len());
    while i > 0 || j > 0 || carry > 0 {
        let da = if i > 0 { a[i - 1] } else { 0 };
        let db = if j > 0 { b[j - 1] } else { 0 };
        let sum = da + db + carry;
        out.push(sum % 10);
        carry = sum / 10;
        i = i.saturating_sub(1);
        j = j.saturating_sub(1);
    }
    out.reverse();
    out
}

/// a >= b 前提の絶対値減算。
fn sub_mag(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(a.len());
    let mut borrow = 0i8;
    let (mut i, mut j) = (a.len(), b.len());
    while i > 0 {
        let da = a[i - 1] as i8 - borrow;
        let db = if j > 0 { b[j - 1] as i8 } else { 0 };
        let d = da - db;
        if d < 0 {
            out.push((d + 10) as u8);
            borrow = 1;
        } else {
            out.push(d as u8);
            borrow = 0;
        }
        i -= 1;
        j = j.saturating_sub(1);
    }
    out.reverse();
    out
}

/// 先頭ゼロなし前提の絶対値比較（桁数 → 辞書式）。
fn cmp_mag(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
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
