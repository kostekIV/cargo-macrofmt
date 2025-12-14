pub fn format_instrument_attr(args: &[String], indent: usize) -> String {
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

pub fn reindent_nested_content(arg: &str, base_indent: usize) -> String {
    let lines: Vec<&str> = arg.lines().collect();

    if lines.len() <= 1 {
        return arg.to_string();
    }

    let mut result = lines[0].trim().to_string();

    if lines.len() == 2 {
        result.push('\n');
        result.push_str(&" ".repeat(base_indent));
        result.push_str(lines[1].trim());

        return result;
    }

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
