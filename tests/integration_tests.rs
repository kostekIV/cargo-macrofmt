use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow, bail};
use cargo_macrofmt::format_file;
use similar::TextDiff;

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
    format_file(input, max_line_length).expect("Failed to format file")
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
            TextDiff::from_lines(&expected, &result)
                .unified_diff()
                .header(
                    &input_path.to_string_lossy(),
                    &expected_path.to_string_lossy(),
                )
                .context_radius(3)
                .to_writer(std::io::stderr())?;

            bail!("Test case '{case_name}' failed");
        }
    }

    Ok(())
}
