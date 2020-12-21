use anyhow::{Context, Result};
use clap::Clap;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Clap, Debug)]
#[clap(name = env!("CARGO_BIN_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    #[clap(name = ".asm FILE")]
    asm_path: PathBuf,
}

/// 拡張子が.asmなら、.hackに置換した文字列を返す
fn ensure_asm_file(path: &Path) -> Result<OsString> {
    path.extension()
        .and_then(OsStr::to_str)
        .and_then(|ext| match ext {
            "asm" => {
                let stem = path.file_stem()?;
                let mut filename = stem.to_os_string();
                filename.push(".hack");
                Some(filename)
            }
            _ => None,
        })
        .context("is not .asm file!")
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    let asm_path = opts.asm_path;
    let code = fs::read_to_string(&asm_path)?;

    let hack_path = ensure_asm_file(&asm_path)?;
    // let mut file = File::create(hack_path)?;
    // file.write_all(code.as_bytes())?;
    // file.flush()?;

    // 出力先のファイルは上書き
    let mut writer = BufWriter::new(File::create(&hack_path)?);
    writer.write_all(&code.as_bytes())?;

    let hack_path: PathBuf = hack_path.into();
    println!("Success: assembled {:?} to {:?} .", &asm_path, &hack_path);
    Ok(())
}
