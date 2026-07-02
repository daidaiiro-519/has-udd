//! KeyCondition の解決・照合（spec §5.1）。Query 実行の基盤。

use super::ast::{CmpOp, PathSeg, SkCond};
use super::eval::{lookup_value, seg_name, try_ord, values_equal, ExprContext};
use crate::domain::attribute::AttributeValue;
use crate::domain::error::DbError;
use std::cmp::Ordering;

/// キー属性名セグメント（Name / #Placeholder）を実属性名へ解決する。
pub fn attr_name(seg: &PathSeg, ctx: &ExprContext) -> Result<String, DbError> {
    seg_name(seg, ctx)
}

/// sk 条件を実際の sk 値に対して照合する。
pub fn sk_matches(
    cond: &SkCond,
    sk: Option<&AttributeValue>,
    ctx: &ExprContext,
) -> Result<bool, DbError> {
    let Some(sk) = sk else {
        return Ok(false); // sk なし項目は不一致（スキーマ検証で通常は到達しない）
    };
    match cond {
        SkCond::Cmp(op, ph) => {
            let v = lookup_value(ph, ctx)?;
            match op {
                CmpOp::Eq => values_equal(sk, v),
                CmpOp::Ne => Err(DbError::Validation(
                    "sort key condition does not support '<>'".into(),
                )),
                _ => match try_ord(sk, v)? {
                    None => Ok(false), // 型不一致は不一致
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
        SkCond::Between(a, b) => {
            let lo = lookup_value(a, ctx)?;
            let hi = lookup_value(b, ctx)?;
            match (try_ord(lo, sk)?, try_ord(sk, hi)?) {
                (Some(x), Some(y)) => Ok(x != Ordering::Greater && y != Ordering::Greater),
                _ => Ok(false),
            }
        }
        SkCond::BeginsWith(ph) => {
            let prefix = lookup_value(ph, ctx)?;
            match (sk, prefix) {
                (AttributeValue::S(t), AttributeValue::S(p)) => Ok(t.starts_with(p.as_str())),
                (AttributeValue::B(t), AttributeValue::B(p)) => Ok(t.starts_with(p.as_slice())),
                _ => Err(DbError::Validation(
                    "begins_with on a sort key requires S or B".into(),
                )),
            }
        }
    }
}
