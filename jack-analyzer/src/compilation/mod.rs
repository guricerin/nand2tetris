mod xml_token;

use crate::lex;
use crate::parse;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Lex(#[from] lex::LexError),
    #[error(transparent)]
    Parse(#[from] parse::ParseError),
}

enum Mode {
    Lex,
    Parse,
}

pub struct Engine {
    jack_files: Vec<PathBuf>,
    output_dir: PathBuf,
}

impl Engine {
    pub fn new(jack_files: Vec<PathBuf>, output_dir: PathBuf) -> Self {
        Self {
            jack_files,
            output_dir,
        }
    }

    pub fn lex_to_xml(&self) -> Result<(), CompileError> {
        for jack_file in self.jack_files.iter() {
            let jack_code = fs::read_to_string(jack_file)?;
            let tokens = lex::Lexer::new(jack_file).run(&jack_code)?;
            let xml = xml_token::translate(&tokens);
            let _ = self.write_file(jack_file, xml, Mode::Lex)?;
        }
        Ok(())
    }

    pub fn parse_to_xml(&self) -> Result<(), CompileError> {
        for jack_file in self.jack_files.iter() {
            let jack_code = fs::read_to_string(jack_file)?;
            let tokens = lex::Lexer::new(jack_file).run(&jack_code)?;
            let ast = parse::Parser::new(jack_file).run(tokens)?;
            todo!();
        }
        todo!();
    }

    fn write_file(
        &self,
        jack_file: &PathBuf,
        code: String,
        mode: Mode,
    ) -> Result<(), CompileError> {
        let out_path = self.replace_path_to_output(jack_file, "xml", mode);
        let mut writer = BufWriter::new(File::create(out_path)?);
        writer.write(code.as_bytes())?;
        Ok(())
    }

    fn replace_path_to_output(&self, path: &PathBuf, ext: &str, mode: Mode) -> PathBuf {
        let name = path.file_name().unwrap().to_str().unwrap();
        match mode {
            Mode::Lex => {
                let name = name.replace(".jack", "T");
                self.output_dir.join(name).with_extension(ext)
            }
            Mode::Parse => {
                let name = name.replace(".jack", "");
                self.output_dir.join(name).with_extension(ext)
            }
        }
    }
}
