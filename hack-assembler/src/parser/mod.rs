pub mod command;
mod common;
mod lexer;
mod token;

use command::*;
use common::*;
use thiserror::Error;
use token::*;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unexpected token")]
    UnexpectedToken(Token),
    #[error("redundant token")]
    RedundantExpr(Token),
    #[error("unexpected eof")]
    Eof,
}

// impl ParseError

type Commands = Vec<Command>;

pub fn parse(tokens: Vec<Token>) -> Result<Commands, ParseError> {
    let mut pos = 0;
    let mut commands = vec![];

    macro_rules! parse_a_token {
        ($parser:expr) => {{
            let (cmd, p) = $parser?;
            commands.push(cmd);
            pos = p;
        }};
    }

    while pos < tokens.len() {
        let tok = tokens[pos].value.clone();
        match tok {
            // A command
            TokenKind::At => parse_a_token!(parse_acommand(&tokens, pos)),
            // L command
            TokenKind::LParen => parse_a_token!(parse_lcommand(&tokens, pos)),
            // C command
            _ => parse_a_token!(parse_ccommand(&tokens, pos)),
            // _ => return Err(ParseError::UnexpectedToken(tokens[pos].clone())),
        }
    }

    Ok(commands)
}

/// 先読み
fn peek(tokens: &Vec<Token>, pos: usize) -> Option<Token> {
    if tokens.len() <= pos {
        None
    } else {
        Some(tokens[pos].clone())
    }
}

/// `pos`のトークンが期待するものであれば、`pos`を1進める
fn consume_token(
    tokens: &Vec<Token>,
    pos: usize,
    expect: TokenKind,
) -> Result<(TokenKind, usize), ParseError> {
    if tokens.len() <= pos {
        // return Err(ParseError::eof(Loc::new(pos, pos)));
        return Err(ParseError::Eof);
    }
    let actual = tokens[pos].value.clone();
    if actual != expect {
        return Err(ParseError::UnexpectedToken(tokens[pos].clone()));
    }

    Ok((expect, pos + 1))
}

fn check_eof(tokens: &Vec<Token>, pos: usize) -> Result<(), ParseError> {
    if tokens.len() <= pos {
        return Err(ParseError::Eof);
    }
    Ok(())
}

fn parse_acommand(tokens: &Vec<Token>, start: usize) -> Result<(Command, usize), ParseError> {
    let (_, pos) = consume_token(tokens, start, TokenKind::At)?;
    let _ = check_eof(tokens, pos)?;
    let actual = tokens[pos].clone();
    match actual.value {
        TokenKind::Number(n) => {
            let cmd = AddrCommand::num(n);
            let loc = Loc::new(start, 0).merge(&actual.loc);
            let cmd = Command::addr(cmd, loc);
            Ok((cmd, pos + 1))
        }
        TokenKind::Symbol(s) => {
            let cmd = AddrCommand::symbol(&s);
            let loc = Loc::new(start, 0).merge(&actual.loc);
            let cmd = Command::addr(cmd, loc);
            Ok((cmd, pos + 1))
        }
        _ => Err(ParseError::UnexpectedToken(actual)),
    }
}

fn parse_ccommand(tokens: &Vec<Token>, start: usize) -> Result<(Command, usize), ParseError> {
    let mut pos = start;
    let dest = match tokens[pos].value.clone() {
        TokenKind::Mem(m) => {
            pos += 1;
            let (_, p) = consume_token(tokens, pos, TokenKind::Eq)?;
            pos = p;
            Some(m)
        }
        _ => None,
    };

    let _ = check_eof(tokens, pos)?;
    let (comp, mut pos) = parse_comp(&tokens, pos)?;

    let jump = if check_eof(tokens, pos).is_err() {
        None
    } else {
        match tokens[pos].value {
            TokenKind::Semicolon => {
                pos += 1;
                let _ = check_eof(tokens, pos)?;
                match tokens[pos].value.clone() {
                    TokenKind::Jump(j) => {
                        pos += 1;
                        Some(j)
                    }
                    _ => return Err(ParseError::UnexpectedToken(tokens[pos].clone())),
                }
            }
            _ => None,
        }
    };

    let cmd = CompCommand::new(dest, comp, jump);
    todo!();
    //Ok((cmd, pos))
}

fn parse_comp(tokens: &Vec<Token>, start: usize) -> Result<(Comp, usize), ParseError> {
    let mut pos = start;
    let _ = check_eof(tokens, pos)?;
    let _ = match tokens[pos].value.clone() {
        TokenKind::Number(n) => {
            let n = if n == 0 {
                Constant::Zero
            } else if n == 1 {
                Constant::One
            } else {
                return Err(ParseError::UnexpectedToken(tokens[pos].clone()));
            };
        }
        TokenKind::Mem(m) => {}
        TokenKind::Not => {}
        TokenKind::Minus => {}
        _ => return Err(ParseError::UnexpectedToken(tokens[pos].clone())),
    };
    todo!();
}

fn parse_lcommand(tokens: &Vec<Token>, start: usize) -> Result<(Command, usize), ParseError> {
    let (_, pos) = consume_token(tokens, start, TokenKind::LParen)?;
    let _ = check_eof(tokens, pos)?;
    let _ = check_eof(tokens, pos + 1)?;
    let (actual0, actual1) = (tokens[pos].clone(), tokens[pos + 1].clone());
    match (actual0.value, actual1.value) {
        (TokenKind::Symbol(s), TokenKind::RParen) => {
            let cmd = LabelCommand::new(&s);
            let loc = Loc::new(start, 0).merge(&actual1.loc);
            let cmd = Command::label(cmd, loc);
            Ok((cmd, pos + 2))
        }
        (TokenKind::Symbol(_), _) => Err(ParseError::UnexpectedToken(tokens[pos + 1].clone())),
        (_, _) => Err(ParseError::UnexpectedToken(tokens[pos].clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::*;

    #[test]
    fn test_parse_acommand() {
        let tokens = lex("@100").unwrap();
        let (actual, _) = parse_acommand(&tokens, 0).unwrap();
        let expect = AddrCommand::num(100);
        let expect = Command::addr(expect, Loc::new(0, 4));
        assert_eq!(actual, expect);

        let tokens = lex("@0").unwrap();
        let (actual, _) = parse_acommand(&tokens, 0).unwrap();
        let expect = AddrCommand::num(0);
        let expect = Command::addr(expect, Loc::new(0, 2));
        assert_eq!(actual, expect);

        let tokens = lex("@symbol").unwrap();
        let (actual, _) = parse_acommand(&tokens, 0).unwrap();
        let expect = AddrCommand::symbol("symbol");
        let expect = Command::addr(expect, Loc::new(0, 7));
        assert_eq!(actual, expect);

        let tokens = lex("@").unwrap();
        assert!(parse_acommand(&tokens, 0).is_err(), "unexpected eof");
        let tokens = lex("@JMP").unwrap();
        assert!(parse_acommand(&tokens, 0).is_err(), "unexpected keyword");
        let tokens = lex("@D").unwrap();
        assert!(parse_acommand(&tokens, 0).is_err(), "unexpected keyword");
    }

    #[test]
    fn test_parse_lcommand() {
        // let tokens = lex("(LOOP)").unwrap();
        let tokens = lex("(LOOP)").unwrap();
        let (actual, _) = parse_lcommand(&tokens, 0).unwrap();
        let expect = LabelCommand::new("LOOP");
        let expect = Command::label(expect, Loc::new(0, 6));
        assert_eq!(actual, expect);

        let tokens = lex("(LOOP").unwrap();
        assert!(parse_lcommand(&tokens, 0).is_err(), "rparen not close");
        let tokens = lex("LOOP)").unwrap();
        assert!(parse_lcommand(&tokens, 0).is_err(), "unexpected rparen");
        let tokens = lex("(0)").unwrap();
        assert!(parse_lcommand(&tokens, 0).is_err(), "unexpected number");
        let tokens = lex("(M)").unwrap();
        assert!(parse_lcommand(&tokens, 0).is_err(), "unexpected keyword");
        let tokens = lex("(JGT)").unwrap();
        assert!(parse_lcommand(&tokens, 0).is_err(), "unexpected keyword");
    }
}
