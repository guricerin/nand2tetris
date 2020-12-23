use anyhow::{anyhow, Context, Result};
use clap::Clap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Clap, Debug)]
#[clap(name = env!("CARGO_BIN_NAME"), version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    #[clap(name = ".vm file or dir PATH")]
    vm_path: PathBuf,
}

fn ensure_vm_file(path: &Path) -> Result<()> {
    let ext = path
        .extension()
        .with_context(|| format!("failed to get file extention\nfile path: {:?}", path))?;
    if ext == "vm" {
        Ok(())
    } else {
        Err(anyhow!("{:?} is not .vm file", path))
    }
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    // 絶対パス取得
    let vm_path = fs::canonicalize(opts.vm_path)?;
    // .vmファイル一覧と出力ファイル
    let (vm_paths, asm_path) = if vm_path.is_dir() {
        let dirname = &vm_path
            .file_name()
            .with_context(|| "failed to get leaf dir")?;
        let vms = fs::read_dir(&vm_path)?;
        let vms = vms
            .flat_map(|path| {
                let path = path.unwrap().path();
                if let Ok(()) = ensure_vm_file(&path) {
                    Ok(path)
                } else {
                    Err(())
                }
            })
            .collect::<Vec<PathBuf>>();
        (vms, vm_path.join(dirname).with_extension("asm"))
    } else {
        let _ = ensure_vm_file(&vm_path)?;
        (vec![vm_path.clone()], vm_path.with_extension("asm"))
    };

    println!("{:?}", vm_paths);
    println!("{:?}", asm_path);
    // File::create(asm_path)?;
    Ok(())
}
