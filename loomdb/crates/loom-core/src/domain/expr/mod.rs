//! 式言語（spec §5）。手書き再帰下降パーサ＋純関数の AST 評価器。
//!
//! v1 実装範囲: Condition/Filter 式（§5.2）。KeyCondition（§5.1）・Update（§5.3）・
//! Projection（§5.4）は後続サイクル。

pub mod ast;
pub mod eval;
pub mod parser;

pub use ast::{CmpOp, Expr, Operand, Path, PathSeg};
pub use eval::{eval, ExprContext};
pub use parser::parse_condition;
