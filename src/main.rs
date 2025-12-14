use std::{fs, path::PathBuf};

use anyhow::{Result, bail};
use cargo_macrofmt::{
    CONFIG_FILENAME, Config, IndentChar, ResolvedConfig, find_rust_files, find_workspace_root,
    get_crate_directories, print_diff,
};
use clap::Parser;
use proc_macro2 as _;

const TARGET_DIR: &str = "target";

#[derive(Parser)]
struct Args {
    #[arg(long, help = "Check if files need formatting without modifying them")]
    check: bool,

    #[arg(long, help = "Maximum line length before formatting macro attributes")]
    max_line_length: Option<usize>,

    #[arg(long, help = "Number of spaces/tabs per indentation level")]
    indent_width: Option<usize>,

    #[arg(
        long,
        value_enum,
        help = "Character to use for indentation (space or tab)"
    )]
    indent_char: Option<IndentChar>,

    #[arg(
        short,
        long,
        value_name = "MACRO",
        help = "Macro names to format (matches last segment of macro path)"
    )]
    macros_to_format: Vec<String>,

    #[arg(
        short,
        long,
        value_name = "DIR",
        help = "Directory names to ignore during file discovery"
    )]
    ignore_dirs: Vec<String>,

    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Specific file or directory to format (defaults to workspace root)"
    )]
    file: Option<PathBuf>,
}

fn resolve_config(args: Args, config: Option<Config>) -> Result<ResolvedConfig> {
    let max_line_length = args
        .max_line_length
        .or_else(|| config.as_ref().and_then(|c| c.max_line_length))
        .unwrap_or(80);

    let indent_width = args
        .indent_width
        .or_else(|| config.as_ref().and_then(|c| c.indent_width))
        .unwrap_or(4);

    let indent_char = args
        .indent_char
        .or_else(|| config.as_ref().and_then(|c| c.indent_char))
        .unwrap_or(IndentChar::Space);

    let macros_to_format = match !args.macros_to_format.is_empty() {
        true => args.macros_to_format,
        false => config
            .as_ref()
            .and_then(|c| c.macros_to_format.clone())
            .unwrap_or_default(),
    };

    let mut ignore_dirs = match !args.ignore_dirs.is_empty() {
        true => args.ignore_dirs,
        false => config
            .as_ref()
            .and_then(|c| c.ignore_dirs.clone())
            .unwrap_or_default(),
    };

    if ignore_dirs.is_empty() {
        ignore_dirs.push(TARGET_DIR.to_owned());
    }

    if macros_to_format.is_empty() {
        bail!(
            "No macros specified to format. Use -m or set macros_to_format in ${CONFIG_FILENAME}"
        );
    }

    Ok(ResolvedConfig {
        max_line_length,
        indent_width,
        indent_char,
        macros_to_format,
        ignore_dirs,
    })
}

fn resolve_files(file: Option<&PathBuf>, ignore_dirs: &[String]) -> Result<Vec<PathBuf>> {
    let Some(file) = file else {
        let root = find_workspace_root()?;
        let members = get_crate_directories(&root)?;

        return Ok(find_rust_files(&members, ignore_dirs));
    };

    if !file.exists() {
        bail!("Path does not exist: {}", file.display());
    }

    if file.is_dir() {
        let root = file.join("Cargo.toml");
        if !root.exists() {
            bail!("Directory does not contain Cargo.toml: {}", file.display());
        }
        let members = get_crate_directories(&root)?;
        return Ok(find_rust_files(&members, ignore_dirs));
    }

    if file.extension().is_none_or(|ext| ext != "rs") {
        bail!("Not a Rust file: {}", file.display());
    }

    Ok(vec![file.clone()])
}

fn main() -> Result<()> {
    let args = Args::parse();
    let check = args.check;
    let file = args.file.clone();

    let config = Config::find_config_file().and_then(|path| Config::from_file(&path).ok());
    let resolved = resolve_config(args, config)?;

    let files = resolve_files(file.as_ref(), &resolved.ignore_dirs)?;

    let mut has_changes = false;

    for path in files {
        let content = fs::read_to_string(&path)?;
        let formatted = match cargo_macrofmt::format_file(&content, &resolved) {
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

        if check {
            print_diff(
                &content,
                &formatted,
                &path.to_string_lossy(),
                &path.to_string_lossy(),
            )?;
        } else {
            fs::write(&path, formatted)?;
            println!("Formatted: {}", path.display());
        }
    }

    if check && has_changes {
        bail!("Some files require formatting");
    }

    Ok(())
}
