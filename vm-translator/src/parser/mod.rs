pub mod arithmetic;
pub mod flow;
pub mod func;
pub mod mem_access;
pub mod segment;

use arithmetic::*;
use flow::*;
use func::*;
use mem_access::*;
use segment::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("lack tokens: {0}")]
    LackTokens(String),
    #[error("redundant tokens: {0}")]
    RedundantTokens(String),
    #[error(transparent)]
    ParseNum(#[from] std::num::ParseIntError),
    #[error("`{0}` is not permited as label")]
    NotPermitLabel(String),
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
        Self::RedundantTokens(tokens)
    }
    pub fn not_permit_label(label: &str) -> Self {
        Self::NotPermitLabel(label.to_owned())
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
    pub fn push(seg: Segment, n: u16) -> Self {
        Self::MemAccess(MemAccess::Push(seg, n))
    }
    pub fn pop(seg: Segment, n: u16) -> Self {
        Self::MemAccess(MemAccess::Pop(seg, n))
    }
    pub fn label(label: &str) -> Self {
        Self::Flow(Flow::Label(label.to_string()))
    }
    pub fn goto(label: &str) -> Self {
        Self::Flow(Flow::Goto(label.to_string()))
    }
    pub fn ifgoto(label: &str) -> Self {
        Self::Flow(Flow::IfGoto(label.to_string()))
    }
    pub fn func(f: &str, n: u16) -> Self {
        Self::Func(Func::Func {
            name: f.to_string(),
            paramc: n,
        })
    }
    pub fn call(f: &str, n: u16) -> Self {
        Self::Func(Func::Call {
            name: f.to_string(),
            argc: n,
        })
    }
    pub fn f_return() -> Self {
        Self::Func(Func::Return)
    }
}

pub fn parse(input: &str) -> Result<Vec<Command>, ParseError> {
    let lines = input.lines().collect::<Vec<&str>>();
    let mut cmds = vec![];
    for line in lines.iter() {
        let line = if let Some(i) = line.find("//") {
            &line[0..i]
        } else {
            line
        };
        let tokens = line.split_whitespace().collect::<Vec<&str>>();
        if !tokens.is_empty() {
            let cmd = parse_line(&tokens)?;
            cmds.push(cmd);
        }
    }
    Ok(cmds)
}

macro_rules! parse_arithmetic {
    ($tokens:expr, $arith:expr) => {{
        let _ = check_tokens_num($tokens, 1)?;
        let cmd = Command::Arithmetic($arith);
        Ok(cmd)
    }};
}

fn parse_line(tokens: &Vec<&str>) -> Result<Command, ParseError> {
    match tokens[0] {
        // arith
        "add" => parse_arithmetic!(tokens, Arithmetic::Add),
        "sub" => parse_arithmetic!(tokens, Arithmetic::Sub),
        "neg" => parse_arithmetic!(tokens, Arithmetic::Neg),
        "eq" => parse_arithmetic!(tokens, Arithmetic::Eq),
        "gt" => parse_arithmetic!(tokens, Arithmetic::Gt),
        "lt" => parse_arithmetic!(tokens, Arithmetic::Lt),
        "and" => parse_arithmetic!(tokens, Arithmetic::And),
        "or" => parse_arithmetic!(tokens, Arithmetic::Or),
        "not" => parse_arithmetic!(tokens, Arithmetic::Not),
        // memory access
        "push" => parse_push(tokens),
        "pop" => parse_pop(tokens),
        // program flow
        "label" => parse_label(tokens),
        "goto" => parse_goto(tokens),
        "if-goto" => parse_ifgoto(tokens),
        // function call
        "function" => parse_func(tokens),
        "call" => parse_call(tokens),
        "return" => parse_return(tokens),
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

fn parse_push(tokens: &Vec<&str>) -> Result<Command, ParseError> {
    let _ = check_tokens_num(tokens, 3)?;
    let seg = parse_segment(tokens[1])?;
    let n = parse_num(tokens[2])?;
    let cmd = Command::push(seg, n);
    Ok(cmd)
}

fn parse_pop(tokens: &Vec<&str>) -> Result<Command, ParseError> {
    let _ = check_tokens_num(tokens, 3)?;
    let seg = parse_segment(tokens[1])?;
    let n = parse_num(tokens[2])?;
    let cmd = Command::pop(seg, n);
    Ok(cmd)
}

fn parse_segment(tok: &str) -> Result<Segment, ParseError> {
    let seg = match tok {
        // segment
        "argument" => Segment::Arg,
        "local" => Segment::Local,
        "static" => Segment::Static,
        "constant" => Segment::Constant,
        "this" => Segment::This,
        "that" => Segment::That,
        "pointer" => Segment::Pointer,
        "temp" => Segment::Temp,
        _ => return Err(ParseError::unexpected_token(tok)),
    };
    Ok(seg)
}

// 先頭の数値を許容してしまうので直せ
// 直したが、なぜ以下の正規表現が機能していないかいまだにわからん
fn verify_label(label: &str) -> Result<(), ParseError> {
    use regex::Regex;

    let head = label.as_bytes()[0];
    if head.is_ascii_digit() {
        return Err(ParseError::not_permit_label(label));
    }
    let re = Regex::new(r"[a-zA-Z\.:_]+[0-9a-zA-Z\.:_]*").unwrap();
    if re.is_match(label) {
        Ok(())
    } else {
        Err(ParseError::not_permit_label(label))
    }
}

fn parse_label(tokens: &Vec<&str>) -> Result<Command, ParseError> {
    let _ = check_tokens_num(tokens, 2)?;
    let label = tokens[1];
    let _ = verify_label(label)?;
    let cmd = Command::label(label);
    Ok(cmd)
}

fn parse_goto(tokens: &Vec<&str>) -> Result<Command, ParseError> {
    let _ = check_tokens_num(tokens, 2)?;
    let label = tokens[1];
    let cmd = Command::goto(label);
    Ok(cmd)
}

fn parse_ifgoto(tokens: &Vec<&str>) -> Result<Command, ParseError> {
    let _ = check_tokens_num(tokens, 2)?;
    let label = tokens[1];
    let cmd = Command::ifgoto(label);
    Ok(cmd)
}

fn parse_func(tokens: &Vec<&str>) -> Result<Command, ParseError> {
    let _ = check_tokens_num(tokens, 3)?;
    let label = tokens[1];
    let _ = verify_label(label)?;
    let n = parse_num(tokens[2])?;
    let cmd = Command::func(label, n);
    Ok(cmd)
}

fn parse_call(tokens: &Vec<&str>) -> Result<Command, ParseError> {
    let _ = check_tokens_num(tokens, 3)?;
    let label = tokens[1];
    let _ = verify_label(label)?;
    let n = parse_num(tokens[2])?;
    let cmd = Command::call(label, n);
    Ok(cmd)
}

fn parse_return(tokens: &Vec<&str>) -> Result<Command, ParseError> {
    let _ = check_tokens_num(tokens, 1)?;
    let cmd = Command::f_return();
    Ok(cmd)
}

fn parse_num(tok: &str) -> Result<u16, ParseError> {
    let n: u16 = tok.parse()?;
    Ok(n)
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
        let actual = parse(input).unwrap();
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

    #[test]
    fn test_parse_arith_err() {
        let input = "add local";
        let actual = parse(input);
        assert!(actual.is_err(), "redundant token");

        let input = "sub not";
        let actual = parse(input);
        assert!(actual.is_err(), "redundant token");

        let input = "eq 123";
        let actual = parse(input);
        assert!(actual.is_err(), "redundant token");

        let input = "gt 123 argument";
        let actual = parse(input);
        assert!(actual.is_err(), "redundant token");
    }

    #[test]
    fn test_parse_mem_access() {
        let input = r###"
        push argument 0
        pop argument 1
        push local 2
        pop local 3
        push static 4
        pop static 5
        push constant 6
        pop constant 7
        push this 8
        pop this 9
        push that 10
        pop that 11
        push pointer 12
        pop pointer 13
        push temp 14
        pop temp 15
        "###;
        let actual = parse(input).unwrap();
        let expect = vec![
            Command::push(Segment::Arg, 0),
            Command::pop(Segment::Arg, 1),
            Command::push(Segment::Local, 2),
            Command::pop(Segment::Local, 3),
            Command::push(Segment::Static, 4),
            Command::pop(Segment::Static, 5),
            Command::push(Segment::Constant, 6),
            Command::pop(Segment::Constant, 7),
            Command::push(Segment::This, 8),
            Command::pop(Segment::This, 9),
            Command::push(Segment::That, 10),
            Command::pop(Segment::That, 11),
            Command::push(Segment::Pointer, 12),
            Command::pop(Segment::Pointer, 13),
            Command::push(Segment::Temp, 14),
            Command::pop(Segment::Temp, 15),
        ];
        assert_eq!(actual, expect);
    }

    #[test]
    fn test_parse_mem_access_err() {
        let input = "push";
        let actual = parse(input);
        assert!(actual.is_err(), "lack token");
        let input = "pop 100";
        let actual = parse(input);
        assert!(actual.is_err(), "unexpected token");
        let input = "pop local local";
        let actual = parse(input);
        assert!(actual.is_err(), "unexpected token");
        let input = "push local 100 local";
        let actual = parse(input);
        assert!(actual.is_err(), "redundant token");
    }

    static TEST_NAME: &'static str =
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_.:0123456789";

    #[test]
    fn test_parse_label() {
        let input = format!("label {}", TEST_NAME);
        let actual = parse(&input).unwrap();
        assert_eq!(actual, vec![Command::label(TEST_NAME)]);

        let actual = parse("label 0_w");
        assert!(actual.is_err(), "label head must be not number");
    }

    #[test]
    fn test_parse_goto_and_ifgoto() {
        let input = format!("goto {}", TEST_NAME);
        let actual = parse(&input).unwrap();
        assert_eq!(actual, vec![Command::goto(TEST_NAME)]);

        let input = format!("if-goto {}", TEST_NAME);
        let actual = parse(&input).unwrap();
        assert_eq!(actual, vec![Command::ifgoto(TEST_NAME)]);
    }

    #[test]
    fn test_parse_func() {
        let input = format!("function {} 0", TEST_NAME);
        let actual = parse(&input).unwrap();
        assert_eq!(actual, vec![Command::func(TEST_NAME, 0)]);

        let input = format!("function {} -1", TEST_NAME);
        let actual = parse(&input);
        assert!(actual.is_err(), "wrong number");

        let input = format!("function {} 0", "0:");
        let actual = parse(&input);
        assert!(actual.is_err(), "label head must be not number");
    }

    #[test]
    fn test_parse_call() {
        let input = format!("call {} 0", TEST_NAME);
        let actual = parse(&input).unwrap();
        assert_eq!(actual, vec![Command::call(TEST_NAME, 0)]);

        let input = format!("call {} -1", TEST_NAME);
        let actual = parse(&input);
        assert!(actual.is_err(), "wrong number");

        let input = format!("call {} 0", "0:");
        let actual = parse(&input);
        assert!(actual.is_err(), "label head must be not number");
    }

    #[test]
    fn test_parse_return() {
        let actual = parse("return").unwrap();
        assert_eq!(actual, vec![Command::f_return()]);
    }
}
