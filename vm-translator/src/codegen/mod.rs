use crate::parser::*;
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

    pub fn run(filename: &str, cmds: Vec<Command>) -> Result<String, CodeGenError> {
        let generator = Self::new(filename);
        generator.generate(cmds)
    }

    fn generate(&self, cmds: Vec<Command>) -> Result<String, CodeGenError> {
        todo!();
    }
}
