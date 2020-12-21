use super::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// [0-9]+
    Number(u64),
    /// D Register
    Mem(MemKind),
    /// jump
    Jump(JumpKind),
    Symbol(String),
    /// @
    At,
    /// ;
    Semicolon,
    /// +
    Plus,
    /// -
    Minus,
    /// &
    And,
    /// |
    Or,
    /// !
    Not,
    /// =
    Eq,
    /// (
    LParen,
    /// )
    RParen,
}

pub type Token = Annot<TokenKind>;

impl Token {
    pub fn number(n: u64, loc: Loc) -> Self {
        Self::new(TokenKind::Number(n), loc)
    }
    pub fn mem(m: MemKind, loc: Loc) -> Self {
        Self::new(TokenKind::Mem(m), loc)
    }
    pub fn symbol(s: &str, loc: Loc) -> Self {
        Self::new(TokenKind::Symbol(s.to_string()), loc)
    }
    pub fn jump(j: JumpKind, loc: Loc) -> Self {
        Self::new(TokenKind::Jump(j), loc)
    }
    pub fn at(loc: Loc) -> Self {
        Self::new(TokenKind::At, loc)
    }
    pub fn semicolon(loc: Loc) -> Self {
        Self::new(TokenKind::Semicolon, loc)
    }
    pub fn plus(loc: Loc) -> Self {
        Self::new(TokenKind::Plus, loc)
    }
    pub fn minus(loc: Loc) -> Self {
        Self::new(TokenKind::Minus, loc)
    }
    pub fn and(loc: Loc) -> Self {
        Self::new(TokenKind::And, loc)
    }
    pub fn or(loc: Loc) -> Self {
        Self::new(TokenKind::Or, loc)
    }
    pub fn not(loc: Loc) -> Self {
        Self::new(TokenKind::Not, loc)
    }
    pub fn eq(loc: Loc) -> Self {
        Self::new(TokenKind::Eq, loc)
    }
    pub fn lparen(loc: Loc) -> Self {
        Self::new(TokenKind::LParen, loc)
    }
    pub fn rparen(loc: Loc) -> Self {
        Self::new(TokenKind::RParen, loc)
    }
}
