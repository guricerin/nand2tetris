use super::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumOrSymbol {
    Num(u64),
    Symbol(String),
}

/// A命令
/// ex: @hoge, @42
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddrCommand {
    pub value: NumOrSymbol,
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

pub type UniOp = Annot<UniOpKind>;

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

pub type BinOp = Annot<BinOpKind>;

impl BinOp {
    pub fn add(loc: Loc) -> Self {
        Self::new(BinOpKind::Add, loc)
    }
    pub fn sub(loc: Loc) -> Self {
        Self::new(BinOpKind::Sub, loc)
    }
    pub fn and(loc: Loc) -> Self {
        Self::new(BinOpKind::And, loc)
    }
    pub fn or(loc: Loc) -> Self {
        Self::new(BinOpKind::Or, loc)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constant {
    Zero,
    One,
}

impl Constant {
    pub fn new(n: u64) -> Option<Self> {
        match n {
            0 => Some(Constant::Zero),
            1 => Some(Constant::One),
            _ => None,
        }
    }
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
    Mem(MemKind),
    UniOp { op: UniOp, e: Operand },
    BinOp { op: BinOp, l: MemKind, r: Operand },
}

pub type Comp = Annot<CompKind>;

impl Comp {
    pub fn constant(c: Constant, loc: Loc) -> Self {
        Self::new(CompKind::Constant(c), loc)
    }
    pub fn mem(m: MemKind, loc: Loc) -> Self {
        Self::new(CompKind::Mem(m), loc)
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

/// C命令
/// ex: dest=comp;jump
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompCommand {
    pub dest: Option<MemKind>,
    pub comp: Comp,
    pub jump: Option<JumpKind>,
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

/// 疑似命令
/// ex: (LOOP)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelCommand {
    pub label: String,
}

impl LabelCommand {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }
    pub fn label(&self) -> String {
        self.label.clone()
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
    pub fn cmd_type(&self) -> &CommandKind {
        &self.value
    }
}
