use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Keyword {
    Class,
    Constructor,
    Func,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Bool,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    RCurlyParen, // {
    LCurlyParen, // }
    RParen,      // (
    LParen,      // )
    RSqParen,    // [
    LSqParen,    // ]
    Dot,         // .
    Comma,       // ,
    SemiColon,   // ;
    Plus,        // +
    Minus,       // -
    Asterisk,    // *
    Slash,       // /
    And,         // &
    Or,          // |
    Lt,          // <
    Gt,          // >
    Eq,          // =
    Tilde,       // ~
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Keyword(Keyword),
    Symbol(Symbol),
    Int(u16),
    String(String),
    Ident(String),
}

// impl Token {
//     pub fn keyword(k: Keyword) -> Self {
//         Self::Keyword(k)
//     }
//     pub fn symbol(s: Symbol) -> Self {
//         Self::Symbol(s)
//     }
//     pub fn int(n: u16) -> Self {
//         Self::Int(n)
//     }
//     pub fn string(s: &str) -> Self {
//         Self::String(s.to_owned())
//     }
//     pub fn ident(s: &str) -> Self {
//         Self::Ident(s.to_owned())
//     }
// }

pub type Token = Annot<TokenKind>;

impl Token {
    pub fn keyword(key: Keyword, loc: Loc) -> Self {
        Self::new(TokenKind::Keyword(key), loc)
    }
    pub fn symbol(s: Symbol, loc: Loc) -> Self {
        Self::new(TokenKind::Symbol(s), loc)
    }
    pub fn int(n: u16, loc: Loc) -> Self {
        Self::new(TokenKind::Int(n), loc)
    }
    // ""で囲まれている文字列
    pub fn string(s: &str, loc: Loc) -> Self {
        Self::new(TokenKind::String(s.to_owned()), loc)
    }
    pub fn ident(s: &str, loc: Loc) -> Self {
        Self::new(TokenKind::Ident(s.to_owned()), loc)
    }
}
