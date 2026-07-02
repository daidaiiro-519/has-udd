//! 式の AST（spec §5.2）。不変・評価は `eval` の純関数で行う。

/// 属性パスの 1 セグメント。`#name` は評価時に ExpressionAttributeNames で解決する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSeg {
    /// 生の属性名（`a` など）
    Name(String),
    /// 名前プレースホルダ（`#a` → "a" を保持）
    Placeholder(String),
    /// リスト添字（`[0]`）
    Index(usize),
}

/// 属性パス（`a.b[0].c`）。先頭は Name/Placeholder。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(pub Vec<PathSeg>);

/// 比較の項。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Path(Path),
    /// 値プレースホルダ（`:v` → "v" を保持）
    Value(String),
    /// `size(path)`（比較の項としてのみ出現）
    Size(Path),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Condition/Filter 式（spec §5.2 の文法に 1:1 対応）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Cmp {
        op: CmpOp,
        left: Operand,
        right: Operand,
    },
    Between {
        target: Operand,
        lo: Operand,
        hi: Operand,
    },
    In {
        target: Operand,
        list: Vec<Operand>,
    },
    AttributeExists(Path),
    AttributeNotExists(Path),
    /// `attribute_type(path, :t)` — :t は型記述子（"S"/"N"/…）の S 値
    AttributeType(Path, String),
    BeginsWith(Path, Operand),
    Contains(Path, Operand),
}
