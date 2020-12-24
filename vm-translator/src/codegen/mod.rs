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
    #[error("unnamed segment")]
    UnnamedSegment,
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
            Add => idiom::binary_op("D+M"),
            // x - y
            Sub => idiom::binary_op("M-D"),
            // -x
            Neg => idiom::unary_op("-"),
            // x == y
            Eq => self.jump("JEQ")?,
            // x > y
            Gt => self.jump("JGT")?,
            // x < y
            Lt => self.jump("JLT")?,
            // x & y
            // M&D はない
            And => idiom::binary_op("D&M"),
            // x or y
            // M|D はない
            Or => idiom::binary_op("D|M"),
            // !x
            Not => idiom::unary_op("!"),
        };
        Ok(code)
    }

    fn jump(&mut self, jmp: &str) -> Result<String, CodeGenError> {
        let filename = self.get_filename()?;
        let true_label = format!("_COND_TRUE_{}_{}_", &filename, self.if_label_id);
        // 実はfalse_labelは不要だが、見やすくするために挿入
        let false_label = format!("_COND_FALSE_{}_{}_", &filename, self.if_label_id);
        let break_label = format!("_IF_BLOCK_BREAK_{}_{}_", &filename, self.if_label_id);
        self.if_label_id += 1;
        let code = format!(
            r#"// jump
{0}
A=A-1 // ptr--
D=M-D // lhs - rhs
@{1}
D;{6} // compare D to 0: true -> jump to TRUE false -> jump to FALSE
({2}) // FALSE block
D={3} // D = false
@{5}
0;JMP // jump to BREAK
({1}) // TRUE block
D={4} // D = true
({5}) // BREAK
@SP
M=M-1 // SP-- (lhsが格納されていたアドレスに条件式の結果を突っ込むため)
{7}
"#,
            idiom::pop_to_d(),
            true_label,
            false_label,
            VmBool::False as i32,
            VmBool::True as i32,
            break_label,
            jmp,
            idiom::push_from_d()
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
    /// index: 0-index
    fn push(&self, segment: &Segment, index: u16) -> Result<String, CodeGenError> {
        use segment::Segment::*;
        // 全場合においてDレジスタにデータを入れてからD経由でRAM[@SP]にプッシュするつもりだが、どうなることやら
        let code = match segment {
            Arg | Local | This | That => {
                let name = segment.name().ok_or(CodeGenError::UnnamedSegment)?;
                idiom::push_from_named_segment(&name, index)
            }
            // 現在翻訳中のjackファイルのスタティック変数
            Static => {
                let filename = self.get_filename()?;
                format!(
                    r#"// push static <n>
@{0}.{1}
D=M
{2}
"#,
                    filename,
                    index,
                    idiom::push_from_d()
                )
            }
            // constantはRAMに割り当てられていないので、indexを単なる定数値として扱う
            Constant => {
                format!(
                    r#"// push constant <n>
@{0}
D=A
{1}
"#,
                    index,
                    idiom::push_from_d()
                )
            }
            Pointer | Temp => idiom::push_from_unnamed_segment(segment, index),
        };
        Ok(code)
    }

    /// スタックからポップしたデータをsegment[index]に格納
    /// index: 0-index
    fn pop(&self, segment: &Segment, index: u16) -> Result<String, CodeGenError> {
        use segment::Segment::*;

        let code = match segment {
            Arg | Local | This | That => {
                let name = segment.name().ok_or(CodeGenError::UnnamedSegment)?;
                idiom::pop_to_named_segment(&name, index)
            }
            Static => todo!(),
            Constant => todo!(),
            Pointer | Temp => idiom::pop_to_unnamed_segment(segment, index),
        };
        Ok(code)
    }

    fn flow(&self, cmd: &Flow) -> Result<String, CodeGenError> {
        todo!();
    }

    fn func(&self, cmd: &Func) -> Result<String, CodeGenError> {
        todo!();
    }
}
