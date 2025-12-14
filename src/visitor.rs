use syn::{Attribute, Meta, spanned::Spanned, visit::Visit};

use crate::{ResolvedConfig, formatting::format_macro_attr};

pub struct MacroVisitor<'a> {
    content: &'a str,
    byte_content: &'a [u8],
    pub replacements: Vec<(usize, usize, String)>,
    config: &'a ResolvedConfig,
}

impl<'a> MacroVisitor<'a> {
    pub fn new(content: &'a str, config: &'a ResolvedConfig) -> Self {
        Self {
            content,
            byte_content: content.as_bytes(),
            replacements: Vec::new(),
            config,
        }
    }

    fn process_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            let Some(ident) = self.is_target_macro(attr) else {
                continue;
            };

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

            if first_line_length <= self.config.max_line_length {
                continue;
            }

            let formatted = format_macro_attr(ident, &args, indent, self.config);

            self.replacements.push((attr_start, attr_end, formatted));
        }
    }

    fn is_target_macro(&self, attr: &Attribute) -> Option<String> {
        let Meta::List(meta_list) = &attr.meta else {
            return None;
        };

        let last_segment = meta_list.path.segments.last()?;

        if !self
            .config
            .macros_to_format
            .iter()
            .any(|macro_name| last_segment.ident == macro_name)
        {
            return None;
        }

        let path = meta_list
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");

        Some(path)
    }
}

impl<'a> Visit<'a> for MacroVisitor<'a> {
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
    let mut in_string = None;
    let mut escape = false;
    let mut braces = vec![];

    for ch in args_text.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        match ch {
            '\\' if in_string.is_some() => {
                current.push(ch);
                escape = true;
            },
            '"' | '\'' => {
                current.push(ch);
                if in_string == Some(ch) {
                    in_string = None;
                } else if in_string.is_none() {
                    in_string = Some(ch);
                }
            },
            '(' | '{' | '[' if in_string.is_none() => {
                current.push(ch);
                braces.push(ch);
            },
            ')' | '}' | ']' if in_string.is_none() => {
                if matches!(
                    (braces.last(), ch),
                    (Some('('), ')') | (Some('{'), '}') | (Some('['), ']')
                ) {
                    braces.pop();
                }

                current.push(ch);
            },
            ',' if braces.is_empty() && in_string.is_none() => {
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

#[cfg(test)]
mod tests {
    use crate::visitor::extract_args_from_source;

    #[test]
    fn empty_source() {
        let result = extract_args_from_source("");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn no_parentheses() {
        let result = extract_args_from_source("instrument");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn empty_args() {
        let result = extract_args_from_source("#[instrument()]");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn single_arg() {
        let result = extract_args_from_source("#[instrument(skip_all)]");
        assert_eq!(result, vec!["skip_all"]);
    }

    #[test]
    fn multiple_args() {
        let result = extract_args_from_source("#[instrument(level = \"debug\", skip_all)]");
        assert_eq!(result, vec!["level = \"debug\"", "skip_all"]);
    }

    #[test]
    fn args_with_whitespace() {
        let result = extract_args_from_source("#[instrument(  level = \"debug\"  ,  skip_all  )]");
        assert_eq!(result, vec!["level = \"debug\"", "skip_all"]);
    }

    #[test]
    fn nested_parentheses() {
        let result = extract_args_from_source("#[instrument(fields(count = x.len()))]");
        assert_eq!(result, vec!["fields(count = x.len())"]);
    }

    #[test]
    fn nested_with_multiple_args() {
        let result = extract_args_from_source(
            "#[instrument(level = \"debug\", fields(count = x.len(), status = \"ok\"), skip_all)]",
        );
        assert_eq!(
            result,
            vec![
                "level = \"debug\"",
                "fields(count = x.len(), status = \"ok\")",
                "skip_all"
            ]
        );
    }

    #[test]
    fn string_with_comma() {
        let result = extract_args_from_source("#[instrument(name = \"hello, world\")]");
        assert_eq!(result, vec!["name = \"hello, world\""]);
    }

    #[test]
    fn string_with_escaped_quote() {
        let result = extract_args_from_source("#[instrument(name = \"say \\\"hello\\\"\")]");
        assert_eq!(result, vec!["name = \"say \\\"hello\\\"\""]);
    }

    #[test]
    fn multiple_delimiter_types() {
        let result = extract_args_from_source("#[macro(vec![1, 2, 3], map {a: b})]");
        assert_eq!(result, vec!["vec![1, 2, 3]", "map {a: b}"]);
    }

    #[test]
    fn deeply_nested() {
        let result = extract_args_from_source("#[macro(outer(inner(deepest(value))))]");
        assert_eq!(result, vec!["outer(inner(deepest(value)))"]);
    }

    #[test]
    fn complex_real_world_example() {
        let result = extract_args_from_source(
            "#[instrument(name = \"ws-service\", target = LOG_TARGET, skip_all, fields(connector_count = self.connectors.len()))]",
        );
        assert_eq!(
            result,
            vec![
                "name = \"ws-service\"",
                "target = LOG_TARGET",
                "skip_all",
                "fields(connector_count = self.connectors.len())"
            ]
        );
    }

    #[test]
    fn trailing_comma() {
        let result = extract_args_from_source("#[instrument(skip_all,)]");
        assert_eq!(result, vec!["skip_all"]);
    }

    #[test]
    fn multiple_trailing_commas() {
        let result = extract_args_from_source("#[instrument(skip_all,,)]");
        assert_eq!(result, vec!["skip_all"]);
    }

    #[test]
    fn newlines_in_args() {
        let result =
            extract_args_from_source("#[instrument(\n    level = \"debug\",\n    skip_all\n)]");
        assert_eq!(result, vec!["level = \"debug\"", "skip_all"]);
    }

    #[test]
    fn string_with_escaped_backslash() {
        let result = extract_args_from_source("#[instrument(path = \"C:\\\\Users\\\\\")]");
        assert_eq!(result, vec!["path = \"C:\\\\Users\\\\\""]);
    }

    #[test]
    fn char_with_comma() {
        let result = extract_args_from_source("#[instrument(path = ',')]");
        assert_eq!(result, vec!["path = ','"]);
    }

    #[test]
    fn char_with_comma_inside_string() {
        let result = extract_args_from_source("#[instrument(path = ',', x = \"','\")]");
        assert_eq!(result, vec!["path = ','", "x = \"','\""]);
    }

    #[test]
    fn char_with_double_inside_string() {
        let result = extract_args_from_source("#[instrument(path = ',', x = \"'\"'\")]");
        assert_eq!(result, vec!["path = ','", "x = \"'\"'\""]);
    }

    #[test]
    fn mismatched_braces_treated_as_single_arg() {
        let result = extract_args_from_source("#[macro(vec![1, 2})]");
        assert_eq!(result, vec!["vec![1, 2}"]);
    }

    #[test]
    fn nested_different_brace_types() {
        let result = extract_args_from_source("#[macro(outer{inner[value]})]");
        assert_eq!(result, vec!["outer{inner[value]}"]);
    }

    #[test]
    fn multiple_args_with_mixed_nested_braces() {
        let result = extract_args_from_source("#[macro(vec![1, 2], map{a: b}, tuple(x, y))]");
        assert_eq!(result, vec!["vec![1, 2]", "map{a: b}", "tuple(x, y)"]);
    }

    #[test]
    fn deeply_nested_mixed_braces() {
        let result = extract_args_from_source("#[macro(outer(middle[inner{value}]))]");
        assert_eq!(result, vec!["outer(middle[inner{value}])"]);
    }

    #[test]
    fn comma_inside_nested_braces() {
        let result = extract_args_from_source("#[macro(skip, fields{a: 1, b: 2}, level)]");
        assert_eq!(result, vec!["skip", "fields{a: 1, b: 2}", "level"]);
    }

    #[test]
    fn mismatched_closing_before_comma() {
        let result = extract_args_from_source("#[macro(vec![1}, x))]");
        assert_eq!(result, vec!["vec![1}, x)"]);
    }
}
