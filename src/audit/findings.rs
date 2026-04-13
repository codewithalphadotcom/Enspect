use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;

use crate::scanner::EnvReference;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(dead_code)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SecretFinding {
    pub key: String,
    pub file: PathBuf,
    pub line: usize,
    pub severity: Severity,
    pub reason: String,
    pub pattern_name: Option<String>,
    pub entropy: f64,
    pub value_preview: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum Finding {
    Missing {
        key: String,
        references: Vec<EnvReference>,
        in_shell: bool,
    },
    Secret(SecretFinding),
    Undocumented {
        key: String,
        defined_in: Vec<PathBuf>,
        references: Vec<EnvReference>,
    },
    Unused {
        key: String,
        defined_in: Vec<(PathBuf, usize)>,
        last_seen_in_git: Option<String>,
    },
    Empty {
        key: String,
        file: PathBuf,
        line: usize,
        is_placeholder: bool,
    },
    GitTracked {
        file: PathBuf,
    },
    NotInGitignore {
        file: PathBuf,
    },
}

impl Finding {
    pub fn category(&self) -> &'static str {
        match self {
            Finding::Missing { .. } => "missing",
            Finding::Secret(_) => "secret",
            Finding::Undocumented { .. } => "undocumented",
            Finding::Unused { .. } => "unused",
            Finding::Empty { .. } => "empty",
            Finding::GitTracked { .. } => "git_tracked",
            Finding::NotInGitignore { .. } => "not_in_gitignore",
        }
    }

    #[allow(dead_code)]
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Finding::Missing { .. }
                | Finding::Secret(SecretFinding {
                    severity: Severity::Critical,
                    ..
                })
                | Finding::GitTracked { .. }
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanStats {
    pub files_scanned: usize,
    pub env_files_found: usize,
    pub duration_ms: u64,
    pub unique_vars_referenced: usize,
    pub unique_vars_defined: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub version: String,
    pub scanned_at: DateTime<Utc>,
    pub root: PathBuf,
    pub stats: ScanStats,
    pub findings: Vec<Finding>,
    pub exit_code: u8,
}

impl AuditReport {
    pub fn missing(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| matches!(f, Finding::Missing { .. }))
            .collect()
    }

    pub fn secrets(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| matches!(f, Finding::Secret(_)))
            .collect()
    }

    pub fn undocumented(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| matches!(f, Finding::Undocumented { .. }))
            .collect()
    }

    pub fn unused(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| matches!(f, Finding::Unused { .. }))
            .collect()
    }

    pub fn empty(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| matches!(f, Finding::Empty { .. }))
            .collect()
    }

    pub fn git_findings(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| matches!(f, Finding::GitTracked { .. } | Finding::NotInGitignore { .. }))
            .collect()
    }

    #[allow(dead_code)]
    pub fn compute_exit_code(&self, fail_on: &[String]) -> u8 {
        for finding in &self.findings {
            let cat = finding.category();
            if fail_on.iter().any(|f| f == cat) {
                return 1;
            }
        }
        0
    }
}
