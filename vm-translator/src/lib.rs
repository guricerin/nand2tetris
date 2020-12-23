mod codegen;
mod parser;

// use codegen;
// use parser;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranslateError {
    #[error(transparent)]
    Parse(#[from] parser::ParseError),
    #[error(transparent)]
    CodeGen(#[from] codegen::CodeGenError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    // std::option::NoneErrorをfromできなかったので追加
    #[error("{0}")]
    Etc(String),
}

impl TranslateError {
    pub fn etc(e: &str) -> Self {
        Self::Etc(e.to_owned())
    }
}

pub struct Translator {
    vm_files: Vec<PathBuf>,
}

impl Translator {
    // fn new(vm_files: Vec<PathBuf>) -> Self {
    //     Self { vm_files: vm_files }
    // }

    pub fn run(vm_paths: &Vec<PathBuf>, out_path: &Path) -> Result<(), TranslateError> {
        // let trans = Self::new(vm_files);
        if vm_paths.is_empty() {
            return Err(TranslateError::etc("not found .vm file"));
        }

        let mut writer = BufWriter::new(File::create(out_path)?);
        writer.write(codegen::CodeGenerator::init_code().as_bytes())?;
        for vm_path in vm_paths.iter() {
            let vm_code = fs::read_to_string(vm_path)?;
            let cmds = parser::parse(&vm_code)?;
            let vm_filename = vm_path
                .file_stem()
                .ok_or(TranslateError::etc("failed to get path leaf"))?;
            let vm_filename = vm_filename
                .to_str()
                .ok_or(TranslateError::etc("failed to convert OsStr to &str"))?;
            let asm_code = codegen::CodeGenerator::run(vm_filename, &cmds)?;
            writer.write(asm_code.as_bytes())?;
        }
        Ok(())
    }
}
