use std::{env::current_dir, fs, path::PathBuf};

use anyhow::Result;
use clap::ValueEnum;
use serde::Deserialize;

use crate::CONFIG_FILENAME;

const SPACE_CHAR: &str = " ";
const TAB_CHAR: &str = "\t";

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub max_line_length: Option<usize>,
    pub indent_width: Option<usize>,
    pub indent_char: Option<IndentChar>,
    pub macros_to_format: Option<Vec<String>>,
    pub ignore_dirs: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone, Copy, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum IndentChar {
    Space,
    Tab,
}

impl IndentChar {
    pub fn to_string_repeated(self, count: usize) -> String {
        match self {
            IndentChar::Space => SPACE_CHAR.repeat(count),
            IndentChar::Tab => TAB_CHAR.repeat(count),
        }
    }
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }

    pub fn find_config_file() -> Option<PathBuf> {
        let dir = current_dir().ok()?;
        let candidate = dir.join(CONFIG_FILENAME);

        if candidate.exists() {
            return Some(candidate);
        }

        None
    }
}

pub struct ResolvedConfig {
    pub max_line_length: usize,
    pub indent_width: usize,
    pub indent_char: IndentChar,
    pub macros_to_format: Vec<String>,
    pub ignore_dirs: Vec<String>,
}

impl ResolvedConfig {
    pub fn indent_string(&self) -> String {
        self.indent_char.to_string_repeated(self.indent_width)
    }
}
