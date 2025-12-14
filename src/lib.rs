use anyhow::Result;
use proc_macro2 as _;
use syn::{File, parse_file, visit::Visit};

use crate::visitor::MacroVisitor;

mod formatting;
mod visitor;
pub mod workspace;

pub fn format_file(
    content: &str,
    max_line_length: usize,
    macros_to_format: &[String],
) -> Result<String> {
    let file: File = parse_file(content)?;

    let mut visitor = MacroVisitor::new(content, max_line_length, macros_to_format);
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
