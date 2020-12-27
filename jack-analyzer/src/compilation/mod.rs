mod xml_token;

use crate::lex;
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
            // let run = || -> Result<(), CompileError> {
            //     let jack_code = fs::read_to_string(jack_file)?;
            //     let tokens = lex::Lexer::new().run(&jack_code)?;
            //     let xml = xml_token::translate(&tokens);
            //     let _ = self.write_file(jack_file, xml)?;
            //     Ok(())
            // };

            // match run() {
            //     Ok(()) => (),
            //     Err(e) => {
            //         eprintln!("FILE: {:?}", &jack_file);
            //         eprintln!("{}", e);
            //     }
            // }

            let jack_code = fs::read_to_string(jack_file)?;
            let tokens = lex::Lexer::new().run(&jack_code)?;
            let xml = xml_token::translate(&tokens);
            let _ = self.write_file(jack_file, xml)?;
        }
        Ok(())
    }

    fn write_file(&self, jack_file: &PathBuf, code: String) -> Result<(), CompileError> {
        let out_path = self.replace_path_to_output(jack_file, "xml");
        let mut writer = BufWriter::new(File::create(out_path)?);
        writer.write(code.as_bytes())?;
        Ok(())
    }

    fn replace_path_to_output(&self, path: &PathBuf, ext: &str) -> PathBuf {
        let name = path.file_name().unwrap();
        self.output_dir.join(name).with_extension(ext)
    }
}
