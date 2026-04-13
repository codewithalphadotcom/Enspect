use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(dead_code)]
pub enum Language {
    JavaScript,
    TypeScript,
    Rust,
    Python,
    Shell,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PatternType {
    ProcessEnv,
    ProcessEnvBracket,
    ImportMetaEnv,
    ImportMetaEnvBracket,
    DenoEnvGet,
    EnvMacro,
    StdEnvVar,
    StdEnvVarOs,
    DotenvVar,
    OsEnviron,
    OsEnvironGet,
    OsGetenv,
    ShellVar,
    ShellVarBraced,
    Dynamic,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvReference {
    pub key: String,
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub pattern_type: PatternType,
    pub language: Language,
}
