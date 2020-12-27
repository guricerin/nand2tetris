mod token;

use super::types::*;
use thiserror::Error;
use token::*;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LexErrorKind {
    #[error("invalid char: {0}")]
    InvalidChar(char),
    #[error("end of file")]
    Eof,
}

type LexError = Annot<LexErrorKind>;

impl LexError {
    pub fn invalid_char(c: char, loc: Loc) -> Self {
        Self::new(LexErrorKind::InvalidChar(c), loc)
    }
    pub fn eof(loc: Loc) -> Self {
        Self::new(LexErrorKind::Eof, loc)
    }
}

pub struct Lexer {
    row: usize,
    col: usize,
    tokens: Vec<Token>,
}

static TAIL_IDENT: &'static [u8] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";

impl Lexer {
    pub fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            tokens: vec![],
        }
    }
    pub fn run(&mut self, input: &str) -> Result<Vec<Token>, LexError> {
        let input = input.as_bytes();
        let mut pos = 0;
        let mut tokens = vec![];

        macro_rules! lex_a_token {
            ($lexpr:expr) => {{
                let (tok, p) = $lexpr?;
                tokens.push(tok);
                pos = p;
            }};
        }

        while pos < input.len() {
            match input[pos] {
                b'\n' => {
                    self.row += 1;
                    self.col = 0;
                    pos += 1;
                    continue;
                }
                b' ' | b'\t' => {
                    let (_, p) = self.skip_spaces(input, pos)?;
                    pos = p;
                }
                // symbol
                b'{' | b'}' | b'(' | b')' | b'[' | b']' | b'.' | b';' | b'+' | b'-' | b','
                | b'*' | b'=' | b'&' | b'|' | b'<' | b'>' | b'~' => {
                    lex_a_token!(self.symbol(input, pos));
                }
                b'/' => {
                    // todo: slash, comment, line comment
                }
                b'"' => {
                    // todo: string
                }
                b => {
                    // todo ident or keyword
                    return Err(LexError::invalid_char(
                        b as char,
                        Loc::new(self.row, self.col),
                    ));
                }
            }
        }

        Ok(tokens)
    }

    fn peek(&self, input: &[u8], pos: usize) -> Option<u8> {
        if pos < input.len() {
            Some(input[pos])
        } else {
            None
        }
    }

    fn consume_byte(
        &mut self,
        input: &[u8],
        pos: usize,
        expect: u8,
    ) -> Result<(u8, usize), LexError> {
        if input.len() <= pos {
            return Err(LexError::eof(Loc::new(self.row, self.col)));
        }
        if input[pos] != expect {
            return Err(LexError::invalid_char(
                input[pos] as char,
                Loc::new(self.row, self.col),
            ));
        }

        self.col += 1;
        Ok((expect, pos + 1))
    }

    /// 条件に当てはまる入力を複数認識し、位置情報を返す
    fn recognize_many(
        &mut self,
        input: &[u8],
        mut pos: usize,
        mut f: impl FnMut(u8) -> bool,
    ) -> usize {
        while pos < input.len() && f(input[pos]) {
            pos += 1;
            self.col += 1;
        }
        pos
    }

    fn skip_spaces(&mut self, input: &[u8], start: usize) -> Result<((), usize), LexError> {
        let pos = self.recognize_many(input, start, |b| b" \t".contains(&b));
        Ok(((), pos))
    }

    fn symbol(&mut self, input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
        let mut f = |symbol, b| {
            self.consume_byte(input, start, b).map(|(_, end)| {
                let tok = Token::symbol(symbol, Loc::new(self.row, self.col));
                (tok, end)
            })
        };
        let b = input[start];
        let (tok, end) = match b {
            // b'{' => self.consume_byte(input, start, b'{').map(|(_, end)| {
            //     let symbol = Symbol::LCurlyParen;
            //     let tok = Token::symbol(symbol, Loc::new(self.row, self.col));
            //     (tok, end)
            // })?,
            b'{' => f(Symbol::LCurlyParen, b)?,
            b'}' => f(Symbol::RCurlyParen, b)?,
            b'(' => f(Symbol::LParen, b)?,
            b')' => f(Symbol::RParen, b)?,
            b'[' => f(Symbol::LSqParen, b)?,
            b']' => f(Symbol::RSqParen, b)?,
            b'.' => f(Symbol::Dot, b)?,
            b',' => f(Symbol::Comma, b)?,
            b';' => f(Symbol::SemiColon, b)?,
            b'+' => f(Symbol::Plus, b)?,
            b'-' => f(Symbol::Minus, b)?,
            b'*' => f(Symbol::Asterisk, b)?,
            b'&' => f(Symbol::And, b)?,
            b'|' => f(Symbol::Pipe, b)?,
            b'<' => f(Symbol::Lt, b)?,
            b'>' => f(Symbol::Gt, b)?,
            b'=' => f(Symbol::Eq, b)?,
            b'~' => f(Symbol::Tilde, b)?,
            _ => {
                // unreachable!();
                return Err(LexError::invalid_char(
                    b as char,
                    Loc::new(self.row, self.col),
                ));
            }
        };
        Ok((tok, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_symbol() {
        let mut lexer = Lexer::new();
        let actual = lexer.run("{}()[].,;+-*&|<>=~").unwrap();
        assert_eq!(
            actual,
            vec![
                Token::symbol(Symbol::LCurlyParen, Loc::new(0, 1)),
                Token::symbol(Symbol::RCurlyParen, Loc::new(0, 2)),
                Token::symbol(Symbol::LParen, Loc::new(0, 3)),
                Token::symbol(Symbol::RParen, Loc::new(0, 4)),
                Token::symbol(Symbol::LSqParen, Loc::new(0, 5)),
                Token::symbol(Symbol::RSqParen, Loc::new(0, 6)),
                Token::symbol(Symbol::Dot, Loc::new(0, 7)),
                Token::symbol(Symbol::Comma, Loc::new(0, 8)),
                Token::symbol(Symbol::SemiColon, Loc::new(0, 9)),
                Token::symbol(Symbol::Plus, Loc::new(0, 10)),
                Token::symbol(Symbol::Minus, Loc::new(0, 11)),
                Token::symbol(Symbol::Asterisk, Loc::new(0, 12)),
                Token::symbol(Symbol::And, Loc::new(0, 13)),
                Token::symbol(Symbol::Pipe, Loc::new(0, 14)),
                Token::symbol(Symbol::Lt, Loc::new(0, 15)),
                Token::symbol(Symbol::Gt, Loc::new(0, 16)),
                Token::symbol(Symbol::Eq, Loc::new(0, 17)),
                Token::symbol(Symbol::Tilde, Loc::new(0, 18)),
            ]
        );
    }
}
