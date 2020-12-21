use super::common::*;
use crate::types::Address;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumOrSymbol {
    Num(u64),
    Symbol(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddrCommand {
    value: NumOrSymbol,
}

impl AddrCommand {
    fn new(value: NumOrSymbol) -> Self {
        Self { value }
    }
    pub fn num(n: u64) -> Self {
        let n = NumOrSymbol::Num(n);
        Self::new(n)
    }
    pub fn symbol(s: &str) -> Self {
        let s = NumOrSymbol::Symbol(s.to_string());
        Self::new(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UniOpKind {
    Minus,
    Not,
}

type UniOp = Annot<UniOpKind>;

impl UniOp {
    pub fn minus(loc: Loc) -> Self {
        Self::new(UniOpKind::Minus, loc)
    }
    pub fn not(loc: Loc) -> Self {
        Self::new(UniOpKind::Not, loc)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinOpKind {
    Add,
    Sub,
    And,
    Or,
}

type BinOp = Annot<BinOpKind>;

impl BinOp {
    fn add(loc: Loc) -> Self {
        Self::new(BinOpKind::Add, loc)
    }
    fn sub(loc: Loc) -> Self {
        Self::new(BinOpKind::Sub, loc)
    }
    fn and(loc: Loc) -> Self {
        Self::new(BinOpKind::And, loc)
    }
    fn or(loc: Loc) -> Self {
        Self::new(BinOpKind::Or, loc)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constant {
    Zero,
    One,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    Constant(Constant),
    Mem(MemKind),
}

impl Operand {
    pub fn constant(c: Constant) -> Self {
        Self::Constant(c)
    }
    pub fn mem(m: MemKind) -> Self {
        Self::Mem(m)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompKind {
    Constant(Constant),
    UniOp { op: UniOp, e: Operand },
    BinOp { op: BinOp, l: MemKind, r: Operand },
}

pub type Comp = Annot<CompKind>;

impl Comp {
    pub fn constant(c: Constant, loc: Loc) -> Self {
        Self::new(CompKind::Constant(c), loc)
    }
    pub fn uniop(op: UniOp, e: Operand, loc: Loc) -> Self {
        let uniop = CompKind::UniOp { op: op, e: e };
        Self::new(uniop, loc)
    }
    pub fn binop(op: BinOp, l: MemKind, r: Operand, loc: Loc) -> Self {
        let binop = CompKind::BinOp { op: op, l: l, r: r };
        Self::new(binop, loc)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompCommand {
    dest: Option<MemKind>,
    comp: Comp,
    jump: Option<JumpKind>,
}

impl CompCommand {
    pub fn new(dest: Option<MemKind>, comp: Comp, jump: Option<JumpKind>) -> Self {
        Self { dest, comp, jump }
    }
    pub fn dest(dest: MemKind, comp: Comp) -> Self {
        Self::new(Some(dest), comp, None)
    }
    pub fn jump(comp: Comp, jump: JumpKind) -> Self {
        Self::new(None, comp, Some(jump))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelCommand {
    label: String,
}

impl LabelCommand {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandKind {
    A(AddrCommand),
    C(CompCommand),
    L(LabelCommand),
}

pub type Command = Annot<CommandKind>;

impl Command {
    pub fn addr(cmd: AddrCommand, loc: Loc) -> Self {
        Self::new(CommandKind::A(cmd), loc)
    }
    pub fn comp(cmd: CompCommand, loc: Loc) -> Self {
        Self::new(CommandKind::C(cmd), loc)
    }
    pub fn label(cmd: LabelCommand, loc: Loc) -> Self {
        Self::new(CommandKind::L(cmd), loc)
    }
}
