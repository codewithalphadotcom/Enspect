use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub scan: ScanConfig,
    pub env_files: EnvFilesConfig,
    pub secrets: SecretsConfig,
    pub output: OutputConfig,
    pub git: GitConfig,
    pub audit: AuditConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ScanConfig {
    pub root: String,
    pub extensions: Vec<String>,
    pub ignore_dirs: Vec<String>,
    pub ignore_files: Vec<String>,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EnvFilesConfig {
    pub example_files: Vec<String>,
    pub local_files: Vec<String>,
    pub check_git_tracking: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecretsConfig {
    pub enabled: bool,
    pub entropy_threshold: f64,
    pub min_length_for_entropy_check: usize,
    pub check_patterns: bool,
    pub custom_patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    pub format: String,
    pub color: String,
    pub show_unused: bool,
    pub show_empty: bool,
    pub fail_on: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GitConfig {
    pub enabled: bool,
    pub check_history: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuditConfig {
    pub allowed_missing: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan: ScanConfig::default(),
            env_files: EnvFilesConfig::default(),
            secrets: SecretsConfig::default(),
            output: OutputConfig::default(),
            git: GitConfig::default(),
            audit: AuditConfig::default(),
        }
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            root: ".".to_string(),
            extensions: vec![
                "ts", "tsx", "js", "jsx", "mjs", "cjs", "rs", "py", "sh", "bash", "zsh",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            ignore_dirs: vec![
                "node_modules",
                ".git",
                "dist",
                "build",
                "target",
                ".next",
                "__pycache__",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            ignore_files: vec![],
            follow_symlinks: false,
        }
    }
}

impl Default for EnvFilesConfig {
    fn default() -> Self {
        Self {
            example_files: vec![".env.example", ".env.sample", ".env.template"]
                .into_iter()
                .map(String::from)
                .collect(),
            local_files: vec![
                ".env",
                ".env.local",
                ".env.development",
                ".env.production",
                ".env.staging",
                ".env.test",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            check_git_tracking: true,
        }
    }
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            entropy_threshold: 4.5,
            min_length_for_entropy_check: 16,
            check_patterns: true,
            custom_patterns: vec![],
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "pretty".to_string(),
            color: "auto".to_string(),
            show_unused: true,
            show_empty: true,
            fail_on: vec!["missing".to_string(), "secret".to_string()],
        }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_history: false,
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            allowed_missing: vec![],
        }
    }
}

impl Config {
    /// Load config from a .Enspect.toml file. Returns default if not found.
    pub fn load(root: &Path) -> Result<Self> {
        let config_path = root.join(".Enspect.toml");
        Self::load_from(&config_path)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {path:?}"))?;
        let config: Config =
            toml::from_str(&content).with_context(|| format!("Failed to parse {path:?}"))?;
        Ok(config)
    }

    #[allow(dead_code)]
    pub fn resolve_root(&self, cli_root: Option<&str>) -> PathBuf {
        if let Some(root) = cli_root {
            PathBuf::from(root)
        } else {
            PathBuf::from(&self.scan.root)
        }
    }
}
