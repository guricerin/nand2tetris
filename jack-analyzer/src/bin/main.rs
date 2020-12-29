use anyhow::{anyhow, Result};
use clap::Clap;
use jack_analyzer::compilation;
use std::fs;
use std::path::PathBuf;

#[derive(Clap, Debug)]
#[clap(name = env!("CARGO_BIN_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    #[clap(name = ".jack FILE or DIR")]
    jack_path: PathBuf,
    #[clap(short = 'o', name = "OUTPUT DIR")]
    output_dir: PathBuf,
    #[clap(subcommand)]
    mode: Mode,
}

#[derive(Clap, Debug)]
enum Mode {
    Txml, // token to xml
    Axml, // ast to xml
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    // 絶対パス取得
    let jack_path = fs::canonicalize(opts.jack_path)?;
    let jack_files = if jack_path.is_dir() {
        let children = fs::read_dir(&jack_path)?;
        children
            .map(|path| path.unwrap().path())
            .filter(|path| path.is_file())
            .filter(|path| path.extension().unwrap() == "jack")
            .collect::<Vec<_>>()
    } else {
        if jack_path.extension().unwrap() == "jack" {
            vec![jack_path]
        } else {
            return Err(anyhow!("{:?} is not .jack file", jack_path));
        }
    };

    let output_dir = fs::canonicalize(opts.output_dir)?;
    if output_dir.is_file() {
        return Err(anyhow!("{:?} is not directory", output_dir));
    }

    let engine = compilation::Engine::new(jack_files, output_dir);
    match opts.mode {
        Mode::Txml => {
            engine.lex_to_xml()?;
        }
        Mode::Axml => {
            engine.parse_to_xml()?;
        }
    };

    Ok(())
}
