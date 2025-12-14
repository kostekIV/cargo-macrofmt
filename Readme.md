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

todo

## Usage

todo

## Command-line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--check` | | Check formatting without modifying files | false |
| `--max-line-length` | | Maximum line length before formatting | 80 |
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

1. Only formats macros specified via `-m` flag
2. Only formats when first line exceeds `--max-line-length`
3. Splits arguments across multiple lines with 4-space indentation
4. Adds trailing commas after each argument
5. Preserves nested structures (like `fields(...)`) with additional indentation
6. Does not reformat argument content itself (preserves spacing and tokens as-is)

## Examples

### Multiple macros in one project

```bash
cargo macrofmt -m instrument -m my_custom_macro -m benchmark
```

### Integration with CI

```bash
# Check formatting in CI
cargo macrofmt -m instrument --check

# Exit code 1 if formatting needed, 0 if already formatted
```

## License

MIT
