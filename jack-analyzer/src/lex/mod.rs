pub mod token;

use super::types::*;
use std::path::PathBuf;
use thiserror::Error;
use token::*;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    #[error("{0}\n{1}\ninvalid char: {2}")]
    InvalidChar(PathBuf, Loc, char),
    #[error("{0}\n{1}\nhead 0 number")]
    HeadZero(PathBuf, Loc),
    #[error("{0}\n{1}\nint range [0 .. 32767], actual: {2}")]
    IntOverflow(PathBuf, Loc, u64),
    #[error("{0}\n{1}\nident head number: {2}")]
    IdentHeadNumber(PathBuf, Loc, char),
    #[error("{0}\n{1}\nunexpected end of file")]
    Eof(PathBuf, Loc),
}

impl LexError {
    pub fn invalid_char(path: &PathBuf, c: char, loc: Loc) -> Self {
        Self::InvalidChar(path.to_owned(), loc, c)
    }
    pub fn head_zero(path: &PathBuf, loc: Loc) -> Self {
        Self::HeadZero(path.to_owned(), loc)
    }
    pub fn int_overflow(path: &PathBuf, n: u64, loc: Loc) -> Self {
        Self::IntOverflow(path.to_owned(), loc, n)
    }
    pub fn ident_head_number(path: &PathBuf, c: char, loc: Loc) -> Self {
        Self::IdentHeadNumber(path.to_owned(), loc, c)
    }
    pub fn eof(path: &PathBuf, loc: Loc) -> Self {
        Self::Eof(path.to_owned(), loc)
    }
}

pub struct Lexer {
    row: usize,
    col: usize,
    file_path: PathBuf,
}

static TAIL_IDENT: &'static [u8] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";

impl Lexer {
    pub fn new(path: &PathBuf) -> Self {
        Self {
            row: 0,
            col: 0,
            file_path: path.to_owned(),
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
                        // 行末までコメント
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
                _ => {
                    lex_a_token!(self.keyword_or_ident(input, pos));
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
            return Err(LexError::eof(&self.file_path, Loc::new(self.row, self.col)));
        }
        if input[pos] != expect {
            return Err(LexError::invalid_char(
                &self.file_path,
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
                        None => {
                            return Err(LexError::eof(
                                &self.file_path,
                                Loc::new(self.row, self.col),
                            ))
                        }
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
            return Err(LexError::head_zero(
                &self.file_path,
                Loc::new(self.row, self.col),
            ));
        }
        // todo: 32768はいいかも。2の補数なので。
        if 32767 < n {
            return Err(LexError::int_overflow(
                &self.file_path,
                n,
                Loc::new(self.row, self.col),
            ));
        };

        let tok = Token::int(n as u16, Loc::new(self.row, self.col));
        Ok((tok, end))
    }

    fn symbol(&mut self, input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
        let mut consume = |symbol, b| {
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
                    &self.file_path,
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
                b'\n' => {
                    return Err(LexError::invalid_char(
                        &self.file_path,
                        '?',
                        Loc::new(self.row, self.col),
                    ))
                }
                _ => (),
            };
            pos += 1;
            self.col += 1;
        }
        let end = pos;
        let s = String::from_utf8(input[start + 1..end].to_vec()).unwrap();
        let tok = Token::string(&s, Loc::new(self.row, self.col));
        // 右端の " をスキップ
        Ok((tok, end + 1))
    }

    fn keyword_or_ident(&mut self, input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
        if input[start].is_ascii_digit() {
            return Err(LexError::ident_head_number(
                &self.file_path,
                input[start] as char,
                Loc::new(self.row, self.col),
            ));
        }
        let end = self.recognize_many(input, start, |b| TAIL_IDENT.contains(&b));
        let s = String::from_utf8(input[start..end].to_vec()).unwrap();

        let loc = Loc::new(self.row, self.col);
        let tok = Token::keyword_or_ident(&s, loc);
        Ok((tok, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_to_tokenkind(input: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
        lexer
            .run(input)
            .unwrap()
            .iter()
            .map(|ano| ano.value.clone())
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_lex_number() {
        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
        let actual = lexer.run("1").unwrap();
        let expect = Token::int(1, Loc::new(0, 1));
        assert_eq!(actual, vec![expect]);

        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
        let actual = lexer.run("0").unwrap();
        let expect = Token::int(0, Loc::new(0, 1));
        assert_eq!(actual, vec![expect]);

        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
        let actual = lexer.run("10").unwrap();
        let expect = Token::int(10, Loc::new(0, 2));
        assert_eq!(actual, vec![expect]);

        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
        let actual = lexer.run("32767").unwrap();
        let expect = Token::int(32767, Loc::new(0, 5));
        assert_eq!(actual, vec![expect]);
    }

    #[test]
    fn test_lex_number_err() {
        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
        let actual = lexer.run("01");
        assert!(actual.is_err(), "head zero");

        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
        let actual = lexer.run("00");
        assert!(actual.is_err(), "head zero");

        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
        let actual = lexer.run("32768");
        assert!(actual.is_err(), "overflow");
    }

    #[test]
    fn test_lex_symbol() {
        let input = "{}()[].,;+-*/&|<>=~/";
        // let mut lexer = Lexer::new();
        // let actual = lexer.run("{}()[].,;+-*/&|<>=~/").unwrap();
        let actual = lex_to_tokenkind(input);
        assert_eq!(
            actual,
            vec![
                TokenKind::Symbol(Symbol::LCurlyParen),
                TokenKind::Symbol(Symbol::RCurlyParen),
                TokenKind::Symbol(Symbol::LParen),
                TokenKind::Symbol(Symbol::RParen),
                TokenKind::Symbol(Symbol::LSqParen),
                TokenKind::Symbol(Symbol::RSqParen),
                TokenKind::Symbol(Symbol::Dot),
                TokenKind::Symbol(Symbol::Comma),
                TokenKind::Symbol(Symbol::SemiColon),
                TokenKind::Symbol(Symbol::Plus),
                TokenKind::Symbol(Symbol::Minus),
                TokenKind::Symbol(Symbol::Asterisk),
                TokenKind::Symbol(Symbol::Slash),
                TokenKind::Symbol(Symbol::And),
                TokenKind::Symbol(Symbol::Or),
                TokenKind::Symbol(Symbol::Lt),
                TokenKind::Symbol(Symbol::Gt),
                TokenKind::Symbol(Symbol::Eq),
                TokenKind::Symbol(Symbol::Tilde),
                TokenKind::Symbol(Symbol::Slash),
            ]
        );
    }

    #[test]
    fn test_lex_string() {
        let input = "\"{}()[].,;+-*/&|<>=~/\"";
        let actual = lex_to_tokenkind(input);
        let expect = Token::string("{}()[].,;+-*/&|<>=~/", Loc::new(0, 20)).value;
        assert_eq!(actual, vec![expect]);

        let input = "\"present day, present time, HAHAHAHAHAHAHAHA\"";
        let actual = lex_to_tokenkind(input);
        let expect = Token::string(
            "present day, present time, HAHAHAHAHAHAHAHA",
            Loc::new(0, 0),
        )
        .value;
        assert_eq!(actual, vec![expect]);

        let actual = lex_to_tokenkind("\"  This statement contains 0123456789.  \"");
        let expect = Token::string("  This statement contains 0123456789.  ", Loc::new(0, 0)).value;
        assert_eq!(actual, vec![expect]);
    }

    #[test]
    fn test_lex_string_err() {
        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
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
        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
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

    #[test]
    fn test_comment_last_dq() {
        let mut lexer = Lexer::new(&PathBuf::from("hoge"));
        let actual = lexer.run("// hoge \"\n").unwrap();
        assert_eq!(actual, vec![]);
    }

    #[test]
    fn test_lex_keyword() {
        fn keyword(input: &str) {
            let actual = lex_to_tokenkind(input);
            let expect = Token::keyword_or_ident(input, Loc::new(0, 0)).value;
            assert_eq!(actual.clone(), vec![expect]);
            match actual[0] {
                TokenKind::Keyword(_) => (),
                _ => panic!("{} is not keyword", input),
            };
        }

        keyword("class");
        keyword("constructor");
        keyword("function");
        keyword("method");
        keyword("field");
        keyword("static");
        keyword("var");
        keyword("int");
        keyword("char");
        keyword("boolean");
        keyword("void");
        keyword("true");
        keyword("false");
        keyword("null");
        keyword("this");
        keyword("let");
        keyword("do");
        keyword("if");
        keyword("else");
        keyword("while");
        keyword("return");
    }
}
