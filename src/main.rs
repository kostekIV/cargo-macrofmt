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

    #[arg(short, long, value_name = "MACRO", required = true, num_args = 1..)]
    macros_to_format: Vec<String>,

    #[arg(short, long, value_name = "DIR", default_values = ["target"])]
    ignore_dirs: Vec<String>,

    #[arg(short, long, value_name = "FILE")]
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

    let files = if let Some(path) = args.file {
        if !path.exists() {
            bail!("Path does not exist: {}", path.display());
        }

        if path.is_dir() {
            let root = path.join("Cargo.toml");
            if !root.exists() {
                bail!("Directory does not contain Cargo.toml: {}", path.display());
            }
            let members = workspace::get_crate_directories(&root)?;
            workspace::find_rust_files(&members, &args.ignore_dirs)
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            vec![path]
        } else {
            bail!("Not a Rust file: {}", path.display());
        }
    } else {
        let root = workspace::find_workspace_root()?;
        let members = workspace::get_crate_directories(&root)?;
        workspace::find_rust_files(&members, &args.ignore_dirs)
    };

    let mut has_changes = false;

    for path in files {
        let content = fs::read_to_string(&path)?;
        let formatted = match cargo_macrofmt::format_file(
            &content,
            args.max_line_length,
            &args.macros_to_format,
        ) {
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
