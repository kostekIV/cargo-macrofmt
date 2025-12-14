use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use cargo_macrofmt::workspace;
use clap::Parser;
use similar::TextDiff;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    check: bool,

    #[arg(long, default_value = "100")]
    max_line_length: usize,

    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

fn print_diff(path: &Path, original: &str, formatted: &str) {
    TextDiff::from_lines(original, formatted)
        .unified_diff()
        .header(&path.display().to_string(), &path.display().to_string())
        .context_radius(3)
        .to_writer(io::stdout())
        .unwrap();
}

fn main() -> Result<()> {
    let args = Args::parse();

    let files = if let Some(file) = args.file {
        if !file.exists() {
            bail!("File does not exist: {}", file.display());
        }

        vec![file]
    } else {
        let root = workspace::find_workspace_root()?;
        let members = workspace::get_workspace_members(&root)?;
        workspace::find_rust_files(&members)
    };

    let mut has_changes = false;

    for path in files {
        let content = fs::read_to_string(&path)?;
        let formatted = match cargo_macrofmt::format_file(&content, args.max_line_length) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to parse {}: {e}", path.display());
                continue;
            },
        };

        if content == formatted {
            continue;
        }

        has_changes = true;

        if args.check {
            print_diff(&path, &content, &formatted);
        } else {
            fs::write(&path, formatted)?;
            println!("Formatted: {}", path.display());
        }
    }

    if args.check && has_changes {
        bail!("Some files require formatting");
    }

    Ok(())
}
