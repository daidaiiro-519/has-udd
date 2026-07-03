//! UpdateExpression の適用（spec §5.3/§5.5）。純関数: 式 × item → 新 item。
//!
//! 適合規則:
//! - 右辺の読取はすべて「元の item」に対して行う（宣言順に依存しない）
//! - SET は親パスが存在しないと `ValidationError`（トップレベルは新規作成可）
//! - リスト添字への SET は範囲内なら置換・範囲外なら末尾追加（DynamoDB 準拠）
//! - REMOVE は存在しないパスに対して no-op
//! - ADD はトップレベル属性のみ: N は数値加算（欠落は 0 起点 = 原子カウンタ）、
//!   SS/NS/BS は集合和（欠落は新規作成）
//! - DELETE はトップレベル属性の集合差のみ。空になったら属性ごと削除・
//!   欠落属性には no-op

use super::ast::{Path, PathSeg, SetOperand, SetValue, UpdateExpr};
use super::eval::{lookup_value, ns_contains, resolve_path, seg_name, ExprContext};
use crate::domain::attribute::{AttributeValue, Item, Number};
use crate::domain::error::DbError;
use crate::domain::number;

pub fn apply_update(expr: &UpdateExpr, item: &Item, ctx: &ExprContext) -> Result<Item, DbError> {
    let mut out = item.clone();

    // 1) SET — 右辺をすべて元の item で評価してから適用
    let mut staged = Vec::with_capacity(expr.sets.len());
    for (path, value) in &expr.sets {
        staged.push((path, eval_set_value(value, item, ctx)?));
    }
    for (path, value) in staged {
        set_path(&mut out, path, value, ctx)?;
    }

    // 2) REMOVE
    for path in &expr.removes {
        remove_path(&mut out, path, ctx)?;
    }

    // 3) ADD（トップレベルのみ・N は数値加算 / SS・NS・BS は集合和）
    for (path, ph) in &expr.adds {
        let name = top_level_name(path, "ADD", ctx)?;
        let delta = lookup_value(ph, ctx)?;
        match delta {
            AttributeValue::N(d) => {
                let current = match item.get(&name) {
                    None => Number("0".into()), // 欠落は 0 起点（DynamoDB 準拠）
                    Some(AttributeValue::N(n)) => n.clone(),
                    Some(other) => {
                        return Err(DbError::Validation(format!(
                            "ADD target {name:?} is not N: {other:?}"
                        )))
                    }
                };
                out.insert(name, AttributeValue::N(number::add(&current, d)?));
            }
            AttributeValue::Ss(_) | AttributeValue::Ns(_) | AttributeValue::Bs(_) => {
                let merged = set_union(item.get(&name), delta, &name)?;
                out.insert(name, merged);
            }
            other => {
                return Err(DbError::Validation(format!(
                    "ADD expects an N or set (SS/NS/BS) value, got {other:?}"
                )))
            }
        }
    }

    // 4) DELETE（トップレベルのみ・集合差。空になったら属性ごと削除）
    for (path, ph) in &expr.deletes {
        let name = top_level_name(path, "DELETE", ctx)?;
        let delta = lookup_value(ph, ctx)?;
        let Some(current) = item.get(&name) else {
            continue; // 欠落属性への DELETE は no-op
        };
        match set_difference(current, delta, &name)? {
            Some(rest) => out.insert(name, rest),
            None => out.remove(&name), // 空集合は存在しない（DynamoDB 準拠）
        };
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// 集合演算（ADD = 和・DELETE = 差）
// ---------------------------------------------------------------------------

/// 集合和。欠落属性（cur = None）は delta をそのまま新規作成。
fn set_union(
    cur: Option<&AttributeValue>,
    delta: &AttributeValue,
    name: &str,
) -> Result<AttributeValue, DbError> {
    let Some(cur) = cur else {
        return Ok(delta.clone());
    };
    match (cur, delta) {
        (AttributeValue::Ss(a), AttributeValue::Ss(b)) => {
            AttributeValue::string_set(a.iter().chain(b).cloned().collect())
        }
        (AttributeValue::Ns(a), AttributeValue::Ns(b)) => {
            AttributeValue::number_set(a.iter().chain(b).cloned().collect())
        }
        (AttributeValue::Bs(a), AttributeValue::Bs(b)) => {
            AttributeValue::binary_set(a.iter().chain(b).cloned().collect())
        }
        _ => Err(DbError::Validation(format!(
            "ADD set type mismatch on {name:?}: cannot add {delta:?} to {cur:?}"
        ))),
    }
}

/// 集合差。残りが空なら None（呼び出し側で属性ごと削除する）。
fn set_difference(
    cur: &AttributeValue,
    delta: &AttributeValue,
    name: &str,
) -> Result<Option<AttributeValue>, DbError> {
    // 正規化済み集合の部分列は正規化済みなので、variant を直接構築してよい
    let rest = match (cur, delta) {
        (AttributeValue::Ss(a), AttributeValue::Ss(b)) => {
            let rest: Vec<_> = a.iter().filter(|x| !b.contains(x)).cloned().collect();
            if rest.is_empty() {
                None
            } else {
                Some(AttributeValue::Ss(rest))
            }
        }
        (AttributeValue::Ns(a), AttributeValue::Ns(b)) => {
            let mut rest = Vec::new();
            for x in a {
                if !ns_contains(b, x)? {
                    rest.push(x.clone());
                }
            }
            if rest.is_empty() {
                None
            } else {
                Some(AttributeValue::Ns(rest))
            }
        }
        (AttributeValue::Bs(a), AttributeValue::Bs(b)) => {
            let rest: Vec<_> = a.iter().filter(|x| !b.contains(x)).cloned().collect();
            if rest.is_empty() {
                None
            } else {
                Some(AttributeValue::Bs(rest))
            }
        }
        _ => {
            return Err(DbError::Validation(format!(
                "DELETE requires matching set types on {name:?}: \
                 cannot delete {delta:?} from {cur:?}"
            )))
        }
    };
    Ok(rest)
}

/// 式が触るトップレベル属性名（キー属性の変更禁止チェックに使う）。
pub fn touched_roots(expr: &UpdateExpr, ctx: &ExprContext) -> Result<Vec<String>, DbError> {
    let mut out = Vec::new();
    for (path, _) in &expr.sets {
        out.push(head_name(path, ctx)?);
    }
    for path in &expr.removes {
        out.push(head_name(path, ctx)?);
    }
    for (path, _) in &expr.adds {
        out.push(head_name(path, ctx)?);
    }
    for (path, _) in &expr.deletes {
        out.push(head_name(path, ctx)?);
    }
    Ok(out)
}

fn head_name(path: &Path, ctx: &ExprContext) -> Result<String, DbError> {
    let first = path
        .0
        .first()
        .ok_or_else(|| DbError::Validation("empty attribute path".into()))?;
    seg_name(first, ctx)
}

/// ADD/DELETE 用: トップレベル 1 セグメントのパスのみ許可。
fn top_level_name(path: &Path, verb: &str, ctx: &ExprContext) -> Result<String, DbError> {
    if path.0.len() != 1 {
        return Err(DbError::Validation(format!(
            "{verb} supports only top-level attributes"
        )));
    }
    head_name(path, ctx)
}

// ---------------------------------------------------------------------------
// SET 右辺の評価（元 item に対して）
// ---------------------------------------------------------------------------

fn eval_set_value(
    value: &SetValue,
    item: &Item,
    ctx: &ExprContext,
) -> Result<AttributeValue, DbError> {
    match value {
        SetValue::Single(op) => eval_set_operand(op, item, ctx),
        SetValue::Plus(a, b) => arith(a, b, item, ctx, number::add),
        SetValue::Minus(a, b) => arith(a, b, item, ctx, number::sub),
    }
}

fn arith(
    a: &SetOperand,
    b: &SetOperand,
    item: &Item,
    ctx: &ExprContext,
    f: fn(&Number, &Number) -> Result<Number, DbError>,
) -> Result<AttributeValue, DbError> {
    let (va, vb) = (
        eval_set_operand(a, item, ctx)?,
        eval_set_operand(b, item, ctx)?,
    );
    match (&va, &vb) {
        (AttributeValue::N(x), AttributeValue::N(y)) => Ok(AttributeValue::N(f(x, y)?)),
        _ => Err(DbError::Validation(format!(
            "arithmetic requires N operands, got {va:?} and {vb:?}"
        ))),
    }
}

fn eval_set_operand(
    op: &SetOperand,
    item: &Item,
    ctx: &ExprContext,
) -> Result<AttributeValue, DbError> {
    match op {
        SetOperand::Value(ph) => Ok(lookup_value(ph, ctx)?.clone()),
        SetOperand::Path(p) => resolve_path(p, item, ctx)?
            .cloned()
            .ok_or_else(|| DbError::Validation("operand path does not exist in the item".into())),
        SetOperand::IfNotExists(p, default) => match resolve_path(p, item, ctx)? {
            Some(v) => Ok(v.clone()),
            None => eval_set_operand(default, item, ctx),
        },
        SetOperand::ListAppend(a, b) => {
            let (va, vb) = (
                eval_set_operand(a, item, ctx)?,
                eval_set_operand(b, item, ctx)?,
            );
            match (va, vb) {
                (AttributeValue::L(mut xs), AttributeValue::L(ys)) => {
                    xs.extend(ys);
                    Ok(AttributeValue::L(xs))
                }
                (va, vb) => Err(DbError::Validation(format!(
                    "list_append requires L operands, got {va:?} and {vb:?}"
                ))),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 書込パス（可変走査）
// ---------------------------------------------------------------------------

fn invalid_path() -> DbError {
    DbError::Validation("the document path provided in the update expression is invalid".into())
}

fn set_path(
    root: &mut Item,
    path: &Path,
    value: AttributeValue,
    ctx: &ExprContext,
) -> Result<(), DbError> {
    let head = head_name(path, ctx)?;
    if path.0.len() == 1 {
        root.insert(head, value);
        return Ok(());
    }
    let mut cur = root.get_mut(&head).ok_or_else(invalid_path)?;
    for seg in &path.0[1..path.0.len() - 1] {
        cur = match seg {
            PathSeg::Index(i) => match cur {
                AttributeValue::L(list) => list.get_mut(*i).ok_or_else(invalid_path)?,
                _ => return Err(invalid_path()),
            },
            _ => {
                let name = seg_name(seg, ctx)?;
                match cur {
                    AttributeValue::M(map) => map.get_mut(&name).ok_or_else(invalid_path)?,
                    _ => return Err(invalid_path()),
                }
            }
        };
    }
    match &path.0[path.0.len() - 1] {
        PathSeg::Index(i) => match cur {
            AttributeValue::L(list) => {
                if *i < list.len() {
                    list[*i] = value; // 範囲内は置換
                } else {
                    list.push(value); // 範囲外は末尾追加（DynamoDB 準拠）
                }
                Ok(())
            }
            _ => Err(invalid_path()),
        },
        seg => {
            let name = seg_name(seg, ctx)?;
            match cur {
                AttributeValue::M(map) => {
                    map.insert(name, value);
                    Ok(())
                }
                _ => Err(invalid_path()),
            }
        }
    }
}

/// REMOVE。途中・末端が存在しなければ no-op。
fn remove_path(root: &mut Item, path: &Path, ctx: &ExprContext) -> Result<(), DbError> {
    let head = head_name(path, ctx)?;
    if path.0.len() == 1 {
        root.remove(&head);
        return Ok(());
    }
    let Some(mut cur) = root.get_mut(&head) else {
        return Ok(());
    };
    for seg in &path.0[1..path.0.len() - 1] {
        let next = match seg {
            PathSeg::Index(i) => match cur {
                AttributeValue::L(list) => list.get_mut(*i),
                _ => None,
            },
            _ => {
                let name = seg_name(seg, ctx)?;
                match cur {
                    AttributeValue::M(map) => map.get_mut(&name),
                    _ => None,
                }
            }
        };
        match next {
            Some(v) => cur = v,
            None => return Ok(()),
        }
    }
    match &path.0[path.0.len() - 1] {
        PathSeg::Index(i) => {
            if let AttributeValue::L(list) = cur {
                if *i < list.len() {
                    list.remove(*i); // 詰める
                }
            }
        }
        seg => {
            let name = seg_name(seg, ctx)?;
            if let AttributeValue::M(map) = cur {
                map.remove(&name);
            }
        }
    }
    Ok(())
}
