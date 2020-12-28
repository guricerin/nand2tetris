use crate::types::*;
use std::fmt;

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

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Keyword::*;
        match self {
            Class => write!(f, "class"),
            Constructor => write!(f, "constructor"),
            Func => write!(f, "function"),
            Method => write!(f, "method"),
            Field => write!(f, "field"),
            Static => write!(f, "static"),
            Var => write!(f, "var"),
            Int => write!(f, "int"),
            Char => write!(f, "char"),
            Bool => write!(f, "boolean"),
            Void => write!(f, "void"),
            True => write!(f, "true"),
            False => write!(f, "false"),
            Null => write!(f, "null"),
            This => write!(f, "this"),
            Let => write!(f, "let"),
            Do => write!(f, "do"),
            If => write!(f, "if"),
            Else => write!(f, "else"),
            While => write!(f, "while"),
            Return => write!(f, "return"),
        }
    }
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

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Symbol::*;
        match self {
            LCurlyParen => write!(f, "{{"),
            RCurlyParen => write!(f, "}}"),
            LParen => write!(f, "("),
            RParen => write!(f, ")"),
            LSqParen => write!(f, "["),
            RSqParen => write!(f, "]"),
            Dot => write!(f, "."),
            Comma => write!(f, ","),
            SemiColon => write!(f, ";"),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Asterisk => write!(f, "*"),
            Slash => write!(f, "/"),
            And => write!(f, "&"),
            Or => write!(f, "|"),
            Lt => write!(f, "<"),
            Gt => write!(f, ">"),
            Eq => write!(f, "="),
            Tilde => write!(f, "~"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Keyword(Keyword),
    Symbol(Symbol),
    Int(u16),
    String(String), // ""で囲まれたリテラル文字列
    Ident(String),  // 変数名など
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TokenKind::*;
        match self {
            Keyword(s) => s.fmt(f),
            Symbol(s) => s.fmt(f),
            Int(s) => s.fmt(f),
            String(s) => s.fmt(f),
            Ident(s) => s.fmt(f),
        }
    }
}

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
