use crate::config::ResolvedConfig;

pub fn format_macro_attr(
    ident: String,
    args: &[String],
    indent: usize,
    config: &ResolvedConfig,
) -> String {
    let base_indent = config.indent_char.to_string_repeated(indent);
    let arg_indent = config
        .indent_char
        .to_string_repeated(indent + config.indent_width);

    let formatted_args = args
        .iter()
        .map(|arg| {
            let reindented = reindent_nested_content(arg, indent + config.indent_width, config);
            format!("{arg_indent}{reindented},")
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("#[{ident}(\n{formatted_args}\n{base_indent})]")
}

pub fn reindent_nested_content(arg: &str, base_indent: usize, config: &ResolvedConfig) -> String {
    let lines: Vec<_> = arg.lines().collect();

    if lines.len() <= 1 {
        return arg.to_string();
    }

    let [first, middle @ .., last] = lines.as_slice() else {
        return arg.to_string();
    };

    let mut result = first.trim().to_string();

    for line in middle {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        result.push('\n');
        result.push_str(
            &config
                .indent_char
                .to_string_repeated(base_indent + config.indent_width),
        );
        result.push_str(trimmed);
    }

    result.push('\n');
    result.push_str(&config.indent_char.to_string_repeated(base_indent));
    result.push_str(last.trim());

    result
}
