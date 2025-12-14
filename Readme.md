# cargo-macrofmt

A best-effort formatter for Rust macro attributes that splits long single-line macros into multi-line format.

## Problem

`rustfmt` intentionally avoids formatting macro attributes when they contain identifiers or constants, since it cannot guarantee the macro isn't whitespace-sensitive. This means long attribute macros remain unformatted

```rust
// rustfmt formats this - only string literals
#[test_macro(param1 = "xxx", param2 = "value2", param3 = "value3", param4 = "value4")]
fn test_function() {
    assert!(true);
}

// but not this - constants make rustfmt skip formatting
#[instrument(level = "debug", target = LOG_TARGET, name = "handle-msg", skip_all)]
async fn handle_msg(&mut self, msg: Request<T>) -> Result<(), ProcessError> {
    Ok(())
}
```

Most attribute macros used in practice (`tracing::instrument`, `tokio::test`, etc.) are not whitespace-sensitive and can benefit from basic formatting when they exceed line length limits.

This tool was created because I am too lazy to format all of them by hand (and because I like to enforce a single consistent format when possible).

## Similar Tools

To the best of my knowledge, no other tool provides this specific functionality for Rust macro formatting. If you know of an existing tool that formats macro attributes, please let me know - I would genuinely love to use it instead :D.

## Solution

`cargo-macrofmt` is a simple, best-effort formatter that splits long macro attributes across multiple lines. It formats only by splitting into new lines and adding commas at the end. It does not reformat arguments themselves unless they contain nested parentheses.

```rust
// Before
#[instrument(level = "debug", target = LOG_TARGET, name = "handle-msg", skip_all)]
async fn handle_msg(&mut self) -> Result<()> {
    Ok(())
}

// After
#[instrument(
    level = "debug",
    target = LOG_TARGET,
    name = "handle-msg",
    skip_all,
)]
async fn handle_msg(&mut self) -> Result<()> {
    Ok(())
}
```

## Installation

cargo install cargo-macrofmt

## Usage

Format workspace with specific macros:

```sh
cargo macrofmt -m instrument -m test_macro
```

### Check mode

Check formatting without modifying files:

```sh
cargo macrofmt -m instrument --check
```

### Format single file or directory

```sh
cargo macrofmt -m instrument -f src/main.rs
cargo macrofmt -m instrument -f ./crates/my-crate
```

### Custom formatting options

```sh
cargo macrofmt -m instrument --max-line-length 120 --indent-width 2
```

## Best Practices

**Run `rustfmt` first, then `cargo-macrofmt`** - this ensures your code follows standard Rust formatting conventions before applying macro-specific formatting.

## Command-line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--check` | | Check formatting without modifying files | `false` |
| `--max-line-length` | | Maximum line length before formatting | `80` |
| `--indent-width` | | Number of spaces/tabs per indentation level | `4` |
| `--indent-char` | | Character to use for indentation (`space` or `tab`) | `space` |
| `--macros-to-format` | `-m` | Macro names to format (required) | - |
| `--ignore-dirs` | `-i` | Directory names to ignore | `["target"]` |
| `--file` | `-f` | Specific file or directory to format | workspace root |

## Macro Matching

The tool matches macro names by their last path segment:

- `-m instrument` matches both `#[instrument(...)]` and `#[tracing::instrument(...)]`
- `-m test` matches `#[test]` and `#[tokio::test]`

This allows flexible matching without requiring full path specification.

## Formatting Rules

The formatter is a simple, best-effort tool that:

1. Only formats macros specified via `-m` flag or config file
2. Only formats when first line exceeds configured line length
3. Splits arguments across multiple lines with configurable indentation
4. Adds trailing commas after each argument
5. Preserves nested structures (like `fields(...)`) with additional indentation
6. Does not reformat argument content itself (preserves spacing and tokens as-is)

## Examples

### With configuration file

Create `macrofmt.toml`:

```toml
max_line_length = 80 # by default
macros_to_format = ["instrument", "benchmark"] # empty by default
ignore_dirs = ["target"] # by default
indent_width = 4 # by default
indent_char = "space" # by default or use "tab" for well, tabs
```

Then simply run:

```sh
cargo macrofmt
```

### Multiple macros

```sh
cargo macrofmt -m instrument -m test_macro -m benchmark
```

### Custom indentation

```sh
# Use 2-space indentation
cargo macrofmt -m instrument --indent-width 2

# Use tabs
cargo macrofmt -m instrument --indent-char tab
```

### Integration with CI

```sh
# Check formatting in CI
cargo macrofmt -m instrument --check

# Exit code 1 if formatting needed, 0 if already formatted
```

### Ignore additional directories

```sh
cargo macrofmt -m instrument -i target -i generated -i vendor
```

## License

MIT
