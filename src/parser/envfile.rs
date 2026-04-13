use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::utils::string::{is_placeholder, mask_value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum EnvFileType {
    Local,
    Example,
    Development,
    Production,
    Staging,
    Test,
    Generic,
}

impl EnvFileType {
    pub fn from_filename(name: &str) -> Self {
        match name {
            ".env" => Self::Generic,
            ".env.local" => Self::Local,
            ".env.example" | ".env.sample" | ".env.template" => Self::Example,
            ".env.development" => Self::Development,
            ".env.production" => Self::Production,
            ".env.staging" => Self::Staging,
            ".env.test" => Self::Test,
            _ => Self::Generic,
        }
    }

    pub fn is_example(&self) -> bool {
        matches!(self, Self::Example)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
    pub value_masked: String,
    pub line: usize,
    pub is_empty: bool,
    pub is_placeholder: bool,
    pub is_commented: bool,
}

impl EnvEntry {
    pub fn new(key: String, value: String, line: usize) -> Self {
        let is_empty = value.is_empty();
        let is_plc = is_placeholder(&value);
        let value_masked = mask_value(&value);
        Self {
            key,
            value,
            value_masked,
            line,
            is_empty,
            is_placeholder: is_plc,
            is_commented: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvFile {
    pub path: PathBuf,
    pub file_type: EnvFileType,
    pub entries: Vec<EnvEntry>,
    pub is_git_tracked: bool,
    pub parse_errors: Vec<String>,
}

impl EnvFile {
    pub fn new(path: PathBuf) -> Self {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let file_type = EnvFileType::from_filename(file_name);
        Self {
            path,
            file_type,
            entries: vec![],
            is_git_tracked: false,
            parse_errors: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|e| e.key.as_str())
    }

    pub fn is_example(&self) -> bool {
        self.file_type.is_example()
    }

    #[allow(dead_code)]
    pub fn classify_with_config(path: &Path, example_files: &[String], _local_files: &[String]) -> EnvFileType {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if example_files.iter().any(|e| e == file_name) {
            EnvFileType::Example
        } else {
            EnvFileType::from_filename(file_name)
        }
    }
}
