use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow, bail};
use cargo_macrofmt::{IndentChar, ResolvedConfig, format_file, print_diff};

fn find_test_cases() -> Vec<PathBuf> {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases");

    fs::read_dir(&test_dir)
        .expect("Failed to read test cases directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path())
        .collect()
}

fn run_formatter(input: &str, max_line_length: usize) -> String {
    format_file(
        input,
        &ResolvedConfig {
            max_line_length,
            indent_width: 4,
            indent_char: IndentChar::Space,
            macros_to_format: vec!["instrument".to_owned(), "test_macro".to_owned()],
            ignore_dirs: vec![],
        },
    )
    .expect("Failed to format file")
}

#[test]
fn test_all_cases() -> Result<()> {
    let cases = find_test_cases();

    assert!(!cases.is_empty(), "No test cases found");

    for case_dir in cases {
        let case_name = case_dir
            .file_name()
            .ok_or(anyhow!("Bad directory termination"))?
            .to_string_lossy();

        let input_path = case_dir.join("input.rs");
        let expected_path = case_dir.join("expected.rs");

        assert!(input_path.exists(), "Missing input.rs in {case_name}");
        assert!(expected_path.exists(), "Missing expected.rs in {case_name}");

        let input = fs::read_to_string(&input_path)?;
        let expected = fs::read_to_string(&expected_path)?;

        let result = run_formatter(&input, 80);

        if result != expected {
            print_diff(
                &expected,
                &result,
                &expected_path.to_string_lossy(),
                &input_path.to_string_lossy(),
            )?;

            bail!("Test case '{case_name}' failed");
        }
    }

    Ok(())
}
