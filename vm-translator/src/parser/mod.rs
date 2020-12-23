mod arithmetic;
mod flow;
mod func;
mod mem_access;
mod segment;
mod stack;

// use anyhow::{anyhow, Context, Result};
use arithmetic::*;
use flow::*;
use func::*;
use mem_access::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("lack tokens: {0}")]
    LackTokens(String),
    #[error("redundant tokens: {0}")]
    RedundantToken(String),
    #[error("end of file")]
    Eof,
}

impl ParseError {
    pub fn unexpected_token(tok: &str) -> Self {
        Self::UnexpectedToken(tok.to_string())
    }
    pub fn lack_tokens(tokens: &Vec<&str>) -> Self {
        let tokens = tokens.join(" ");
        Self::LackTokens(tokens)
    }
    pub fn redundant_tokens(tokens: &Vec<&str>) -> Self {
        let tokens = tokens.join(" ");
        Self::RedundantToken(tokens)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Arithmetic(Arithmetic),
    MemAccess(MemAccess),
    Flow(Flow),
    Func(Func),
}

impl Command {
    pub fn arithmetic(arith: Arithmetic) -> Self {
        Self::Arithmetic(arith)
    }
    // pub fn mem_access(seg: Segment, index: u16) -> Self {
    //     let m = MemAccess(seg, index);
    //     Self::MemAccess(m)
    // }
}

#[derive(Debug)]
pub struct Parser {
    file_name: String,
}

macro_rules! parse_arithmetic {
    ($self:ident, $tokens:expr, $cmd:expr) => {{
        let _ = Parser::check_tokens_num($tokens, 1)?;
        let cmd = Command::Arithmetic($cmd);
        Ok(cmd)
    }};
}

impl Parser {
    pub fn new(file_name: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
        }
    }

    pub fn parse(&self, input: &str) -> Result<Vec<Command>, ParseError> {
        let lines = input.lines().collect::<Vec<&str>>();
        let mut cmds = vec![];
        for line in lines.iter() {
            let tokens = line.split_whitespace().collect::<Vec<&str>>();
            if !tokens.is_empty() {
                let cmd = self.parse_line(&tokens)?;
                cmds.push(cmd);
            }
        }
        Ok(cmds)
    }

    fn parse_line(&self, tokens: &Vec<&str>) -> Result<Command, ParseError> {
        match tokens[0] {
            // arith
            "add" => self.parse_add(tokens),
            "sub" => parse_arithmetic!(self, tokens, Arithmetic::Sub),
            "neg" => parse_arithmetic!(self, tokens, Arithmetic::Neg),
            "eq" => parse_arithmetic!(self, tokens, Arithmetic::Eq),
            "gt" => parse_arithmetic!(self, tokens, Arithmetic::Gt),
            "lt" => parse_arithmetic!(self, tokens, Arithmetic::Lt),
            "and" => parse_arithmetic!(self, tokens, Arithmetic::And),
            "or" => parse_arithmetic!(self, tokens, Arithmetic::Or),
            "not" => parse_arithmetic!(self, tokens, Arithmetic::Not),
            // memory access
            "push" => todo!(),
            "pop" => todo!(),
            // segment
            "argument" => todo!(),
            "local" => todo!(),
            "static" => todo!(),
            "constant" => todo!(),
            "this" => todo!(),
            "that" => todo!(),
            "pointer" => todo!(),
            "temp" => todo!(),
            // program flow
            "label" => todo!(),
            "goto" => todo!(),
            "if-goto" => todo!(),
            // function call
            "function" => todo!(),
            "call" => todo!(),
            "return" => todo!(),
            _ => Err(ParseError::unexpected_token(tokens[0])),
        }
    }

    fn check_tokens_num(tokens: &Vec<&str>, expect: usize) -> Result<(), ParseError> {
        if tokens.len() < expect {
            Err(ParseError::lack_tokens(tokens))
        } else if tokens.len() > expect {
            Err(ParseError::redundant_tokens(tokens))
        } else {
            Ok(())
        }
    }
    fn parse_add(&self, tokens: &Vec<&str>) -> Result<Command, ParseError> {
        let _ = Parser::check_tokens_num(tokens, 1)?;
        let cmd = Command::arithmetic(Arithmetic::Add);
        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_arith() {
        let input = r###"
        add
        sub
        neg
        eq
        gt
        lt
        and
        or
        not
        "###;
        let parser = Parser::new("test");
        let actual = parser.parse(input).unwrap();
        let expect = vec![
            Command::Arithmetic(Arithmetic::Add),
            Command::Arithmetic(Arithmetic::Sub),
            Command::Arithmetic(Arithmetic::Neg),
            Command::Arithmetic(Arithmetic::Eq),
            Command::Arithmetic(Arithmetic::Gt),
            Command::Arithmetic(Arithmetic::Lt),
            Command::Arithmetic(Arithmetic::And),
            Command::Arithmetic(Arithmetic::Or),
            Command::Arithmetic(Arithmetic::Not),
        ];
        assert_eq!(actual, expect);
    }
}
