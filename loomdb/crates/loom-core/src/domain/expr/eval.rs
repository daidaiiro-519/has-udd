//! 式の評価器（spec §5.5）。副作用なしの純関数: AST × item × プレースホルダ → 真偽。
//!
//! 適合規則（DynamoDB 準拠）:
//! - 属性欠落・型不一致の比較は **偽**（エラーにしない）
//! - 未知のプレースホルダ・不正な型記述子は `ValidationError`
//! - N の比較は数値として行う（"1.0" = "1"）
//! - S の順序は UTF-8 バイト順、size(S) は UTF-8 バイト長

use super::ast::{CmpOp, Expr, Operand, Path, PathSeg};
use crate::domain::attribute::{AttributeValue, Item, Number};
use crate::domain::error::DbError;
use crate::domain::number;
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// ExpressionAttributeNames / Values（キーは DynamoDB 同様 `#n` / `:v` の完全形）。
pub struct ExprContext<'a> {
    pub names: &'a BTreeMap<String, String>,
    pub values: &'a BTreeMap<String, AttributeValue>,
}

pub fn eval(expr: &Expr, item: &Item, ctx: &ExprContext) -> Result<bool, DbError> {
    match expr {
        Expr::And(a, b) => Ok(eval(a, item, ctx)? && eval(b, item, ctx)?),
        Expr::Or(a, b) => Ok(eval(a, item, ctx)? || eval(b, item, ctx)?),
        Expr::Not(e) => Ok(!eval(e, item, ctx)?),
        Expr::Cmp { op, left, right } => {
            let (Some(l), Some(r)) = (
                resolve_operand(left, item, ctx)?,
                resolve_operand(right, item, ctx)?,
            ) else {
                return Ok(false); // 欠落は偽
            };
            cmp_values(*op, &l, &r)
        }
        Expr::Between { target, lo, hi } => {
            let (Some(x), Some(lo), Some(hi)) = (
                resolve_operand(target, item, ctx)?,
                resolve_operand(lo, item, ctx)?,
                resolve_operand(hi, item, ctx)?,
            ) else {
                return Ok(false);
            };
            match (try_ord(&lo, &x)?, try_ord(&x, &hi)?) {
                (Some(a), Some(b)) => Ok(a != Ordering::Greater && b != Ordering::Greater),
                _ => Ok(false), // 型不一致
            }
        }
        Expr::In { target, list } => {
            let Some(x) = resolve_operand(target, item, ctx)? else {
                return Ok(false);
            };
            for cand in list {
                if let Some(c) = resolve_operand(cand, item, ctx)? {
                    if values_equal(&x, &c)? {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
        Expr::AttributeExists(path) => Ok(resolve_path(path, item, ctx)?.is_some()),
        Expr::AttributeNotExists(path) => Ok(resolve_path(path, item, ctx)?.is_none()),
        Expr::AttributeType(path, ph) => {
            let want = match lookup_value(ph, ctx)? {
                AttributeValue::S(t) => t.clone(),
                other => {
                    return Err(DbError::Validation(format!(
                        "attribute_type expects an S type descriptor, got {other:?}"
                    )))
                }
            };
            if !matches!(want.as_str(), "S" | "N" | "B" | "BOOL" | "NULL" | "M" | "L") {
                return Err(DbError::Validation(format!(
                    "unknown type descriptor {want:?}"
                )));
            }
            Ok(resolve_path(path, item, ctx)?.is_some_and(|v| type_desc(v) == want))
        }
        Expr::BeginsWith(path, op) => {
            let (Some(target), Some(prefix)) = (
                resolve_path(path, item, ctx)?.cloned(),
                resolve_operand(op, item, ctx)?,
            ) else {
                return Ok(false);
            };
            Ok(match (&target, &prefix) {
                (AttributeValue::S(t), AttributeValue::S(p)) => t.starts_with(p.as_str()),
                (AttributeValue::B(t), AttributeValue::B(p)) => t.starts_with(p),
                _ => false,
            })
        }
        Expr::Contains(path, op) => {
            let (Some(target), Some(needle)) = (
                resolve_path(path, item, ctx)?.cloned(),
                resolve_operand(op, item, ctx)?,
            ) else {
                return Ok(false);
            };
            match (&target, &needle) {
                (AttributeValue::S(t), AttributeValue::S(n)) => Ok(t.contains(n.as_str())),
                (AttributeValue::L(list), _) => {
                    for elem in list {
                        if values_equal(elem, &needle)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 解決
// ---------------------------------------------------------------------------

pub(super) fn lookup_value<'a>(
    ph: &str,
    ctx: &'a ExprContext,
) -> Result<&'a AttributeValue, DbError> {
    ctx.values
        .get(&format!(":{ph}"))
        .ok_or_else(|| DbError::Validation(format!("unknown expression attribute value :{ph}")))
}

fn resolve_operand(
    op: &Operand,
    item: &Item,
    ctx: &ExprContext,
) -> Result<Option<AttributeValue>, DbError> {
    match op {
        Operand::Value(ph) => Ok(Some(lookup_value(ph, ctx)?.clone())),
        Operand::Path(p) => Ok(resolve_path(p, item, ctx)?.cloned()),
        Operand::Size(p) => Ok(resolve_path(p, item, ctx)?
            .and_then(size_of)
            .map(|n| AttributeValue::N(Number(n.to_string())))),
    }
}

/// パスを item に対して解決。欠落は None、未知の #name は ValidationError。
pub(super) fn resolve_path<'a>(
    path: &Path,
    item: &'a Item,
    ctx: &ExprContext,
) -> Result<Option<&'a AttributeValue>, DbError> {
    let mut segs = path.0.iter();
    // 先頭セグメントは属性名（Index 始まりはパーサが弾くが防御的に検証）
    let first = segs
        .next()
        .ok_or_else(|| DbError::Validation("empty attribute path".into()))?;
    let Some(mut current) = item.get(&seg_name(first, ctx)?) else {
        return Ok(None);
    };
    for seg in segs {
        let next = match seg {
            PathSeg::Index(idx) => match current {
                AttributeValue::L(list) => list.get(*idx),
                _ => None,
            },
            _ => match current {
                AttributeValue::M(map) => map.get(&seg_name(seg, ctx)?),
                _ => None,
            },
        };
        match next {
            Some(v) => current = v,
            None => return Ok(None),
        }
    }
    Ok(Some(current))
}

/// 名前セグメントを実属性名へ（#name は ExpressionAttributeNames で解決）。
pub(super) fn seg_name(seg: &PathSeg, ctx: &ExprContext) -> Result<String, DbError> {
    match seg {
        PathSeg::Name(n) => Ok(n.clone()),
        PathSeg::Placeholder(ph) => {
            ctx.names.get(&format!("#{ph}")).cloned().ok_or_else(|| {
                DbError::Validation(format!("unknown expression attribute name #{ph}"))
            })
        }
        PathSeg::Index(_) => Err(DbError::Validation(
            "attribute path cannot start with a list index".into(),
        )),
    }
}

// ---------------------------------------------------------------------------
// 比較
// ---------------------------------------------------------------------------

fn cmp_values(op: CmpOp, l: &AttributeValue, r: &AttributeValue) -> Result<bool, DbError> {
    match op {
        CmpOp::Eq => values_equal(l, r),
        CmpOp::Ne => Ok(!values_equal(l, r)?),
        _ => match try_ord(l, r)? {
            None => Ok(false), // 順序比較できない型組合せは偽
            Some(ord) => Ok(match op {
                CmpOp::Lt => ord == Ordering::Less,
                CmpOp::Le => ord != Ordering::Greater,
                CmpOp::Gt => ord == Ordering::Greater,
                CmpOp::Ge => ord != Ordering::Less,
                CmpOp::Eq | CmpOp::Ne => unreachable!(),
            }),
        },
    }
}

/// 順序比較（S=バイト順・N=数値・B=バイト順）。それ以外の組合せは None。
pub(super) fn try_ord(a: &AttributeValue, b: &AttributeValue) -> Result<Option<Ordering>, DbError> {
    Ok(match (a, b) {
        (AttributeValue::S(x), AttributeValue::S(y)) => Some(x.cmp(y)),
        (AttributeValue::B(x), AttributeValue::B(y)) => Some(x.cmp(y)),
        (AttributeValue::N(x), AttributeValue::N(y)) => Some(number::compare(x, y)?),
        _ => None,
    })
}

/// 等価（N は数値等価・L/M は再帰・その他は構造等価）。
pub(super) fn values_equal(a: &AttributeValue, b: &AttributeValue) -> Result<bool, DbError> {
    match (a, b) {
        (AttributeValue::N(x), AttributeValue::N(y)) => {
            Ok(number::compare(x, y)? == Ordering::Equal)
        }
        (AttributeValue::L(xs), AttributeValue::L(ys)) => {
            if xs.len() != ys.len() {
                return Ok(false);
            }
            for (x, y) in xs.iter().zip(ys) {
                if !values_equal(x, y)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (AttributeValue::M(xs), AttributeValue::M(ys)) => {
            if xs.len() != ys.len() {
                return Ok(false);
            }
            for ((kx, vx), (ky, vy)) in xs.iter().zip(ys) {
                if kx != ky || !values_equal(vx, vy)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        _ => Ok(a == b),
    }
}

fn type_desc(v: &AttributeValue) -> &'static str {
    match v {
        AttributeValue::S(_) => "S",
        AttributeValue::N(_) => "N",
        AttributeValue::B(_) => "B",
        AttributeValue::Bool(_) => "BOOL",
        AttributeValue::Null => "NULL",
        AttributeValue::M(_) => "M",
        AttributeValue::L(_) => "L",
    }
}

/// size()（S=UTF-8 バイト長・B=バイト長・L/M=要素数）。他は対象外 = None。
fn size_of(v: &AttributeValue) -> Option<usize> {
    match v {
        AttributeValue::S(s) => Some(s.len()),
        AttributeValue::B(b) => Some(b.len()),
        AttributeValue::L(l) => Some(l.len()),
        AttributeValue::M(m) => Some(m.len()),
        _ => None,
    }
}
