use std::fmt;

/// 位置情報
/// ex: Loc(4, 6) => 入力文字の4文字目から6文字目までの区間 (0-indexed)
/// [4..6) 半開区間
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Loc(usize, usize);

impl Loc {
    /// タプル構造体はフィールドがprivateだと、他モジュールからは
    /// Loc(l,r) のような初期化が不可能なので、newを書いている
    pub fn new(l: usize, r: usize) -> Self {
        Self(l, r)
    }
    pub fn merge(&self, other: &Loc) -> Self {
        use std::cmp::{max, min};
        Self(min(self.0, other.0), max(self.1, other.1))
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}

/// アノテーション
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Annot<T> {
    pub value: T,
    pub loc: Loc,
}

impl<T> Annot<T> {
    pub fn new(value: T, loc: Loc) -> Self {
        Self { value, loc }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemKind {
    M, // Memory
    D, // D Register
    A, // A Register
    MD,
    AM,
    AD,
    AMD,
}

impl fmt::Display for MemKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MemKind::*;
        match self {
            M => write!(f, "M memory"),
            D => write!(f, "D register"),
            A => write!(f, "A register"),
            MD => write!(f, "MD"),
            AM => write!(f, "AM"),
            AD => write!(f, "AD"),
            AMD => write!(f, "AMD"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JumpKind {
    Gt,
    Eq,
    Ge,
    Lt,
    Ne,
    Le,
    Jmp,
}

impl fmt::Display for JumpKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::JumpKind::*;
        match self {
            Gt => write!(f, "JGT"),
            Eq => write!(f, "JEQ"),
            Ge => write!(f, "JGE"),
            Lt => write!(f, "JLT"),
            Ne => write!(f, "JNE"),
            Le => write!(f, "JLE"),
            Jmp => write!(f, "JMP"),
        }
    }
}
