use super::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// [0-9]+
    Number(u64),
    /// D Register
    DReg,
    /// A Register
    AReg,
    /// M
    Memory,
    /// [a-zA-Z]
    Symbol(String),
    /// jump
    Jump(String),
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
    pub fn dreg(loc: Loc) -> Self {
        Self::new(TokenKind::DReg, loc)
    }
    pub fn areg(loc: Loc) -> Self {
        Self::new(TokenKind::AReg, loc)
    }
    pub fn memory(loc: Loc) -> Self {
        Self::new(TokenKind::Memory, loc)
    }
    pub fn symbol(s: &str, loc: Loc) -> Self {
        Self::new(TokenKind::Symbol(s.to_string()), loc)
    }
    pub fn jump(s: String, loc: Loc) -> Self {
        Self::new(TokenKind::Jump(s), loc)
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
