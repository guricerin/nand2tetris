use super::common::*;
use super::token::*;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LexErrorKind {
    #[error("")]
    InvalidChar(char),
    #[error("eof")]
    Eof,
}

pub type LexError = Annot<LexErrorKind>;

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::LexErrorKind::*;
        let loc = &self.loc;
        match self.value {
            InvalidChar(c) => write!(f, "{}: invalid char '{}'", loc, c),
            Eof => write!(f, "End of file"),
        }
    }
}

impl LexError {
    fn invalid_char(c: char, loc: Loc) -> Self {
        LexError::new(LexErrorKind::InvalidChar(c), loc)
    }
    fn eof(loc: Loc) -> Self {
        LexError::new(LexErrorKind::Eof, loc)
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = vec![];
    let input = input.as_bytes();
    let mut pos = 0;
    macro_rules! lex_a_token {
        ($lexer:expr) => {{
            let (tok, p) = $lexer?;
            tokens.push(tok);
            pos = p;
        }};
    }

    while pos < input.len() {
        match input[pos] {
            b'0'..=b'9' => lex_a_token!(lex_number(input, pos)),
            b'+' => lex_a_token!(lex_plus(input, pos)),
            b'-' => lex_a_token!(lex_minus(input, pos)),
            b'&' => lex_a_token!(lex_and(input, pos)),
            b'|' => lex_a_token!(lex_or(input, pos)),
            b'!' => lex_a_token!(lex_not(input, pos)),
            b'@' => lex_a_token!(lex_at(input, pos)),
            b'=' => lex_a_token!(lex_eq(input, pos)),
            b';' => lex_a_token!(lex_semicolon(input, pos)),
            b'(' => lex_a_token!(lex_lparen(input, pos)),
            b')' => lex_a_token!(lex_rparen(input, pos)),
            b if available_char_in_ident_head(b as char) => {
                lex_a_token!(lex_ident(input, pos))
            }
            b' ' | b'\n' | b'\t' => {
                let ((), p) = skip_spaces(input, pos)?;
                pos = p;
            }
            b'/' => {
                if let Some(next) = peek(input, pos + 1) {
                    if next == '/' {
                        let ((), p) = skip_comment(input, pos)?;
                        pos = p;
                        continue;
                    }
                }
                // ひとつだけの`/`は認めない
                return Err(LexError::invalid_char(
                    input[pos] as char,
                    Loc::new(pos, pos + 1),
                ));
            }
            b => return Err(LexError::invalid_char(b as char, Loc::new(pos, pos + 1))),
        }
    }
    Ok(tokens)
}

/// 先読み
fn peek(input: &[u8], pos: usize) -> Option<char> {
    if input.len() <= pos {
        None
    } else {
        Some(input[pos] as char)
    }
}

/// `pos`のバイトが期待するものであれば、1バイト消費して`pos`を1進める
fn consume_byte(input: &[u8], pos: usize, b: u8) -> Result<(u8, usize), LexError> {
    if input.len() <= pos {
        return Err(LexError::eof(Loc::new(pos, pos)));
    }

    if input[pos] != b {
        return Err(LexError::invalid_char(
            input[pos] as char,
            Loc::new(pos, pos + 1),
        ));
    }

    Ok((b, pos + 1))
}

/// 条件に当てはまる入力を複数認識し、位置情報を返す
fn recognize_many(input: &[u8], mut pos: usize, mut f: impl FnMut(u8) -> bool) -> usize {
    while pos < input.len() && f(input[pos]) {
        pos += 1;
    }
    pos
}

fn lex_number(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    use std::str::from_utf8;

    let end = recognize_many(input, start, |b| b"1234567890".contains(&b));
    let n = from_utf8(&input[start..end]).unwrap().parse().unwrap();
    Ok((Token::number(n, Loc::new(start, end)), end))
}

fn available_char_in_ident_head(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$' || c == ':' || c == '.'
}

fn available_char_in_ident(c: char) -> bool {
    c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_' || c == '$' || c == ':' || c == '.'
}

fn lex_ident(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    use std::str::from_utf8;

    assert!(available_char_in_ident_head(input[start] as char));

    let end = recognize_many(input, start + 1, |b| {
        let c = b as char;
        available_char_in_ident(c)
    });

    let ident = from_utf8(&input[start..end]).unwrap().into();
    let loc = Loc::new(start, end);

    // キーワードとユーザ定義シンボルの識別
    let token = match ident {
        // memory or register
        "M" => Token::mem(MemKind::M, loc),
        "D" => Token::mem(MemKind::D, loc),
        "A" => Token::mem(MemKind::A, loc),
        "MD" => Token::mem(MemKind::MD, loc),
        "AM" => Token::mem(MemKind::AM, loc),
        "AD" => Token::mem(MemKind::AD, loc),
        "AMD" => Token::mem(MemKind::AMD, loc),
        // jump
        "JGT" => Token::jump(JumpKind::Gt, loc),
        "JEQ" => Token::jump(JumpKind::Eq, loc),
        "JGE" => Token::jump(JumpKind::Ge, loc),
        "JLT" => Token::jump(JumpKind::Lt, loc),
        "JNE" => Token::jump(JumpKind::Ne, loc),
        "JLE" => Token::jump(JumpKind::Le, loc),
        "JMP" => Token::jump(JumpKind::Jmp, loc),
        // symbol
        _ => Token::symbol(ident, loc),
    };

    Ok((token, end))
}

fn skip_spaces(input: &[u8], start: usize) -> Result<((), usize), LexError> {
    let pos = recognize_many(input, start, |b| b" \n\t".contains(&b));
    Ok(((), pos))
}

fn skip_comment(input: &[u8], start: usize) -> Result<((), usize), LexError> {
    let pos = recognize_many(input, start, |b| b != b'\n');
    Ok(((), pos))
}

fn lex_at(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'@').map(|(_, end)| (Token::at(Loc::new(start, end)), end))
}

fn lex_plus(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'+').map(|(_, end)| (Token::plus(Loc::new(start, end)), end))
}

fn lex_minus(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'-').map(|(_, end)| (Token::minus(Loc::new(start, end)), end))
}

fn lex_and(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'&').map(|(_, end)| (Token::and(Loc::new(start, end)), end))
}

fn lex_or(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'|').map(|(_, end)| (Token::or(Loc::new(start, end)), end))
}

fn lex_not(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'!').map(|(_, end)| (Token::not(Loc::new(start, end)), end))
}

fn lex_eq(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'=').map(|(_, end)| (Token::eq(Loc::new(start, end)), end))
}

fn lex_semicolon(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b';').map(|(_, end)| (Token::semicolon(Loc::new(start, end)), end))
}

fn lex_lparen(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'(').map(|(_, end)| (Token::lparen(Loc::new(start, end)), end))
}

fn lex_rparen(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b')').map(|(_, end)| (Token::rparen(Loc::new(start, end)), end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_memory() {
        let tokens = lex("D").unwrap();
        assert_eq!(tokens, vec![Token::mem(MemKind::D, Loc::new(0, 1))]);
        let tokens = lex("A").unwrap();
        assert_eq!(tokens, vec![Token::mem(MemKind::A, Loc::new(0, 1))]);
        let tokens = lex("M").unwrap();
        assert_eq!(tokens, vec![Token::mem(MemKind::M, Loc::new(0, 1))]);
        let tokens = lex("MD").unwrap();
        assert_eq!(tokens, vec![Token::mem(MemKind::MD, Loc::new(0, 2))]);
        let tokens = lex("AM").unwrap();
        assert_eq!(tokens, vec![Token::mem(MemKind::AM, Loc::new(0, 2))]);
        let tokens = lex("AD").unwrap();
        assert_eq!(tokens, vec![Token::mem(MemKind::AD, Loc::new(0, 2))]);
        let tokens = lex("AMD").unwrap();
        assert_eq!(tokens, vec![Token::mem(MemKind::AMD, Loc::new(0, 3))]);
    }

    #[test]
    fn test_lex_defined_symbol() {
        let tokens = lex("SP").unwrap();
        assert_eq!(tokens, vec![Token::symbol("SP", Loc::new(0, 2))]);
        let tokens = lex("LCL").unwrap();
        assert_eq!(tokens, vec![Token::symbol("LCL", Loc::new(0, 3))]);
        let tokens = lex("ARG").unwrap();
        assert_eq!(tokens, vec![Token::symbol("ARG", Loc::new(0, 3))]);
        let tokens = lex("THIS").unwrap();
        assert_eq!(tokens, vec![Token::symbol("THIS", Loc::new(0, 4))]);
        let tokens = lex("THAT").unwrap();
        assert_eq!(tokens, vec![Token::symbol("THAT", Loc::new(0, 4))]);
        let tokens = lex("SCREEN").unwrap();
        assert_eq!(tokens, vec![Token::symbol("SCREEN", Loc::new(0, 6))]);
        let tokens = lex("KBD").unwrap();
        assert_eq!(tokens, vec![Token::symbol("KBD", Loc::new(0, 3))]);

        for i in 0..=15 {
            let s = format!("R{}", i);
            let tokens = lex(&s).unwrap();
            assert_eq!(tokens, vec![Token::symbol(&s, Loc::new(0, s.len()))]);
        }
    }

    #[test]
    fn test_lex_op() {
        let tokens = lex("+").unwrap();
        assert_eq!(tokens, vec![Token::plus(Loc::new(0, 1))]);
        let tokens = lex("-").unwrap();
        assert_eq!(tokens, vec![Token::minus(Loc::new(0, 1))]);
        let tokens = lex("&").unwrap();
        assert_eq!(tokens, vec![Token::and(Loc::new(0, 1))]);
        let tokens = lex("|").unwrap();
        assert_eq!(tokens, vec![Token::or(Loc::new(0, 1))]);
        let tokens = lex("!").unwrap();
        assert_eq!(tokens, vec![Token::not(Loc::new(0, 1))]);
        let tokens = lex("@").unwrap();
        assert_eq!(tokens, vec![Token::at(Loc::new(0, 1))]);
        let tokens = lex("=").unwrap();
        assert_eq!(tokens, vec![Token::eq(Loc::new(0, 1))]);
        let tokens = lex(";").unwrap();
        assert_eq!(tokens, vec![Token::semicolon(Loc::new(0, 1))]);
        let tokens = lex("(").unwrap();
        assert_eq!(tokens, vec![Token::lparen(Loc::new(0, 1))]);
        let tokens = lex(")").unwrap();
        assert_eq!(tokens, vec![Token::rparen(Loc::new(0, 1))]);
    }

    #[test]
    fn test_lex_jump() {
        let tokens = lex("JGT").unwrap();
        assert_eq!(tokens, vec![Token::jump(JumpKind::Gt, Loc::new(0, 3))]);
        let tokens = lex("JEQ").unwrap();
        assert_eq!(tokens, vec![Token::jump(JumpKind::Eq, Loc::new(0, 3))]);
        let tokens = lex("JGE").unwrap();
        assert_eq!(tokens, vec![Token::jump(JumpKind::Ge, Loc::new(0, 3))]);
        let tokens = lex("JLT").unwrap();
        assert_eq!(tokens, vec![Token::jump(JumpKind::Lt, Loc::new(0, 3))]);
        let tokens = lex("JNE").unwrap();
        assert_eq!(tokens, vec![Token::jump(JumpKind::Ne, Loc::new(0, 3))]);
        let tokens = lex("JLE").unwrap();
        assert_eq!(tokens, vec![Token::jump(JumpKind::Le, Loc::new(0, 3))]);
        let tokens = lex("JMP").unwrap();
        assert_eq!(tokens, vec![Token::jump(JumpKind::Jmp, Loc::new(0, 3))]);

        let input = "0;JMP";
        let tokens = lex(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::number(0, Loc::new(0, 1)),
                Token::semicolon(Loc::new(1, 2)),
                Token::jump(JumpKind::Jmp, Loc::new(2, 5))
            ]
        );
    }

    #[test]
    fn test_lex() {
        let input = "A|D";
        let tokens = lex(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::mem(MemKind::A, Loc::new(0, 1)),
                Token::or(Loc::new(1, 2)),
                Token::mem(MemKind::D, Loc::new(2, 3)),
            ]
        );

        let input = "(LOOP)";
        let tokens = lex(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::lparen(Loc::new(0, 1)),
                Token::symbol("LOOP", Loc::new(1, 5)),
                Token::rparen(Loc::new(5, 6)),
            ]
        );
    }

    #[test]
    fn test_lex_comment() {
        let tokens = lex("//").unwrap();
        assert_eq!(tokens, vec![]);
        let tokens = lex("//    ").unwrap();
        assert_eq!(tokens, vec![]);
        let tokens = lex("  \n//    ").unwrap();
        assert_eq!(tokens, vec![]);
        let tokens = lex("//\n").unwrap();
        assert_eq!(tokens, vec![]);
        let input = "1000 // hoge kaokokok 4567uhgy7ik";
        let tokens = lex(input).unwrap();
        assert_eq!(tokens, vec![Token::number(1000, Loc::new(0, 4))]);
        let input = "100 / hoge kaokokok 4567uhgy7ik";
        assert!(lex(input).is_err());
    }

    #[test]
    fn test_lex_space() {
        let input = r###"
        //
        ///
        ////
        // //  //
            //
        "###;
        let tokens = lex(input).unwrap();
        assert_eq!(tokens, vec![]);
    }
}
