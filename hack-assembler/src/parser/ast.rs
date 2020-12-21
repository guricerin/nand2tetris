use super::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AstKind {
    Num(u64),
    UniOp { op: UniOp, e: Box<Ast> },
    BinOp { op: BinOp, l: Box<Ast>, r: Box<Ast> },
}
