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

/// KeyConditionExpression（spec §5.1）。pk は等価のみ・sk は範囲条件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCondition {
    /// pk 属性名（Name または #Placeholder）
    pub pk_name: PathSeg,
    /// pk の値プレースホルダ（`:v` → "v"）
    pub pk_value: String,
    pub sk: Option<(PathSeg, SkCond)>,
}

/// sk 条件（spec §5.1。`<>` は不可）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkCond {
    Cmp(CmpOp, String),
    Between(String, String),
    BeginsWith(String),
}

/// UpdateExpression（spec §5.3）。句ごとのアクション列。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateExpr {
    pub sets: Vec<(Path, SetValue)>,
    pub removes: Vec<Path>,
    /// ADD path :num（v1 は数値加算のみ。集合和は SS/NS/BS 導入後）
    pub adds: Vec<(Path, String)>,
    /// DELETE path :set（集合差。SS/NS/BS 導入までは評価時に Validation）
    pub deletes: Vec<(Path, String)>,
}

/// SET の右辺項。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetOperand {
    Path(Path),
    Value(String),
    IfNotExists(Path, Box<SetOperand>),
    ListAppend(Box<SetOperand>, Box<SetOperand>),
}

/// SET の右辺（単項または加減算）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetValue {
    Single(SetOperand),
    Plus(SetOperand, SetOperand),
    Minus(SetOperand, SetOperand),
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
