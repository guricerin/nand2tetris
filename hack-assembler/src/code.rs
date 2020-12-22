use crate::parser::lexer::{self, LexError};
use crate::parser::{self, command::*, common::*};

use crate::parser::ParseError;
use crate::sysmbol_table::*;

use thiserror::Error;

/// 各エラーの統合型
#[derive(Error, Debug)]
pub enum AssembleError {
    #[error("SymTableError:\n {0}")]
    SymTable(SymTableError),
    #[error("LexError:\n {0}")]
    Lex(LexError),
    #[error("ParseError:\n {0}")]
    Parse(ParseError),
}

impl From<SymTableError> for AssembleError {
    fn from(err: SymTableError) -> Self {
        Self::SymTable(err)
    }
}

impl From<LexError> for AssembleError {
    fn from(err: LexError) -> Self {
        Self::Lex(err)
    }
}

impl From<ParseError> for AssembleError {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

pub struct Assembler {
    sym_table: SymbolTable,
}

impl Assembler {
    fn new(sym_table: SymbolTable) -> Self {
        Self { sym_table }
    }

    pub fn run(input: &str) -> Result<String, AssembleError> {
        let tokens = lexer::lex(input)?;
        let commands = parser::parse(tokens)?;
        Assembler::assemble(commands)
    }

    fn assemble(commands: Vec<Command>) -> Result<String, AssembleError> {
        let mut sym_table = SymbolTable::new();
        sym_table.resolve(&commands)?;
        let asm = Self::new(sym_table);
        let mut code = String::new();

        for cmd in commands.iter() {
            match cmd {
                Annot {
                    value: CommandKind::A(cmd),
                    ..
                } => {
                    let line = asm.gen_acode(cmd);
                    code.push_str(&line);
                }
                Annot {
                    value: CommandKind::C(cmd),
                    ..
                } => {
                    let line = asm.gen_ccode(cmd);
                    code.push_str(&line);
                }
                _ => (),
            }
        }

        Ok(code)
    }

    fn gen_acode(&self, cmd: &AddrCommand) -> String {
        match cmd {
            AddrCommand {
                value: NumOrSymbol::Num(n),
            } => {
                format!("0{:015b}\n", n)
            }
            AddrCommand {
                value: NumOrSymbol::Symbol(s),
            } => {
                let addrees = self.sym_table.get_address(s).unwrap();
                format!("0{:015b}\n", addrees)
            }
        }
    }

    fn gen_ccode(&self, cmd: &CompCommand) -> String {
        match cmd {
            CompCommand {
                dest,
                comp: Annot { value: comp, .. },
                jump,
            } => {
                let comp = Assembler::comp_code(comp);
                let dest = Assembler::dest_code(dest);
                let jump = Assembler::jump_code(jump);
                format!("111{}{}{}\n", comp, dest, jump)
            }
        }
    }

    fn dest_code(dest: &Option<MemKind>) -> String {
        let code = match dest {
            None => "000",
            Some(MemKind::M) => "001",
            Some(MemKind::D) => "010",
            Some(MemKind::MD) => "011",
            Some(MemKind::A) => "100",
            Some(MemKind::AM) => "101",
            Some(MemKind::AD) => "110",
            Some(MemKind::AMD) => "111",
        };
        code.to_owned()
    }

    fn comp_code(cmd: &CompKind) -> String {
        use CompKind::*;

        match cmd {
            Constant(cons) => Assembler::constant_code(cons),
            Mem(m) => Assembler::mem_code(m),
            UniOp {
                op: Annot { value: op, .. },
                e,
            } => Assembler::uniop_code(op, e),
            BinOp {
                op: Annot { value: op, .. },
                l,
                r,
            } => Assembler::binop_code(op, l, r),
        }
    }

    fn constant_code(cons: &Constant) -> String {
        let code = match cons {
            Constant::Zero => "0101010",
            Constant::One => "0111111",
        };
        code.to_owned()
    }

    fn mem_code(mem: &MemKind) -> String {
        use MemKind::*;

        let code = match mem {
            D => "0001100",
            A => "0110000",
            M => "1110000",
            _ => unreachable!(),
        };
        code.to_owned()
    }

    fn uniop_code(op: &UniOpKind, e: &Operand) -> String {
        use MemKind::*;
        use Operand::Mem;

        let code = match (op, e) {
            // -1
            (UniOpKind::Minus, Operand::Constant(Constant::One)) => "0111010",
            // !D
            (UniOpKind::Not, Mem(D)) => "0001101",
            // -D
            (UniOpKind::Minus, Mem(D)) => "0001111",
            // !A
            (UniOpKind::Not, Mem(A)) => "0110001",
            // -A
            (UniOpKind::Minus, Mem(A)) => "0110011",
            // !M
            (UniOpKind::Not, Mem(M)) => "1110001",
            // -M
            (UniOpKind::Minus, Mem(M)) => "1110011",
            _ => unreachable!(),
        };
        code.to_owned()
    }

    fn binop_code(op: &BinOpKind, l: &MemKind, r: &Operand) -> String {
        use BinOpKind::*;
        use MemKind::*;

        let code = match (op, l, r) {
            // D+1
            (Add, D, Operand::Constant(Constant::One)) => "0011111",
            // A+1
            (Add, A, Operand::Constant(Constant::One)) => "0110111",
            // D-1
            (Sub, D, Operand::Constant(Constant::One)) => "0001110",
            // A-1
            (Sub, A, Operand::Constant(Constant::One)) => "0110010",
            // D+A
            (Add, D, Operand::Mem(A)) => "0000010",
            // D-A
            (Sub, D, Operand::Mem(A)) => "0010011",
            // A-D
            (Sub, A, Operand::Mem(D)) => "0000111",
            // D&A
            (And, D, Operand::Mem(A)) => "0000000",
            // D|A
            (Or, D, Operand::Mem(A)) => "0010101",
            // M+1
            (Add, M, Operand::Constant(Constant::One)) => "1110111",
            // M-1
            (Sub, M, Operand::Constant(Constant::One)) => "1110010",
            // D+M
            (Add, D, Operand::Mem(M)) => "1000010",
            // D-M
            (Sub, D, Operand::Mem(M)) => "1010011",
            // M-D
            (Sub, M, Operand::Mem(D)) => "1000111",
            // D&M
            (And, D, Operand::Mem(M)) => "1000000",
            // D|M
            (Or, D, Operand::Mem(M)) => "1010101",
            _ => unreachable!(),
        };
        code.to_owned()
    }

    fn jump_code(jump: &Option<JumpKind>) -> String {
        let code = match jump {
            None => "000",
            Some(JumpKind::Gt) => "001",
            Some(JumpKind::Eq) => "010",
            Some(JumpKind::Ge) => "011",
            Some(JumpKind::Lt) => "100",
            Some(JumpKind::Ne) => "101",
            Some(JumpKind::Le) => "110",
            Some(JumpKind::Jmp) => "111",
        };
        code.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asm_add() {
        let input = r###"
// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/06/add/Add.asm

// Computes R0 = 2 + 3  (R0 refers to RAM[0])

@2
D=A
@3
D=D+A
@0
M=D
        "###;

        let actual = Assembler::run(input).unwrap();
        let expect = r###"0000000000000010
1110110000010000
0000000000000011
1110000010010000
0000000000000000
1110001100001000
"###;
        assert_eq!(actual, expect);
    }

    #[test]
    fn test_asm_max() {
        let input = r###"
// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/06/max/Max.asm

// Computes R2 = max(R0, R1)  (R0,R1,R2 refer to RAM[0],RAM[1],RAM[2])

   @R0
   D=M              // D = first number
   @R1
   D=D-M            // D = first number - second number
   @OUTPUT_FIRST
   D;JGT            // if D>0 (first is greater) goto output_first
   @R1
   D=M              // D = second number
   @OUTPUT_D
   0;JMP            // goto output_d
(OUTPUT_FIRST)
   @R0
   D=M              // D = first number
(OUTPUT_D)
   @R2
   M=D              // M[2] = D (greatest number)
(INFINITE_LOOP)
   @INFINITE_LOOP
   0;JMP            // infinite loop
        "###;
        let actual = Assembler::run(input).unwrap();
        let expect = r###"0000000000000000
1111110000010000
0000000000000001
1111010011010000
0000000000001010
1110001100000001
0000000000000001
1111110000010000
0000000000001100
1110101010000111
0000000000000000
1111110000010000
0000000000000010
1110001100001000
0000000000001110
1110101010000111
"###;
        assert_eq!(actual, expect);
    }

    #[test]
    fn test_asm_rect() {
        let input = r###"
// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/06/rect/Rect.asm

// Draws a rectangle at the top-left corner of the screen.
// The rectangle is 16 pixels wide and R0 pixels high.

   @0
   D=M
   @INFINITE_LOOP
   D;JLE
   @counter
   M=D
   @SCREEN
   D=A
   @address
   M=D
(LOOP)
   @address
   A=M
   M=-1
   @address
   D=M
   @32
   D=D+A
   @address
   M=D
   @counter
   MD=M-1
   @LOOP
   D;JGT
(INFINITE_LOOP)
   @INFINITE_LOOP
   0;JMP

        "###;
        let actual = Assembler::run(input).unwrap();
        let expect = r###"0000000000000000
1111110000010000
0000000000010111
1110001100000110
0000000000010000
1110001100001000
0100000000000000
1110110000010000
0000000000010001
1110001100001000
0000000000010001
1111110000100000
1110111010001000
0000000000010001
1111110000010000
0000000000100000
1110000010010000
0000000000010001
1110001100001000
0000000000010000
1111110010011000
0000000000001010
1110001100000001
0000000000010111
1110101010000111
"###;
        assert_eq!(actual, expect);
    }
}
