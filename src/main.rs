use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use proc_macro2 as _;
use similar::TextDiff;
use syn::{Attribute, File, Meta, parse_file, spanned::Spanned, visit::Visit};
use toml_edit::DocumentMut;
use walkdir::WalkDir;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    check: bool,

    #[arg(long, default_value = "100")]
    max_line_length: usize,
}

fn find_workspace_root() -> Result<PathBuf> {
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

fn get_workspace_members(root_path: &Path) -> Result<Vec<PathBuf>> {
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

fn find_rust_files(members: &[PathBuf]) -> Vec<PathBuf> {
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

struct InstrumentVisitor<'a> {
    content: &'a str,
    byte_content: &'a [u8],
    replacements: Vec<(usize, usize, String)>,
    max_line_length: usize,
}

impl<'a> InstrumentVisitor<'a> {
    fn new(content: &'a str, max_line_length: usize) -> Self {
        Self {
            content,
            byte_content: content.as_bytes(),
            replacements: Vec::new(),
            max_line_length,
        }
    }

    fn process_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if !is_instrument_attr(attr) {
                continue;
            }

            let span = attr.span();
            let start = span.start();
            let end = span.end();

            let attr_start = line_col_to_byte(self.byte_content, start.line, start.column);
            let attr_end = line_col_to_byte(self.byte_content, end.line, end.column);

            let source_text = &self.content[attr_start..attr_end];

            let args = extract_args_from_source(source_text);

            if args.len() <= 1 {
                continue;
            }

            let indent = detect_indent(self.content, attr_start);
            let first_line_end = source_text.find('\n').unwrap_or(source_text.len());
            let first_line_length = indent + first_line_end;

            if first_line_length <= self.max_line_length {
                continue;
            }

            let formatted = format_instrument_attr(&args, indent);

            self.replacements.push((attr_start, attr_end, formatted));
        }
    }
}

impl<'a> Visit<'a> for InstrumentVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        self.process_attrs(&node.attrs);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        self.process_attrs(&node.attrs);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'a syn::TraitItemFn) {
        self.process_attrs(&node.attrs);
        syn::visit::visit_trait_item_fn(self, node);
    }
}

fn line_col_to_byte(content: &[u8], line: usize, col: usize) -> usize {
    let mut byte_pos = 0;
    let mut current_line = 1;

    while current_line < line && byte_pos < content.len() {
        if content[byte_pos] == b'\n' {
            current_line += 1;
        }
        byte_pos += 1;
    }

    byte_pos + col
}

fn detect_indent(content: &str, attr_offset: usize) -> usize {
    let lines: Vec<&str> = content[..attr_offset].lines().collect();
    let line = lines.last().unwrap_or(&"");

    line.chars().take_while(|c| c.is_whitespace()).count()
}

fn is_instrument_attr(attr: &Attribute) -> bool {
    let Meta::List(meta_list) = &attr.meta else {
        return false;
    };

    let segments = &meta_list.path.segments;

    match segments.len() {
        1 => segments[0].ident == "instrument",
        2 => segments[0].ident == "tracing" && segments[1].ident == "instrument",
        _ => false,
    }
}

fn extract_args_from_source(source: &str) -> Vec<String> {
    let Some(start) = source.find('(') else {
        return Vec::new();
    };
    let Some(end) = source.rfind(')') else {
        return Vec::new();
    };

    let args_text = &source[start + 1..end];

    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;

    for ch in args_text.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                current.push(ch);
                escape = true;
            },
            '"' => {
                current.push(ch);
                in_string = !in_string;
            },
            '(' | '{' | '[' if !in_string => {
                current.push(ch);
                depth += 1;
            },
            ')' | '}' | ']' if !in_string => {
                current.push(ch);
                depth -= 1;
            },
            ',' if depth == 0 && !in_string => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    args.push(trimmed.to_string());
                }
                current.clear();
            },
            _ => current.push(ch),
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        args.push(trimmed.to_string());
    }

    args
}

fn format_instrument_attr(args: &[String], indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    let arg_indent = " ".repeat(indent + 4);

    let formatted_args = args
        .iter()
        .map(|arg| {
            let reindented = reindent_nested_content(arg, indent + 4);
            format!("{arg_indent}{reindented},")
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("#[instrument(\n{formatted_args}\n{indent_str})]")
}

fn reindent_nested_content(arg: &str, base_indent: usize) -> String {
    let lines: Vec<&str> = arg.lines().collect();

    if lines.len() <= 1 {
        return arg.to_string();
    }

    let mut result = lines[0].trim().to_string();

    let start = 1;
    let end = lines.len() - 1;

    for line in &lines[start..end] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        result.push('\n');
        result.push_str(&" ".repeat(base_indent + 4));
        result.push_str(trimmed);
    }

    result.push('\n');
    let trimmed = lines[end].trim();
    result.push_str(&" ".repeat(base_indent));
    result.push_str(trimmed);

    result
}

fn format_file(content: &str, max_line_length: usize) -> Result<String> {
    let file: File = parse_file(content)?;

    let mut visitor = InstrumentVisitor::new(content, max_line_length);
    visitor.visit_file(&file);

    if visitor.replacements.is_empty() {
        return Ok(content.to_string());
    }

    visitor.replacements.sort_by_key(|(start, ..)| *start);

    let mut result = String::new();
    let mut last_pos = 0;

    for (start, end, replacement) in visitor.replacements {
        result.push_str(&content[last_pos..start]);
        result.push_str(&replacement);
        last_pos = end;
    }
    result.push_str(&content[last_pos..]);

    Ok(result)
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
    let root = find_workspace_root()?;
    let members = get_workspace_members(&root)?;
    let files = find_rust_files(&members);

    let mut has_changes = false;

    for path in files {
        let content = fs::read_to_string(&path)?;
        let formatted = match format_file(&content, args.max_line_length) {
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
        bail!("Files require formatting");
    }

    Ok(())
}
