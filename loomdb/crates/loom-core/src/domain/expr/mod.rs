//! 式言語（spec §5）。手書き再帰下降パーサ＋純関数の AST 評価器。
//!
//! v1 実装範囲: Condition/Filter 式（§5.2）と UpdateExpression（§5.3。集合和/差は
//! SS/NS/BS 導入後）。KeyCondition（§5.1）・Projection（§5.4）は後続サイクル。

pub mod ast;
pub mod eval;
pub mod key;
pub mod parser;
pub mod update;

pub use ast::{
    CmpOp, Expr, KeyCondition, Operand, Path, PathSeg, SetOperand, SetValue, SkCond, UpdateExpr,
};
pub use eval::{eval, ExprContext};
pub use parser::{parse_condition, parse_key_condition, parse_update};
pub use update::{apply_update, touched_roots};
