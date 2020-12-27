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
    fn keyword(key: Keyword, loc: Loc) -> Self {
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
    fn ident(s: &str, loc: Loc) -> Self {
        Self::new(TokenKind::Ident(s.to_owned()), loc)
    }
    pub fn keyword_or_ident(s: &str, loc: Loc) -> Self {
        match s {
            "class" => Token::keyword(Keyword::Class, loc),
            "constructor" => Token::keyword(Keyword::Constructor, loc),
            "function" => Token::keyword(Keyword::Func, loc),
            "method" => Token::keyword(Keyword::Method, loc),
            "field" => Token::keyword(Keyword::Field, loc),
            "static" => Token::keyword(Keyword::Static, loc),
            "var" => Token::keyword(Keyword::Var, loc),
            "int" => Token::keyword(Keyword::Int, loc),
            "char" => Token::keyword(Keyword::Char, loc),
            "boolean" => Token::keyword(Keyword::Bool, loc),
            "void" => Token::keyword(Keyword::Void, loc),
            "true" => Token::keyword(Keyword::True, loc),
            "false" => Token::keyword(Keyword::False, loc),
            "null" => Token::keyword(Keyword::Null, loc),
            "this" => Token::keyword(Keyword::This, loc),
            "let" => Token::keyword(Keyword::Let, loc),
            "do" => Token::keyword(Keyword::Do, loc),
            "if" => Token::keyword(Keyword::If, loc),
            "else" => Token::keyword(Keyword::Else, loc),
            "while" => Token::keyword(Keyword::While, loc),
            "return" => Token::keyword(Keyword::Return, loc),
            _ => Token::ident(&s, loc),
        }
    }
}
