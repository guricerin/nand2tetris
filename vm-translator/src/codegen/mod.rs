use crate::parser::*;
use arithmetic::*;
use flow::*;
use func::*;
use mem_access::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeGenError {
    #[error("todo")]
    Hoge,
}

pub struct CodeGenerator {
    filename: String,
}

impl CodeGenerator {
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
            Add => "M=D+M\n",
            Sub => "M-D\n",
            Neg => "-M\n",
            Eq => "JEQ\n",
            Gt => "JGT\n",
            Lt => "JLT\n",
            // M&D はない
            And => "D&M\n",
            // M|D はない
            Or => "D|M\n",
            Not => "!M\n",
        };
        Ok(code.to_string())
    }

    fn mem_access(&self, cmd: &MemAccess) -> Result<String, CodeGenError> {
        todo!();
    }

    fn flow(&self, cmd: &Flow) -> Result<String, CodeGenError> {
        todo!();
    }

    fn func(&self, cmd: &Func) -> Result<String, CodeGenError> {
        todo!();
    }
}
