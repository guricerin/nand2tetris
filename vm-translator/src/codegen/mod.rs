use crate::parser::*;
use arithmetic::*;
use flow::*;
use func::*;
use mem_access::*;
use segment::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("todo")]
    Hoge,
}

pub struct CodeGenerator {
    /// without ext
    filename: String,
}

impl CodeGenerator {
    pub fn init_code() -> String {
        // SP: RAM[0]に格納。値はRAM[256..2047]へのpointer
        let code = r#"// initialize
@256
D=A
@SP
M=D
"#;
        code.to_owned()
    }

    fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }

    pub fn run(filename: &str, cmds: &Vec<Command>) -> Result<String, CodeGenError> {
        let generator = Self::new(filename);
        generator.generate(cmds)
    }

    fn generate(&self, cmds: &Vec<Command>) -> Result<String, CodeGenError> {
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

    fn arithmetic(&self, cmd: &Arithmetic) -> Result<String, CodeGenError> {
        use arithmetic::Arithmetic::*;
        // todo: fix
        let code = match cmd {
            // M+D はない
            Add => self.binary_op("M=D+M"),
            // Sub => "M-D\n",
            // Neg => "-M\n",
            // Eq => "JEQ\n",
            // Gt => "JGT\n",
            // Lt => "JLT\n",
            // // M&D はない
            // And => "D&M\n",
            // // M|D はない
            // Or => "D|M\n",
            // Not => "!M\n",
            _ => todo!(),
        };
        Ok(code.to_string())
    }

    fn binary_op(&self, op: &str) -> String {
        // スタックからrightをポップ
        // スタックの頂点(left)を left op right に書き換える
        // A=A-1 は、RAM[@SP]を-1するわけではない
        format!(
            r#"// binary op
@SP
AM=M-1
D=M
A=A-1
{}
        "#,
            op
        )
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
                format!(
                    r#"// static <n>
@{}.{}
D=M
"#,
                    self.filename, index
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
        // スタックにプッシュ
        // @SPの参照先にDを入れた後、@SP自体をインクリメント
        let code1 = r#"// push
@SP // *SP=D
A=M
M=D
@SP // SP++
M=M+1
"#;
        let code = format!("{}{}", code0, code1);
        Ok(code)
    }

    /// スタックからポップしたデータをsegment[index]に格納
    fn pop(&self, segment: &Segment, index: u16) -> Result<String, CodeGenError> {
        use segment::Segment::*;
        todo!();
    }

    fn flow(&self, cmd: &Flow) -> Result<String, CodeGenError> {
        todo!();
    }

    fn func(&self, cmd: &Func) -> Result<String, CodeGenError> {
        todo!();
    }
}
