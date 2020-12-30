use crate::lex::token::*;

pub fn translate(tokens: &Vec<Token>) -> String {
    let head = "<tokens>".to_string();
    let mut xml = vec![head];
    for tok in tokens.iter() {
        let x = match tok.value.clone() {
            TokenKind::Keyword(key) => keyword(key),
            TokenKind::Symbol(symbol) => sym(symbol),
            TokenKind::Int(n) => int(n),
            TokenKind::String(s) => string(s),
            TokenKind::Ident(ident) => id(ident),
        };
        xml.push(x);
    }
    let last = "</tokens>".to_string();
    xml.push(last);
    xml.join("\n")
}

fn keyword(key: Keyword) -> String {
    use Keyword::*;
    let s = match key {
        Class => "class",
        Void => "void",
        Func => "function",
        True => "true",
        False => "false",
        Bool => "boolean",
        Int => "int",
        Char => "char",
        Method => "method",
        Null => "null",
        Static => "static",
        Field => "field",
        This => "this",
        Let => "let",
        Do => "do",
        If => "if",
        Else => "else",
        While => "while",
        Return => "return",
        Constructor => "constructor",
        Var => "var",
    };
    format!("<keyword> {} </keyword>", s)
}

fn sym(symbol: Symbol) -> String {
    use Symbol::*;
    let x = match symbol {
        LCurlyParen => "{",
        RCurlyParen => "}",
        LParen => "(",
        RParen => ")",    // )
        LSqParen => "[",  // [
        RSqParen => "]",  // ]
        Dot => ".",       // .
        Comma => ",",     // ,
        SemiColon => ";", // ;
        Plus => "+",      // +
        Minus => "-",     // -
        Asterisk => "*",  // *
        Slash => "/",     // /
        And => "&amp;",   // &
        Or => "|",        // |
        Lt => "&lt;",     // <
        Gt => "&gt;",     // >
        Eq => "=",        // =
        Tilde => "~",     // ~
    };
    format!("<symbol> {} </symbol>", x)
}

fn int(n: u16) -> String {
    format!("<integerConstant> {} </integerConstant>", n)
}

fn string(s: String) -> String {
    format!("<stringConstant> {} </stringConstant>", s)
}

fn id(ident: String) -> String {
    format!("<identifier> {} </identifier>", ident)
}
