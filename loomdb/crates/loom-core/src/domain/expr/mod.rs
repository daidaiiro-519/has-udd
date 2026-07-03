//! 式言語（spec §5）。手書き再帰下降パーサ＋純関数の AST 評価器。
//!
//! 実装範囲: KeyCondition（§5.1）・Condition/Filter（§5.2）・
//! UpdateExpression（§5.3・集合和/差込み）・Projection（§5.4）。

pub mod ast;
pub mod eval;
pub mod key;
pub mod parser;
pub mod projection;
pub mod update;

pub use ast::{
    CmpOp, Expr, KeyCondition, Operand, Path, PathSeg, SetOperand, SetValue, SkCond, UpdateExpr,
};
pub use eval::{eval, ExprContext};
pub use parser::{parse_condition, parse_key_condition, parse_projection, parse_update};
pub use projection::project;
pub use update::{apply_update, touched_roots};
