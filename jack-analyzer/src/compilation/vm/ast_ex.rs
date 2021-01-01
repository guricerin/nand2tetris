use crate::parse::ast::*;
use std::fmt;

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Type::Int => "int".to_owned(),
            Type::Char => "char".to_owned(),
            Type::Bool => "boolean".to_owned(),
            Type::Void => "void".to_owned(),
            Type::Class(ident) => format!("{}", ident),
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for SubRoutineModifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "function")
    }
}

impl ExprList {
    pub fn count(&self) -> usize {
        match &self.0 {
            Some((_, ps)) => 1 + ps.len(),
            None => 0,
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            BinaryOp::Plus => "add",
            BinaryOp::Minus => "sub",
            BinaryOp::Mult => "call Math.multiply 2",
            BinaryOp::Div => "call Math.divide 2",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
            BinaryOp::Lt => "lt",
            BinaryOp::Gt => "gt",
            BinaryOp::Eq => "eq",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            UnaryOp::Minus => "neg",
            UnaryOp::Not => "not",
        };
        write!(f, "{}", s)
    }
}
