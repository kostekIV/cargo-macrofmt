use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use toml_edit::DocumentMut;
use walkdir::WalkDir;

pub fn find_workspace_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.exists() {
            return Ok(candidate);
        }
        if !dir.pop() {
            break;
        }
    }

    bail!("No Cargo.toml found in current directory or any parent")
}

pub fn get_workspace_members(root_path: &Path) -> Result<Vec<PathBuf>> {
    let contents = fs::read_to_string(root_path).context("Failed to read root Cargo.toml")?;
    let doc = contents
        .parse::<DocumentMut>()
        .context("Invalid root TOML")?;

    let workspace_item = doc
        .get("workspace")
        .context("No [workspace] section found")?;

    let members_item = workspace_item
        .get("members")
        .context("No [workspace.members] found")?;

    let members_array = members_item
        .as_array()
        .context("[workspace.members] is not an array")?;

    let root_dir = root_path.parent().unwrap_or_else(|| Path::new("."));

    let result = members_array
        .iter()
        .filter_map(|item| item.as_str().map(|s| root_dir.join(s)))
        .collect();

    Ok(result)
}

pub fn find_rust_files(members: &[PathBuf]) -> Vec<PathBuf> {
    members
        .iter()
        .flat_map(|member| {
            WalkDir::new(member)
                .into_iter()
                .filter_entry(|e| {
                    e.file_name() != "target"
                        && !e.path().components().any(|c| c.as_os_str() == "target")
                })
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .map(|e| e.path().to_path_buf())
        })
        .collect()
}
