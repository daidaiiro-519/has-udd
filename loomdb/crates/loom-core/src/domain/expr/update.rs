//! UpdateExpression の適用（spec §5.3/§5.5）。純関数: 式 × item → 新 item。
//!
//! 適合規則:
//! - 右辺の読取はすべて「元の item」に対して行う（宣言順に依存しない）
//! - SET は親パスが存在しないと `ValidationError`（トップレベルは新規作成可）
//! - リスト添字への SET は範囲内なら置換・範囲外なら末尾追加（DynamoDB 準拠）
//! - REMOVE は存在しないパスに対して no-op
//! - ADD はトップレベル属性の数値加算のみ（欠落は 0 起点 = 原子カウンタ）。
//!   集合和・DELETE（集合差）は SS/NS/BS 導入後（TODO spec §2.2）

use super::ast::{Path, PathSeg, SetOperand, SetValue, UpdateExpr};
use super::eval::{lookup_value, resolve_path, seg_name, ExprContext};
use crate::domain::attribute::{AttributeValue, Item, Number};
use crate::domain::error::DbError;
use crate::domain::number;

pub fn apply_update(expr: &UpdateExpr, item: &Item, ctx: &ExprContext) -> Result<Item, DbError> {
    // DELETE は集合型（SS/NS/BS）前提。導入までは明示的に拒否する。
    if !expr.deletes.is_empty() {
        return Err(DbError::Validation(
            "DELETE requires set types (SS/NS/BS), which are not yet supported".into(),
        ));
    }

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

    // 3) ADD（トップレベル・N のみ）
    for (path, ph) in &expr.adds {
        let name = top_level_name(path, ctx)?;
        let delta = match lookup_value(ph, ctx)? {
            AttributeValue::N(n) => n.clone(),
            other => {
                return Err(DbError::Validation(format!(
                    "ADD expects an N value, got {other:?}"
                )))
            }
        };
        let current = match item.get(&name) {
            None => Number("0".into()), // 欠落は 0 起点（DynamoDB 準拠）
            Some(AttributeValue::N(n)) => n.clone(),
            Some(other) => {
                return Err(DbError::Validation(format!(
                    "ADD target {name:?} is not N: {other:?}"
                )))
            }
        };
        out.insert(name, AttributeValue::N(number::add(&current, &delta)?));
    }

    Ok(out)
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
fn top_level_name(path: &Path, ctx: &ExprContext) -> Result<String, DbError> {
    if path.0.len() != 1 {
        return Err(DbError::Validation(
            "ADD supports only top-level attributes".into(),
        ));
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
