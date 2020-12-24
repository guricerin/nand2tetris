mod idiom;
mod vm_bool;
use vm_bool::*;

use crate::parser::*;
use arithmetic::*;
use flow::*;
use func::*;
use mem_access::*;
use segment::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("uninitialize file name")]
    UninitializeFileName,
}

pub struct CodeGenerator {
    /// without ext
    filename: Option<String>,
    if_label_id: usize,
}

impl CodeGenerator {
    pub fn init_code() -> String {
        idiom::INITIALIZE.to_string()
    }

    pub fn new() -> Self {
        Self {
            filename: None,
            if_label_id: 0,
        }
    }

    pub fn run(&mut self, filename: &str, cmds: &Vec<Command>) -> Result<String, CodeGenError> {
        self.filename = Some(filename.to_owned());
        self.if_label_id = 0;
        self.generate(cmds)
    }

    fn get_filename(&self) -> Result<String, CodeGenError> {
        let filename = self
            .filename
            .clone()
            .ok_or(CodeGenError::UninitializeFileName)?;
        Ok(filename)
    }

    fn generate(&mut self, cmds: &Vec<Command>) -> Result<String, CodeGenError> {
        let mut buf = String::new();
        for cmd in cmds.iter() {
            match cmd {
                Command::Arithmetic(cmd) => {
                    let code = self.arithmetic(cmd)?;
                    buf.push_str(&code);
                }
                Command::MemAccess(cmd) => {
                    let code = self.mem_access(cmd)?;
                    buf.push_str(&code);
                }
                Command::Flow(cmd) => {
                    let code = self.flow(cmd)?;
                    buf.push_str(&code);
                }
                Command::Func(cmd) => {
                    let code = self.func(cmd)?;
                    buf.push_str(&code);
                }
            }
        }
        Ok(buf)
    }

    fn arithmetic(&mut self, cmd: &Arithmetic) -> Result<String, CodeGenError> {
        use arithmetic::Arithmetic::*;
        let code = match cmd {
            // x + y
            // M+D はない
            Add => self.binary_op("M=D+M"),
            // x - y
            Sub => self.binary_op("M=M-D"),
            // -x
            Neg => self.unary_op("-"),
            // x == y
            Eq => self.jump("JEQ")?,
            // x > y
            Gt => self.jump("JGT")?,
            // x < y
            Lt => self.jump("JLT")?,
            // x & y
            // M&D はない
            And => self.binary_op("M=D&M"),
            // x or y
            // M|D はない
            Or => self.binary_op("M=D|M"),
            // !x
            Not => self.unary_op("!"),
        };
        Ok(code)
    }

    fn binary_op(&self, expr: &str) -> String {
        // スタックからrightをポップ
        // スタックの頂点(left)を left op right に書き換える
        // A=A-1 は、RAM[@SP]を-1するわけではない
        format!(
            r#"// binary op
@SP
AM=M-1 // SP--
D=M    // D = *SP
A=A-1  // ptr = SP - 1
{}     // *(ptr) = *(ptr) op D
"#,
            expr
        )
    }

    fn unary_op(&self, op: &str) -> String {
        format!(
            r#"// unary op
@SP     // ptr = SP
A=M-1   // ptr = ptr - 1
M={}M   // *(ptr) = op *(ptr)
"#,
            op
        )
    }

    fn jump(&mut self, jmp: &str) -> Result<String, CodeGenError> {
        let filename = self.get_filename()?;
        let true_label = format!("_COND_TRUE_{}_{}_", &filename, self.if_label_id);
        // 実はfalse_labelは不要だが、見やすくするために挿入
        let false_label = format!("_COND_FALSE_{}_{}_", &filename, self.if_label_id);
        let break_label = format!("_IF_BLOCK_BREAK_{}_{}_", &filename, self.if_label_id);
        self.if_label_id += 1;
        let code = format!(
            r#"
{0}
A=A-1 // ptr = ptr - 1
D=M-D // lhs - rhs
@{1}
D;{6} // compare D to 0: true -> jump to TRUE false -> jump to FALSE
({2})
@SP
A=M
M={3} // *SP = false
@{5}
0;JMP // jump to BREAK
({1})
@SP
A=M
M={4} // *SP = true
({5}) // BREAK
"#,
            idiom::pop_to_d(),
            true_label,
            false_label,
            VmBool::False as i32,
            VmBool::True as i32,
            break_label,
            jmp
        );
        Ok(code)
    }

    fn mem_access(&self, cmd: &MemAccess) -> Result<String, CodeGenError> {
        use MemAccess::*;
        match cmd {
            Push(segment, index) => self.push(segment, *index),
            Pop(segment, index) => self.pop(segment, *index),
        }
    }

    /// segment[index]をスタックにプッシュ
    fn push(&self, segment: &Segment, index: u16) -> Result<String, CodeGenError> {
        use segment::Segment::*;
        // 全場合においてDレジスタにデータを入れてからD経由でRAM[@SP]にプッシュするつもりだが、どうなることやら
        let code0 = match segment {
            Arg => todo!(),
            Local => todo!(),
            // 現在翻訳中のjackファイルのスタティック変数
            Static => {
                let filename = self.get_filename()?;
                format!(
                    r#"// static <n>
@{}.{}
D=M
"#,
                    filename, index
                )
            }
            // constantはRAMに割り当てられていないので、indexを単なる定数値として扱う
            Constant => {
                format!(
                    r#"// constant <n>
@{}
D=A
"#,
                    index
                )
            }
            This => todo!(),
            That => todo!(),
            Pointer => todo!(),
            Temp => todo!(),
        };
        let code1 = idiom::push_from_d();
        let code = format!("{}{}", code0, code1);
        Ok(code)
    }

    /// スタックからポップしたデータをsegment[index]に格納
    fn pop(&self, segment: &Segment, index: u16) -> Result<String, CodeGenError> {
        todo!();
        use segment::Segment::*;

        let code0 = idiom::pop_to_d();
        let code1 = match segment {
            Arg => todo!(),
            Local => todo!(),
            Static => todo!(),
            Constant => todo!(),
            This => todo!(),
            That => todo!(),
            Pointer => todo!(),
            Temp => todo!(),
        };
        let code = format!("{}{}", code0, "");
        Ok(code)
    }

    fn flow(&self, cmd: &Flow) -> Result<String, CodeGenError> {
        todo!();
    }

    fn func(&self, cmd: &Func) -> Result<String, CodeGenError> {
        todo!();
    }
}
