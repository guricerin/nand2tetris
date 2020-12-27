mod token;

use super::types::*;
use thiserror::Error;
use token::*;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LexErrorKind {
    #[error("invalid char: {0}")]
    InvalidChar(char),
    #[error("head 0 number")]
    HeadZero,
    #[error("int range [0 .. 32767], actual: {0}")]
    IntOverflow(u64),
    #[error("unexpected end of file")]
    Eof,
}

type LexError = Annot<LexErrorKind>;

impl LexError {
    pub fn invalid_char(c: char, loc: Loc) -> Self {
        Self::new(LexErrorKind::InvalidChar(c), loc)
    }
    pub fn head_zero(loc: Loc) -> Self {
        Self::new(LexErrorKind::HeadZero, loc)
    }
    pub fn int_overflow(n: u64, loc: Loc) -> Self {
        Self::new(LexErrorKind::IntOverflow(n), loc)
    }
    pub fn eof(loc: Loc) -> Self {
        Self::new(LexErrorKind::Eof, loc)
    }
}

pub struct Lexer {
    row: usize,
    col: usize,
}

static TAIL_IDENT: &'static [u8] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";

impl Lexer {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
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
                b'0'..=b'9' => {
                    lex_a_token!(self.number(input, pos));
                }
                // symbol
                b'{' | b'}' | b'(' | b')' | b'[' | b']' | b'.' | b';' | b'+' | b'-' | b','
                | b'*' | b'=' | b'&' | b'|' | b'<' | b'>' | b'~' => {
                    lex_a_token!(self.symbol(input, pos));
                }
                b'/' => {
                    let (next1, next2) = (self.peek(input, pos + 1), self.peek(input, pos + 2));
                    match (next1, next2) {
                        // 行の終わりまでコメント
                        (Some(b'/'), _) => {
                            let p = self.recognize_many(input, pos, |b| b'\n' != b);
                            pos = p;
                        }
                        // */ までコメント 改行含む (P.198)
                        (Some(b'*'), Some(b'*')) => {
                            let (_, p) = self.skip_api_comment(input, pos + 3)?;
                            pos = p;
                        }
                        (Some(b'*'), _) => {
                            let (_, p) = self.skip_api_comment(input, pos + 2)?;
                            pos = p;
                        }
                        // 演算子
                        _ => {
                            lex_a_token!(self.symbol(input, pos));
                        }
                    }
                }
                b'"' => {
                    lex_a_token!(self.string(input, pos));
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

    fn skip_api_comment(&mut self, input: &[u8], start: usize) -> Result<((), usize), LexError> {
        let mut pos = start + 2;
        while pos < input.len() {
            match input[pos] {
                b'*' => {
                    let next = self.peek(input, pos + 1);
                    match next {
                        Some(b'/') => {
                            self.col += 2;
                            pos += 2;
                            break;
                        }
                        Some(_) => (),
                        None => return Err(LexError::eof(Loc::new(self.row, pos))),
                    }
                }
                b'\n' => {
                    self.row += 1;
                    self.col = 0;
                    pos += 1;
                    continue;
                }
                _ => (),
            }
            self.col += 1;
            pos += 1;
        }

        Ok(((), pos))
    }

    fn number(&mut self, input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
        use std::str::from_utf8;

        let end = self.recognize_many(input, start, |b| b"0123456789".contains(&b));
        let n: u64 = from_utf8(&input[start..end]).unwrap().parse().unwrap();

        // 先頭が0の数値は認めない
        if input[start] == b'0' && (end - start) > 1 {
            return Err(LexError::head_zero(Loc::new(self.row, start)));
        }
        // todo: 32768はいいかも。2の補数なので。
        if 32767 < n {
            return Err(LexError::int_overflow(n, Loc::new(self.row, start)));
        };

        let tok = Token::int(n as u16, Loc::new(self.row, start));
        Ok((tok, end))
    }

    fn symbol(&mut self, input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
        let mut consume = |symbol, b| {
            self.consume_byte(input, start, b).map(|(_, end)| {
                let tok = Token::symbol(symbol, Loc::new(self.row, start));
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
            b'{' => consume(Symbol::LCurlyParen, b)?,
            b'}' => consume(Symbol::RCurlyParen, b)?,
            b'(' => consume(Symbol::LParen, b)?,
            b')' => consume(Symbol::RParen, b)?,
            b'[' => consume(Symbol::LSqParen, b)?,
            b']' => consume(Symbol::RSqParen, b)?,
            b'.' => consume(Symbol::Dot, b)?,
            b',' => consume(Symbol::Comma, b)?,
            b';' => consume(Symbol::SemiColon, b)?,
            b'+' => consume(Symbol::Plus, b)?,
            b'-' => consume(Symbol::Minus, b)?,
            b'*' => consume(Symbol::Asterisk, b)?,
            b'/' => consume(Symbol::Slash, b)?,
            b'&' => consume(Symbol::And, b)?,
            b'|' => consume(Symbol::Or, b)?,
            b'<' => consume(Symbol::Lt, b)?,
            b'>' => consume(Symbol::Gt, b)?,
            b'=' => consume(Symbol::Eq, b)?,
            b'~' => consume(Symbol::Tilde, b)?,
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

    fn string(&mut self, input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
        // 左端の " をスキップ
        let mut pos = start + 1;
        while pos < input.len() {
            match input[pos] {
                b'"' => break,
                b'\n' => return Err(LexError::invalid_char('\n', Loc::new(self.row, self.col))),
                _ => (),
            };
            pos += 1;
            self.col += 1;
        }
        let end = pos;
        let s = String::from_utf8(input[start + 1..end].to_vec()).unwrap();
        let tok = Token::string(&s, Loc::new(self.row, start));
        // 右端の " をスキップ
        Ok((tok, end + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_number() {
        let mut lexer = Lexer::new();
        let actual = lexer.run("1").unwrap();
        let expect = Token::int(1, Loc::new(0, 0));
        assert_eq!(actual, vec![expect]);

        let mut lexer = Lexer::new();
        let actual = lexer.run("0").unwrap();
        let expect = Token::int(0, Loc::new(0, 0));
        assert_eq!(actual, vec![expect]);

        let mut lexer = Lexer::new();
        let actual = lexer.run("10").unwrap();
        let expect = Token::int(10, Loc::new(0, 0));
        assert_eq!(actual, vec![expect]);

        let mut lexer = Lexer::new();
        let actual = lexer.run("32767").unwrap();
        let expect = Token::int(32767, Loc::new(0, 0));
        assert_eq!(actual, vec![expect]);
    }

    #[test]
    fn test_lex_number_err() {
        let mut lexer = Lexer::new();
        let actual = lexer.run("01");
        assert!(actual.is_err(), "head zero");

        let mut lexer = Lexer::new();
        let actual = lexer.run("00");
        assert!(actual.is_err(), "head zero");

        let mut lexer = Lexer::new();
        let actual = lexer.run("32768");
        assert!(actual.is_err(), "overflow");
    }

    #[test]
    fn test_lex_symbol() {
        let mut lexer = Lexer::new();
        let actual = lexer.run("{}()[].,;+-*/&|<>=~/").unwrap();
        assert_eq!(
            actual,
            vec![
                Token::symbol(Symbol::LCurlyParen, Loc::new(0, 0)),
                Token::symbol(Symbol::RCurlyParen, Loc::new(0, 1)),
                Token::symbol(Symbol::LParen, Loc::new(0, 2)),
                Token::symbol(Symbol::RParen, Loc::new(0, 3)),
                Token::symbol(Symbol::LSqParen, Loc::new(0, 4)),
                Token::symbol(Symbol::RSqParen, Loc::new(0, 5)),
                Token::symbol(Symbol::Dot, Loc::new(0, 6)),
                Token::symbol(Symbol::Comma, Loc::new(0, 7)),
                Token::symbol(Symbol::SemiColon, Loc::new(0, 8)),
                Token::symbol(Symbol::Plus, Loc::new(0, 9)),
                Token::symbol(Symbol::Minus, Loc::new(0, 10)),
                Token::symbol(Symbol::Asterisk, Loc::new(0, 11)),
                Token::symbol(Symbol::Slash, Loc::new(0, 12)),
                Token::symbol(Symbol::And, Loc::new(0, 13)),
                Token::symbol(Symbol::Or, Loc::new(0, 14)),
                Token::symbol(Symbol::Lt, Loc::new(0, 15)),
                Token::symbol(Symbol::Gt, Loc::new(0, 16)),
                Token::symbol(Symbol::Eq, Loc::new(0, 17)),
                Token::symbol(Symbol::Tilde, Loc::new(0, 18)),
                Token::symbol(Symbol::Slash, Loc::new(0, 19)),
            ]
        );
    }
    #[test]
    fn test_lex_string() {
        let mut lexer = Lexer::new();
        let actual = lexer.run("\"{}()[].,;+-*/&|<>=~/\"").unwrap();
        let expect = Token::string("{}()[].,;+-*/&|<>=~/", Loc::new(0, 0));
        assert_eq!(actual, vec![expect]);

        let mut lexer = Lexer::new();
        let actual = lexer
            .run("\"present day, present time, HAHAHAHAHAHAHAHA\"")
            .unwrap();
        let expect = Token::string(
            "present day, present time, HAHAHAHAHAHAHAHA",
            Loc::new(0, 0),
        );
        assert_eq!(actual, vec![expect]);

        let mut lexer = Lexer::new();
        let actual = lexer
            .run("\"  This statement contains 0123456789.  \"")
            .unwrap();
        let expect = Token::string("  This statement contains 0123456789.  ", Loc::new(0, 0));
        assert_eq!(actual, vec![expect]);
    }

    #[test]
    fn test_lex_string_err() {
        let mut lexer = Lexer::new();
        let actual = lexer.run("\"this statement \ncontains new line.\"");
        assert!(actual.is_err(), "unexpected new line");
    }

    #[test]
    fn test_lex_comment() {
        let input = "// this is a line comment\n
/* this is a comment */ 123\n
/**this is a comment*/ **\n
/** * ** *** **** */\n
/** * ** *** **** *******/\n
/*\n

        hoge 1234567uujh lplpplplp\n
*/ //\n
/**     \n

        hoge 1234567uujh lplpplplp\n
        oo

**/ //\n
0\n
";
        let mut lexer = Lexer::new();
        let actual = lexer
            .run(input)
            .unwrap()
            .iter()
            .map(|ano| ano.value.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            vec![
                TokenKind::Int(123),
                TokenKind::Symbol(Symbol::Asterisk),
                TokenKind::Symbol(Symbol::Asterisk),
                TokenKind::Int(0),
            ]
        );
    }
}
