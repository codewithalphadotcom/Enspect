use regex::Regex;
use std::sync::LazyLock;

use super::reference::{Language, PatternType};

pub struct PatternDef {
    pub regex: &'static LazyLock<Regex>,
    pub pattern_type: PatternType,
    pub language: Language,
    /// Which capture group contains the variable name (0 = no static name, dynamic)
    pub capture_group: usize,
}

// ── JavaScript / TypeScript patterns ──

static RE_PROCESS_ENV_DOT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"process\.env\.([A-Z_][A-Z0-9_]*)").unwrap());

static RE_PROCESS_ENV_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"process\.env\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap());

static RE_PROCESS_ENV_DYNAMIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"process\.env\[([a-z_][a-zA-Z0-9_]*)\]").unwrap());

static RE_IMPORT_META_ENV_DOT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"import\.meta\.env\.([A-Z_][A-Z0-9_]*)").unwrap());

static RE_IMPORT_META_ENV_BRACKET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"import\.meta\.env\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap());

static RE_DENO_ENV_GET: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"Deno\.env\.get\(["']([A-Z_][A-Z0-9_]*)["']\)"#).unwrap());

// ── Rust patterns ──

static RE_ENV_MACRO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"env!\(["']([A-Z_][A-Z0-9_]*)["']\)"#).unwrap());

static RE_STD_ENV_VAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:std::)?env::var\(["']([A-Z_][A-Z0-9_]*)["']\)"#).unwrap()
});

static RE_STD_ENV_VAR_OS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:std::)?env::var_os\(["']([A-Z_][A-Z0-9_]*)["']\)"#).unwrap()
});

static RE_DOTENV_VAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"dotenv::var\(["']([A-Z_][A-Z0-9_]*)["']\)"#).unwrap());

// ── Python patterns ──

static RE_OS_ENVIRON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"os\.environ\[["']([A-Z_][A-Z0-9_]*)["']\]"#).unwrap()
});

static RE_OS_ENVIRON_GET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"os\.environ\.get\(["']([A-Z_][A-Z0-9_]*)["']\)"#).unwrap()
});

static RE_OS_GETENV: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"os\.getenv\(["']([A-Z_][A-Z0-9_]*)["']\)"#).unwrap());

// ── Shell patterns ──

static RE_SHELL_VAR_BRACED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap());

static RE_SHELL_VAR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$([A-Z_][A-Z0-9_]*)").unwrap());

pub fn js_ts_patterns() -> Vec<PatternDef> {
    vec![
        PatternDef {
            regex: &RE_PROCESS_ENV_DOT,
            pattern_type: PatternType::ProcessEnv,
            language: Language::JavaScript,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_PROCESS_ENV_BRACKET,
            pattern_type: PatternType::ProcessEnvBracket,
            language: Language::JavaScript,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_PROCESS_ENV_DYNAMIC,
            pattern_type: PatternType::Dynamic,
            language: Language::JavaScript,
            capture_group: 0, // dynamic — no static key
        },
        PatternDef {
            regex: &RE_IMPORT_META_ENV_DOT,
            pattern_type: PatternType::ImportMetaEnv,
            language: Language::JavaScript,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_IMPORT_META_ENV_BRACKET,
            pattern_type: PatternType::ImportMetaEnvBracket,
            language: Language::JavaScript,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_DENO_ENV_GET,
            pattern_type: PatternType::DenoEnvGet,
            language: Language::JavaScript,
            capture_group: 1,
        },
    ]
}

pub fn rust_patterns() -> Vec<PatternDef> {
    vec![
        PatternDef {
            regex: &RE_ENV_MACRO,
            pattern_type: PatternType::EnvMacro,
            language: Language::Rust,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_STD_ENV_VAR,
            pattern_type: PatternType::StdEnvVar,
            language: Language::Rust,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_STD_ENV_VAR_OS,
            pattern_type: PatternType::StdEnvVarOs,
            language: Language::Rust,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_DOTENV_VAR,
            pattern_type: PatternType::DotenvVar,
            language: Language::Rust,
            capture_group: 1,
        },
    ]
}

pub fn python_patterns() -> Vec<PatternDef> {
    vec![
        PatternDef {
            regex: &RE_OS_ENVIRON,
            pattern_type: PatternType::OsEnviron,
            language: Language::Python,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_OS_ENVIRON_GET,
            pattern_type: PatternType::OsEnvironGet,
            language: Language::Python,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_OS_GETENV,
            pattern_type: PatternType::OsGetenv,
            language: Language::Python,
            capture_group: 1,
        },
    ]
}

pub fn shell_patterns() -> Vec<PatternDef> {
    vec![
        PatternDef {
            regex: &RE_SHELL_VAR_BRACED,
            pattern_type: PatternType::ShellVarBraced,
            language: Language::Shell,
            capture_group: 1,
        },
        PatternDef {
            regex: &RE_SHELL_VAR,
            pattern_type: PatternType::ShellVar,
            language: Language::Shell,
            capture_group: 1,
        },
    ]
}

pub fn patterns_for_extension(ext: &str) -> Vec<PatternDef> {
    match ext {
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" => js_ts_patterns(),
        "rs" => rust_patterns(),
        "py" => python_patterns(),
        "sh" | "bash" | "zsh" => shell_patterns(),
        _ => vec![],
    }
}
