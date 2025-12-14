use syn::{Attribute, Meta, spanned::Spanned, visit::Visit};

use crate::formatting::format_instrument_attr;

pub struct InstrumentVisitor<'a> {
    content: &'a str,
    byte_content: &'a [u8],
    pub replacements: Vec<(usize, usize, String)>,
    max_line_length: usize,
}

impl<'a> InstrumentVisitor<'a> {
    pub fn new(content: &'a str, max_line_length: usize) -> Self {
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
