//! Condition/Filter 式の手書き再帰下降パーサ（spec §5.2・外部パーサ依存なし）。
//!
//! 優先順位: OR < AND < NOT < 一次式。キーワード（AND/OR/NOT/BETWEEN/IN）は
//! 大文字小文字非区別・予約（属性名に使うなら `#name` を使う）。関数名は小文字固定
//! （DynamoDB 準拠）。構文誤りは `ValidationError`。

use super::ast::{
    CmpOp, Expr, KeyCondition, Operand, Path, PathSeg, SetOperand, SetValue, SkCond, UpdateExpr,
};
use crate::domain::error::DbError;

/// KeyConditionExpression（spec §5.1）:
/// `pk = :v [AND (sk cmp :v | sk BETWEEN :a AND :b | begins_with(sk, :p))]`
pub fn parse_key_condition(input: &str) -> Result<KeyCondition, DbError> {
    let toks = lex(input)?;
    let mut p = Parser { toks, pos: 0 };
    let pk_name = p.parse_path_head()?;
    match p.next() {
        Some(Tok::Cmp(CmpOp::Eq)) => {}
        Some(Tok::Cmp(_)) => return Err(p.err("partition key supports '=' only")),
        _ => return Err(p.err("expected '=' after partition key")),
    }
    let pk_value = match p.next() {
        Some(Tok::ValPh(v)) => v,
        _ => return Err(p.err("expected a :value placeholder for partition key")),
    };
    let sk = if p.eat_kw("AND") {
        Some(p.parse_sk_condition()?)
    } else {
        None
    };
    if p.pos != p.toks.len() {
        return Err(p.err("unexpected trailing tokens"));
    }
    Ok(KeyCondition {
        pk_name,
        pk_value,
        sk,
    })
}

pub fn parse_condition(input: &str) -> Result<Expr, DbError> {
    let toks = lex(input)?;
    let mut p = Parser { toks, pos: 0 };
    let expr = p.parse_or()?;
    if p.pos != p.toks.len() {
        return Err(p.err("unexpected trailing tokens"));
    }
    Ok(expr)
}

/// UpdateExpression（spec §5.3）: `SET …` / `REMOVE …` / `ADD …` / `DELETE …` の
/// 句をこの順不同・各 1 回まで。アクションはカンマ区切り。
pub fn parse_update(input: &str) -> Result<UpdateExpr, DbError> {
    let toks = lex(input)?;
    let mut p = Parser { toks, pos: 0 };
    let mut upd = UpdateExpr::default();
    if p.toks.is_empty() {
        return Err(p.err("empty update expression"));
    }
    while p.pos < p.toks.len() {
        if p.eat_kw("SET") {
            if !upd.sets.is_empty() {
                return Err(p.err("duplicate SET clause"));
            }
            loop {
                let path = p.parse_path()?;
                match p.next() {
                    Some(Tok::Cmp(CmpOp::Eq)) => {}
                    _ => return Err(p.err("expected '=' in SET action")),
                }
                let value = p.parse_set_value()?;
                upd.sets.push((path, value));
                if !p.eat_comma() {
                    break;
                }
            }
        } else if p.eat_kw("REMOVE") {
            if !upd.removes.is_empty() {
                return Err(p.err("duplicate REMOVE clause"));
            }
            loop {
                upd.removes.push(p.parse_path()?);
                if !p.eat_comma() {
                    break;
                }
            }
        } else if p.eat_kw("ADD") {
            if !upd.adds.is_empty() {
                return Err(p.err("duplicate ADD clause"));
            }
            loop {
                let path = p.parse_path()?;
                let ph = match p.next() {
                    Some(Tok::ValPh(v)) => v,
                    _ => return Err(p.err("ADD expects a :value placeholder")),
                };
                upd.adds.push((path, ph));
                if !p.eat_comma() {
                    break;
                }
            }
        } else if p.eat_kw("DELETE") {
            if !upd.deletes.is_empty() {
                return Err(p.err("duplicate DELETE clause"));
            }
            loop {
                let path = p.parse_path()?;
                let ph = match p.next() {
                    Some(Tok::ValPh(v)) => v,
                    _ => return Err(p.err("DELETE expects a :value placeholder")),
                };
                upd.deletes.push((path, ph));
                if !p.eat_comma() {
                    break;
                }
            }
        } else {
            return Err(p.err("expected SET, REMOVE, ADD or DELETE"));
        }
    }
    Ok(upd)
}

// ---------------------------------------------------------------------------
// 字句
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Ident(String),
    NamePh(String), // #foo（'#' 抜きで保持）
    ValPh(String),  // :bar（':' 抜きで保持）
    Num(usize),     // リスト添字用
    LParen,
    RParen,
    Comma,
    Dot,
    LBrack,
    RBrack,
    Plus,
    Minus,
    Cmp(CmpOp),
}

fn lex(input: &str) -> Result<Vec<Tok>, DbError> {
    let err = |msg: String| DbError::Validation(format!("expression syntax error: {msg}"));
    let b = input.as_bytes();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        match c {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'(' => {
                toks.push(Tok::LParen);
                i += 1;
            }
            b')' => {
                toks.push(Tok::RParen);
                i += 1;
            }
            b',' => {
                toks.push(Tok::Comma);
                i += 1;
            }
            b'.' => {
                toks.push(Tok::Dot);
                i += 1;
            }
            b'[' => {
                toks.push(Tok::LBrack);
                i += 1;
            }
            b']' => {
                toks.push(Tok::RBrack);
                i += 1;
            }
            b'+' => {
                toks.push(Tok::Plus);
                i += 1;
            }
            b'-' => {
                toks.push(Tok::Minus);
                i += 1;
            }
            b'=' => {
                toks.push(Tok::Cmp(CmpOp::Eq));
                i += 1;
                // "==" は不正（DynamoDB は "=" のみ）
                if b.get(i) == Some(&b'=') {
                    return Err(err("unexpected '=='".into()));
                }
            }
            b'<' => {
                i += 1;
                match b.get(i) {
                    Some(b'>') => {
                        toks.push(Tok::Cmp(CmpOp::Ne));
                        i += 1;
                    }
                    Some(b'=') => {
                        toks.push(Tok::Cmp(CmpOp::Le));
                        i += 1;
                    }
                    _ => toks.push(Tok::Cmp(CmpOp::Lt)),
                }
            }
            b'>' => {
                i += 1;
                if b.get(i) == Some(&b'=') {
                    toks.push(Tok::Cmp(CmpOp::Ge));
                    i += 1;
                } else {
                    toks.push(Tok::Cmp(CmpOp::Gt));
                }
            }
            b'#' | b':' => {
                let start = i + 1;
                let end = ident_end(b, start);
                if end == start {
                    return Err(err(format!("empty placeholder at byte {i}")));
                }
                let name = input[start..end].to_string();
                toks.push(if c == b'#' {
                    Tok::NamePh(name)
                } else {
                    Tok::ValPh(name)
                });
                i = end;
            }
            b'0'..=b'9' => {
                let start = i;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                let n: usize = input[start..i]
                    .parse()
                    .map_err(|_| err("index too large".into()))?;
                toks.push(Tok::Num(n));
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let end = ident_end(b, i);
                toks.push(Tok::Ident(input[i..end].to_string()));
                i = end;
            }
            other => return Err(err(format!("unexpected character {:?}", other as char))),
        }
    }
    Ok(toks)
}

fn ident_end(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
        i += 1;
    }
    i
}

// ---------------------------------------------------------------------------
// 構文
// ---------------------------------------------------------------------------

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn err(&self, msg: &str) -> DbError {
        DbError::Validation(format!(
            "expression syntax error: {msg} (at token {})",
            self.pos
        ))
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, want: &Tok, what: &str) -> Result<(), DbError> {
        match self.next() {
            Some(ref t) if t == want => Ok(()),
            _ => Err(self.err(&format!("expected {what}"))),
        }
    }

    /// sk 条件: `sk cmp :v` | `sk BETWEEN :a AND :b` | `begins_with(sk, :p)`
    fn parse_sk_condition(&mut self) -> Result<(PathSeg, SkCond), DbError> {
        if let Some(Tok::Ident(name)) = self.peek() {
            if name == "begins_with" && self.toks.get(self.pos + 1) == Some(&Tok::LParen) {
                self.pos += 1;
                self.expect(&Tok::LParen, "'('")?;
                let sk_name = self.parse_path_head()?;
                self.expect(&Tok::Comma, "','")?;
                let ph = match self.next() {
                    Some(Tok::ValPh(v)) => v,
                    _ => return Err(self.err("begins_with expects a :value placeholder")),
                };
                self.expect(&Tok::RParen, "')'")?;
                return Ok((sk_name, SkCond::BeginsWith(ph)));
            }
        }
        let sk_name = self.parse_path_head()?;
        if let Some(Tok::Cmp(op)) = self.peek() {
            let op = *op;
            if op == CmpOp::Ne {
                return Err(self.err("sort key condition does not support '<>'"));
            }
            self.pos += 1;
            let ph = match self.next() {
                Some(Tok::ValPh(v)) => v,
                _ => return Err(self.err("expected a :value placeholder for sort key")),
            };
            return Ok((sk_name, SkCond::Cmp(op, ph)));
        }
        if self.eat_kw("BETWEEN") {
            let a = match self.next() {
                Some(Tok::ValPh(v)) => v,
                _ => return Err(self.err("BETWEEN expects :value placeholders")),
            };
            if !self.eat_kw("AND") {
                return Err(self.err("expected AND in BETWEEN"));
            }
            let b = match self.next() {
                Some(Tok::ValPh(v)) => v,
                _ => return Err(self.err("BETWEEN expects :value placeholders")),
            };
            return Ok((sk_name, SkCond::Between(a, b)));
        }
        Err(self.err("expected comparator, BETWEEN or begins_with for sort key"))
    }

    fn eat_comma(&mut self) -> bool {
        if self.peek() == Some(&Tok::Comma) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// SET の右辺（単項または `a + b` / `a - b`）。
    fn parse_set_value(&mut self) -> Result<SetValue, DbError> {
        let a = self.parse_set_operand()?;
        match self.peek() {
            Some(Tok::Plus) => {
                self.pos += 1;
                Ok(SetValue::Plus(a, self.parse_set_operand()?))
            }
            Some(Tok::Minus) => {
                self.pos += 1;
                Ok(SetValue::Minus(a, self.parse_set_operand()?))
            }
            _ => Ok(SetValue::Single(a)),
        }
    }

    /// SET の項: `:v` | `if_not_exists(path, 項)` | `list_append(項, 項)` | path
    fn parse_set_operand(&mut self) -> Result<SetOperand, DbError> {
        match self.peek() {
            Some(Tok::ValPh(_)) => {
                let Some(Tok::ValPh(v)) = self.next() else {
                    unreachable!()
                };
                Ok(SetOperand::Value(v))
            }
            Some(Tok::Ident(name))
                if name == "if_not_exists" && self.toks.get(self.pos + 1) == Some(&Tok::LParen) =>
            {
                self.pos += 1;
                self.expect(&Tok::LParen, "'('")?;
                let path = self.parse_path()?;
                self.expect(&Tok::Comma, "','")?;
                let default = self.parse_set_operand()?;
                self.expect(&Tok::RParen, "')'")?;
                Ok(SetOperand::IfNotExists(path, Box::new(default)))
            }
            Some(Tok::Ident(name))
                if name == "list_append" && self.toks.get(self.pos + 1) == Some(&Tok::LParen) =>
            {
                self.pos += 1;
                self.expect(&Tok::LParen, "'('")?;
                let a = self.parse_set_operand()?;
                self.expect(&Tok::Comma, "','")?;
                let b = self.parse_set_operand()?;
                self.expect(&Tok::RParen, "')'")?;
                Ok(SetOperand::ListAppend(Box::new(a), Box::new(b)))
            }
            _ => Ok(SetOperand::Path(self.parse_path()?)),
        }
    }

    /// 次のトークンがキーワード kw（大文字小文字非区別）なら消費して true。
    fn eat_kw(&mut self, kw: &str) -> bool {
        if let Some(Tok::Ident(s)) = self.peek() {
            if s.eq_ignore_ascii_case(kw) {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    fn parse_or(&mut self) -> Result<Expr, DbError> {
        let mut left = self.parse_and()?;
        while self.eat_kw("OR") {
            let right = self.parse_and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, DbError> {
        let mut left = self.parse_not()?;
        while self.eat_kw("AND") {
            let right = self.parse_not()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, DbError> {
        if self.eat_kw("NOT") {
            Ok(Expr::Not(Box::new(self.parse_not()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, DbError> {
        // ( cond )
        if self.peek() == Some(&Tok::LParen) {
            self.pos += 1;
            let e = self.parse_or()?;
            self.expect(&Tok::RParen, "')'")?;
            return Ok(e);
        }
        // 真偽関数（関数名は小文字固定・直後に '(' が続く場合のみ）
        if let Some(Tok::Ident(name)) = self.peek() {
            let name = name.clone();
            if self.toks.get(self.pos + 1) == Some(&Tok::LParen) && name != "size" {
                return self.parse_bool_function(&name);
            }
        }
        // operand ベースの条件
        let target = self.parse_operand()?;
        if let Some(Tok::Cmp(op)) = self.peek() {
            let op = *op;
            self.pos += 1;
            let right = self.parse_operand()?;
            return Ok(Expr::Cmp {
                op,
                left: target,
                right,
            });
        }
        if self.eat_kw("BETWEEN") {
            let lo = self.parse_operand()?;
            if !self.eat_kw("AND") {
                return Err(self.err("expected AND in BETWEEN"));
            }
            let hi = self.parse_operand()?;
            return Ok(Expr::Between { target, lo, hi });
        }
        if self.eat_kw("IN") {
            self.expect(&Tok::LParen, "'(' after IN")?;
            let mut list = vec![self.parse_operand()?];
            while self.peek() == Some(&Tok::Comma) {
                self.pos += 1;
                list.push(self.parse_operand()?);
            }
            self.expect(&Tok::RParen, "')' after IN list")?;
            return Ok(Expr::In { target, list });
        }
        Err(self.err("expected comparator, BETWEEN or IN after operand"))
    }

    fn parse_bool_function(&mut self, name: &str) -> Result<Expr, DbError> {
        self.pos += 1; // 関数名
        self.expect(&Tok::LParen, "'('")?;
        let expr = match name {
            "attribute_exists" => Expr::AttributeExists(self.parse_path()?),
            "attribute_not_exists" => Expr::AttributeNotExists(self.parse_path()?),
            "attribute_type" => {
                let path = self.parse_path()?;
                self.expect(&Tok::Comma, "','")?;
                let ph = match self.next() {
                    Some(Tok::ValPh(v)) => v,
                    _ => return Err(self.err("attribute_type expects a :value placeholder")),
                };
                Expr::AttributeType(path, ph)
            }
            "begins_with" => {
                let path = self.parse_path()?;
                self.expect(&Tok::Comma, "','")?;
                Expr::BeginsWith(path, self.parse_operand()?)
            }
            "contains" => {
                let path = self.parse_path()?;
                self.expect(&Tok::Comma, "','")?;
                Expr::Contains(path, self.parse_operand()?)
            }
            other => return Err(self.err(&format!("unknown function {other:?}"))),
        };
        self.expect(&Tok::RParen, "')'")?;
        Ok(expr)
    }

    fn parse_operand(&mut self) -> Result<Operand, DbError> {
        match self.peek() {
            Some(Tok::ValPh(_)) => {
                let Some(Tok::ValPh(v)) = self.next() else {
                    unreachable!()
                };
                Ok(Operand::Value(v))
            }
            Some(Tok::Ident(name)) if name == "size" => {
                self.pos += 1;
                self.expect(&Tok::LParen, "'(' after size")?;
                let path = self.parse_path()?;
                self.expect(&Tok::RParen, "')' after size")?;
                Ok(Operand::Size(path))
            }
            _ => Ok(Operand::Path(self.parse_path()?)),
        }
    }

    fn parse_path(&mut self) -> Result<Path, DbError> {
        let mut segs = vec![self.parse_path_head()?];
        loop {
            match self.peek() {
                Some(Tok::Dot) => {
                    self.pos += 1;
                    segs.push(self.parse_path_head()?);
                }
                Some(Tok::LBrack) => {
                    self.pos += 1;
                    let idx = match self.next() {
                        Some(Tok::Num(n)) => n,
                        _ => return Err(self.err("expected list index")),
                    };
                    self.expect(&Tok::RBrack, "']'")?;
                    segs.push(PathSeg::Index(idx));
                }
                _ => break,
            }
        }
        Ok(Path(segs))
    }

    fn parse_path_head(&mut self) -> Result<PathSeg, DbError> {
        match self.next() {
            Some(Tok::Ident(name)) => {
                // キーワードは属性名として使えない（#name を使う）
                for kw in [
                    "AND", "OR", "NOT", "BETWEEN", "IN", "SET", "REMOVE", "ADD", "DELETE",
                ] {
                    if name.eq_ignore_ascii_case(kw) {
                        return Err(self.err(&format!(
                            "reserved word {name:?} in path (use a #name placeholder)"
                        )));
                    }
                }
                Ok(PathSeg::Name(name))
            }
            Some(Tok::NamePh(ph)) => Ok(PathSeg::Placeholder(ph)),
            _ => Err(self.err("expected attribute path")),
        }
    }
}
