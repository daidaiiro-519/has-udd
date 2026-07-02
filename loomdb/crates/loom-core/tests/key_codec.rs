//! @spec 01-spec.md#2.3 — 順序保存エンコードの性質検証（test-standard §必須プロパティ）
//!
//! 必須プロパティ:
//! 1. 順序保存: 任意の (a, b) について 値の順序 ⇔ encode 後の辞書式順序
//! 2. round-trip: decode(encode(v)) == v（N は数値として等価・canonical 安定）
//! 3. 複合キー: (pk, sk) のタプル順序 ⇔ encode_key 後の辞書式順序
//!    （0x00 や 0xFF を含むペイロードでも壊れないこと）

use loom_core::domain::key_codec::{decode_key, encode_key, encode_value};
use loom_core::domain::{AttributeValue, Number};
use proptest::prelude::*;
use std::cmp::Ordering;

fn s(v: &str) -> AttributeValue {
    AttributeValue::S(v.to_string())
}
fn n(v: &str) -> AttributeValue {
    AttributeValue::N(Number(v.to_string()))
}
fn b(v: &[u8]) -> AttributeValue {
    AttributeValue::B(v.to_vec())
}

// ---------------------------------------------------------------------------
// 参照実装: 10進文字列の数値比較（エンコーダと独立のアルゴリズム＝整数部を桁揃え、
// 小数部を辞書式比較）。プロパティテストの「正解」として使う。
// ---------------------------------------------------------------------------

fn ref_num_cmp(a: &str, b: &str) -> Ordering {
    fn parts(s: &str) -> (bool, Vec<u8>, Vec<u8>) {
        let (neg, rest) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (int, frac) = match rest.split_once('.') {
            Some((i, f)) => (i, f),
            None => (rest, ""),
        };
        let mut int: Vec<u8> = int.bytes().map(|c| c - b'0').collect();
        let lead = int.iter().take_while(|&&d| d == 0).count();
        int.drain(..lead);
        let mut frac: Vec<u8> = frac.bytes().map(|c| c - b'0').collect();
        while frac.last() == Some(&0) {
            frac.pop();
        }
        // -0 や -0.000 は 0 と同値なので符号を落とす
        let neg = neg && !(int.is_empty() && frac.is_empty());
        (neg, int, frac)
    }
    let (an, ai, af) = parts(a);
    let (bn, bi, bf) = parts(b);
    match (an, bn) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        _ => {}
    }
    // 整数部は「桁数→辞書式」、小数部は末尾ゼロ除去済みなので辞書式＝数値順
    let mag = ai
        .len()
        .cmp(&bi.len())
        .then_with(|| ai.cmp(&bi))
        .then_with(|| af.cmp(&bf));
    if an {
        mag.reverse()
    } else {
        mag
    }
}

// ---------------------------------------------------------------------------
// 生成器
// ---------------------------------------------------------------------------

/// 平叙形の 10 進文字列（符号・先頭ゼロ・末尾ゼロ・"-0" を含む）。有効桁 ≤30 で
/// spec §2.2 の 38 桁制限内に収める。
fn decimal_str() -> impl Strategy<Value = String> {
    (
        any::<bool>(),
        proptest::collection::vec(0u8..10, 0..16),
        proptest::collection::vec(0u8..10, 0..16),
    )
        .prop_map(|(neg, int, frac)| {
            let mut out = String::new();
            if neg {
                out.push('-');
            }
            if int.is_empty() {
                out.push('0');
            } else {
                for d in int {
                    out.push((b'0' + d) as char);
                }
            }
            if !frac.is_empty() {
                out.push('.');
                for d in frac {
                    out.push((b'0' + d) as char);
                }
            }
            out
        })
}

/// 区切り衝突を突く文字列（NUL・高位コードポイント混じり）。
fn key_str() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            Just('\u{0}'),
            Just('a'),
            Just('b'),
            Just('\u{7f}'),
            Just('\u{ff}'),
            Just('あ'),
        ],
        0..8,
    )
    .prop_map(|cs| cs.into_iter().collect())
}

/// 区切り衝突を突くバイト列（0x00 / 0xFF を高頻度で含む）。
fn key_bytes() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(
        prop_oneof![Just(0x00u8), Just(0x01), Just(0x61), Just(0xFE), Just(0xFF)],
        0..8,
    )
}

// ---------------------------------------------------------------------------
// プロパティ
// ---------------------------------------------------------------------------

proptest! {
    /// N: 数値順序 ⇔ エンコード辞書式順序（負数・小数・ゼロ含む）
    #[test]
    fn n_order_preserved(a in decimal_str(), b in decimal_str()) {
        let ea = encode_value(&n(&a)).unwrap();
        let eb = encode_value(&n(&b)).unwrap();
        prop_assert_eq!(ref_num_cmp(&a, &b), ea.cmp(&eb), "a={} b={}", a, b);
    }

    /// N: round-trip（数値として等価）＋ canonical 安定（再エンコードで不動点）
    #[test]
    fn n_round_trip(a in decimal_str()) {
        let ea = encode_key(&n(&a), None).unwrap();
        let (pk, sk) = decode_key(&ea).unwrap();
        prop_assert!(sk.is_none());
        let AttributeValue::N(Number(canon)) = pk else {
            return Err(TestCaseError::fail("decoded type must be N"));
        };
        prop_assert_eq!(ref_num_cmp(&canon, &a), Ordering::Equal, "canon={} a={}", canon, a);
        let e2 = encode_key(&n(&canon), None).unwrap();
        prop_assert_eq!(ea, e2);
    }

    /// S 複合キー: (pk, sk) のバイト列タプル順序 ⇔ エンコード順序
    #[test]
    fn s_composite_order(p1 in key_str(), s1 in key_str(), p2 in key_str(), s2 in key_str()) {
        let k1 = encode_key(&s(&p1), Some(&s(&s1))).unwrap();
        let k2 = encode_key(&s(&p2), Some(&s(&s2))).unwrap();
        let expected = (p1.as_bytes(), s1.as_bytes()).cmp(&(p2.as_bytes(), s2.as_bytes()));
        prop_assert_eq!(expected, k1.cmp(&k2), "p1={:?} s1={:?} p2={:?} s2={:?}", p1, s1, p2, s2);
    }

    /// B 複合キー: 0x00/0xFF を含む生バイトでもタプル順序が保たれる
    #[test]
    fn b_composite_order(p1 in key_bytes(), s1 in key_bytes(), p2 in key_bytes(), s2 in key_bytes()) {
        let k1 = encode_key(&b(&p1), Some(&b(&s1))).unwrap();
        let k2 = encode_key(&b(&p2), Some(&b(&s2))).unwrap();
        let expected = (&p1, &s1).cmp(&(&p2, &s2));
        prop_assert_eq!(expected, k1.cmp(&k2), "p1={:?} s1={:?} p2={:?} s2={:?}", p1, s1, p2, s2);
    }

    /// S/B: round-trip は完全一致
    #[test]
    fn s_b_round_trip(p in key_str(), q in key_bytes()) {
        let k = encode_key(&s(&p), Some(&b(&q))).unwrap();
        let (pk, sk) = decode_key(&k).unwrap();
        prop_assert_eq!(pk, s(&p));
        prop_assert_eq!(sk, Some(b(&q)));
    }
}

// ---------------------------------------------------------------------------
// 表駆動の単体テスト
// ---------------------------------------------------------------------------

/// 既知の並び: エンコード後も昇順のまま
#[test]
fn n_known_ordering() {
    let cases = [
        "-1200", "-2", "-1.5", "-1", "-0.11", "-0.1", "0", "0.001", "0.1", "0.5", "1", "1.5", "2",
        "10", "10.01", "1200",
    ];
    let encoded: Vec<Vec<u8>> = cases.iter().map(|c| encode_value(&n(c)).unwrap()).collect();
    let mut sorted = encoded.clone();
    sorted.sort();
    assert_eq!(encoded, sorted, "encoding must preserve numeric order");
}

/// ゼロの表記ゆれはすべて同一エンコード
#[test]
fn n_zero_forms() {
    let z = encode_value(&n("0")).unwrap();
    for form in ["-0", "0.0", "0.000", "-0.00", "+0"] {
        assert_eq!(z, encode_value(&n(form)).unwrap(), "form={form}");
    }
}

/// 指数表記の入力は平叙形と同一エンコード
#[test]
fn n_scientific_input() {
    assert_eq!(
        encode_value(&n("1e3")).unwrap(),
        encode_value(&n("1000")).unwrap()
    );
    assert_eq!(
        encode_value(&n("-1.5E-2")).unwrap(),
        encode_value(&n("-0.015")).unwrap()
    );
}

/// 不正入力・spec §2.2/§11 の範囲逸脱は ValidationError
#[test]
fn n_validation_errors() {
    let too_many_digits = "1".repeat(39);
    for bad in [
        "",
        "-",
        "abc",
        "1.2.3",
        "--1",
        "1e",
        &too_many_digits,
        "1e127",
        "1e-131",
    ] {
        assert!(encode_value(&n(bad)).is_err(), "must reject {bad:?}");
    }
    // 境界内は受理
    for ok in ["9e125", "1e-130"] {
        assert!(encode_value(&n(ok)).is_ok(), "must accept {ok:?}");
    }
}

/// キーに使えるのは S/N/B のみ（spec §2.3）
#[test]
fn key_type_must_be_s_n_b() {
    assert!(encode_value(&AttributeValue::Bool(true)).is_err());
    assert!(encode_value(&AttributeValue::Null).is_err());
}
