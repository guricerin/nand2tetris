use anyhow::{anyhow, Context, Result};
use clap::Clap;
use hack_assembler::*;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Clap, Debug)]
#[clap(name = env!("CARGO_BIN_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    #[clap(name = ".asm FILE")]
    asm_path: PathBuf,
}

fn ensure_asm_file(path: &Path) -> Result<()> {
    let ext = path
        .extension()
        .with_context(|| format!("failed to get file extention\nfile path: {:?}", path))?;
    if ext == "asm" {
        Ok(())
    } else {
        Err(anyhow!("{:?} is not .asm file", path))
    }
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    let asm_path = opts.asm_path;
    let code = fs::read_to_string(&asm_path)?;

    let _ = ensure_asm_file(&asm_path)?;
    // 拡張子を置換
    let hack_path = asm_path.with_extension("hack");

    // 出力先のファイルは上書き
    let mut writer = BufWriter::new(File::create(&hack_path)?);
    let code = Assembler::run(&code)?;
    writer.write_all(&code.as_bytes())?;

    let hack_path: PathBuf = hack_path.into();
    println!("Success: assembled {:?} to {:?}", &asm_path, &hack_path);
    Ok(())
}
