use anyhow::Result;
use similar::TextDiff;
use syn::{File, parse_file, visit::Visit};

mod config;
mod formatting;
mod visitor;
mod workspace;

pub use config::{Config, IndentChar, ResolvedConfig};
pub use formatting::{format_macro_attr, reindent_nested_content};
pub use visitor::MacroVisitor;
pub use workspace::{find_rust_files, find_workspace_root, get_crate_directories};

pub const CONFIG_FILENAME: &str = "macrofmt.toml";

pub fn format_file(content: &str, config: &ResolvedConfig) -> Result<String> {
    let file: File = parse_file(content)?;

    let mut visitor = MacroVisitor::new(content, config);
    visitor.visit_file(&file);

    if visitor.replacements.is_empty() {
        return Ok(content.to_string());
    }

    let mut replacements = visitor.replacements;
    replacements.sort_by_key(|(start, ..)| *start);

    let mut result = String::new();
    let mut last_pos = 0;

    for (start, end, replacement) in replacements {
        result.push_str(&content[last_pos..start]);
        result.push_str(&replacement);
        last_pos = end;
    }
    result.push_str(&content[last_pos..]);

    Ok(result)
}

pub fn print_diff(expected: &str, result: &str, header_a: &str, header_b: &str) -> Result<()> {
    TextDiff::from_lines(expected, result)
        .unified_diff()
        .header(header_a, header_b)
        .context_radius(3)
        .to_writer(std::io::stderr())?;

    Ok(())
}
