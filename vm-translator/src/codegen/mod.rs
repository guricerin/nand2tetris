mod idiom;
mod vm_bool;

use crate::parser::{arithmetic::*, flow::*, func::*, mem_access::*, segment::*, *};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("uninitialize file name")]
    UninitializeFileName,
    #[error("not direct segment (argument, local, this, or that only")]
    NotDirectSegment,
    #[error("not indirect segment (pointer, temp, static only")]
    NotIndirectSegment,
}

pub struct CodeGenerator {
    /// without ext
    filename: Option<String>,
    label_id: u64,
    /// 関数呼び出し履歴 末尾は現在翻訳中の関数名
    func_name: String,
}

/// 擬似的なトップレベル関数
static TOP_LEVEL_FUNC_LABEL: &'static str = "::__TOP_LEVEL__::";

impl CodeGenerator {
    pub fn init_code() -> String {
        format!(
            r#"
{0}
{1}
        "#,
            idiom::INITIALIZE,
            idiom::call(TOP_LEVEL_FUNC_LABEL, "Sys.init", 0, 0)
        )
    }

    pub fn new() -> Self {
        Self {
            filename: None,
            label_id: 0,
            func_name: TOP_LEVEL_FUNC_LABEL.to_owned(),
        }
    }

    pub fn run(&mut self, filename: &str, cmds: &Vec<Command>) -> Result<String, CodeGenError> {
        self.filename = Some(filename.to_owned());
        self.label_id = 0;
        self.func_name = TOP_LEVEL_FUNC_LABEL.to_owned();
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
        let true_label = format!("_COND_TRUE_{}_{}_", &filename, self.label_id);
        // 実はfalse_labelは不要だが、見やすくするために挿入
        let false_label = format!("_COND_FALSE_{}_{}_", &filename, self.label_id);
        let break_label = format!("_IF_BLOCK_BREAK_{}_{}_", &filename, self.label_id);
        self.label_id += 1;
        let code = idiom::jump(jmp, &true_label, &false_label, &break_label);
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
        // 全場合においてDレジスタにデータを入れてからD経由でRAM[@SP]にプッシュする
        let code = match segment {
            // constantはRAMに割り当てられていないので、indexを単なる定数値として扱う
            Constant => {
                format!(
                    r#"// push constant n
@{0}
D=A
{1}
"#,
                    index,
                    idiom::push_from_d()
                )
            }
            Arg | Local | This | That => {
                let name = segment.name().ok_or(CodeGenError::NotIndirectSegment)?;
                idiom::push_from_direct_segment(&name, index)
            }
            Pointer | Temp => {
                let ram_index = segment.ram_index().ok_or(CodeGenError::NotDirectSegment)?;
                let name = format!("R{}", ram_index + index);
                idiom::push_from_indirect_segment(&name)
            }
            // 現在翻訳中のjackファイルのスタティック変数
            Static => {
                let filename = self.get_filename()?;
                let name = format!("{}.{}", filename, index);
                idiom::push_from_indirect_segment(&name)
            }
        };
        Ok(code)
    }

    /// スタックからポップしたデータをsegment[index]に格納
    /// index: 0-index
    fn pop(&self, segment: &Segment, index: u16) -> Result<String, CodeGenError> {
        use segment::Segment::*;

        let code = match segment {
            Arg | Local | This | That => {
                let name = segment.name().ok_or(CodeGenError::NotIndirectSegment)?;
                idiom::pop_to_direct_segment(&name, index)
            }
            Pointer | Temp => {
                let ram_index = segment.ram_index().ok_or(CodeGenError::NotDirectSegment)?;
                let name = format!("R{}", ram_index + index);
                idiom::pop_to_indirect_segment(&name)
            }
            Static => {
                let filename = self.get_filename()?;
                let name = format!("{}.{}", filename, index);
                idiom::pop_to_indirect_segment(&name)
            }
            Constant => unimplemented!(),
        };
        Ok(code)
    }

    fn flow(&mut self, cmd: &Flow) -> Result<String, CodeGenError> {
        use flow::Flow::*;

        let code = match cmd {
            // スコープはそれが定義された関数内
            Label(label) => idiom::label(&self.func_name, label),
            // 無条件移動 labelの位置に移動
            // 移動先は同じ関数内に限られる
            Goto(label) => idiom::goto(&self.func_name, label),
            // スタックをポップ、値が0以外なら移動
            // 移動先は同じ関数内に限られる
            IfGoto(label) => idiom::ifgoto(&self.func_name, label),
        };
        Ok(code)
    }

    fn func(&mut self, cmd: &Func) -> Result<String, CodeGenError> {
        use func::Func::*;

        let code = match cmd {
            // paramc個のローカル変数をもつnameという名前の関数を定義する
            Func { name, paramc } => {
                self.func_name = name.to_owned();
                idiom::func(name, *paramc)
            }
            // 呼び出し元の状態を復元し、呼び出し元にリターン、
            Return => idiom::f_return(),
            // 関数呼び出し
            Call { name, argc } => {
                let callee = name;
                self.label_id += 1;
                idiom::call(&self.func_name, callee, *argc, self.label_id)
            }
        };
        Ok(code)
    }
}
