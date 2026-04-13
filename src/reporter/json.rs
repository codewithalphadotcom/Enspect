use anyhow::Result;
use serde::Serialize;

use crate::audit::findings::{AuditReport, Finding};

#[derive(Serialize)]
struct JsonReport {
    version: String,
    scanned_at: String,
    root: String,
    stats: JsonStats,
    findings: JsonFindings,
    git: JsonGit,
}

#[derive(Serialize)]
struct JsonStats {
    files_scanned: usize,
    duration_ms: u64,
    unique_vars_found: usize,
}

#[derive(Serialize)]
struct JsonFindings {
    missing: Vec<JsonMissing>,
    secrets: Vec<JsonSecret>,
    undocumented: Vec<JsonUndocumented>,
    unused: Vec<JsonUnused>,
    empty: Vec<JsonEmpty>,
}

#[derive(Serialize)]
struct JsonMissing {
    key: String,
    references: Vec<JsonRef>,
    in_shell: bool,
    in_example: bool,
    in_local: bool,
}

#[derive(Serialize)]
struct JsonRef {
    file: String,
    line: usize,
    col: usize,
}

#[derive(Serialize)]
struct JsonSecret {
    key: String,
    file: String,
    line: usize,
    severity: String,
    reason: String,
    pattern: Option<String>,
    value_preview: String,
}

#[derive(Serialize)]
struct JsonUndocumented {
    key: String,
    defined_in: Vec<String>,
    references: Vec<JsonRef>,
}

#[derive(Serialize)]
struct JsonUnused {
    key: String,
    defined_in: Vec<JsonLocation>,
    last_seen_in_git: Option<String>,
}

#[derive(Serialize)]
struct JsonLocation {
    file: String,
    line: usize,
}

#[derive(Serialize)]
struct JsonEmpty {
    key: String,
    file: String,
    line: usize,
    is_placeholder: bool,
}

#[derive(Serialize)]
struct JsonGit {
    env_files_tracked: Vec<String>,
    env_files_in_history: Vec<String>,
}

pub fn render(report: &AuditReport) -> Result<String> {
    let mut missing = Vec::new();
    let mut secrets = Vec::new();
    let mut undocumented = Vec::new();
    let mut unused = Vec::new();
    let mut empty = Vec::new();
    let mut tracked = Vec::new();

    for f in &report.findings {
        match f {
            Finding::Missing {
                key,
                references,
                in_shell,
            } => {
                missing.push(JsonMissing {
                    key: key.clone(),
                    references: references
                        .iter()
                        .map(|r| JsonRef {
                            file: r.file.display().to_string(),
                            line: r.line,
                            col: r.col,
                        })
                        .collect(),
                    in_shell: *in_shell,
                    in_example: false,
                    in_local: false,
                });
            }
            Finding::Secret(s) => {
                secrets.push(JsonSecret {
                    key: s.key.clone(),
                    file: s.file.display().to_string(),
                    line: s.line,
                    severity: s.severity.to_string(),
                    reason: s.reason.clone(),
                    pattern: s.pattern_name.clone(),
                    value_preview: s.value_preview.clone(),
                });
            }
            Finding::Undocumented {
                key,
                defined_in,
                references,
            } => {
                undocumented.push(JsonUndocumented {
                    key: key.clone(),
                    defined_in: defined_in.iter().map(|p| p.display().to_string()).collect(),
                    references: references
                        .iter()
                        .map(|r| JsonRef {
                            file: r.file.display().to_string(),
                            line: r.line,
                            col: r.col,
                        })
                        .collect(),
                });
            }
            Finding::Unused {
                key,
                defined_in,
                last_seen_in_git,
            } => {
                unused.push(JsonUnused {
                    key: key.clone(),
                    defined_in: defined_in
                        .iter()
                        .map(|(p, l)| JsonLocation {
                            file: p.display().to_string(),
                            line: *l,
                        })
                        .collect(),
                    last_seen_in_git: last_seen_in_git.clone(),
                });
            }
            Finding::Empty {
                key,
                file,
                line,
                is_placeholder,
            } => {
                empty.push(JsonEmpty {
                    key: key.clone(),
                    file: file.display().to_string(),
                    line: *line,
                    is_placeholder: *is_placeholder,
                });
            }
            Finding::GitTracked { file } => {
                tracked.push(file.display().to_string());
            }
            Finding::NotInGitignore { .. } => {}
        }
    }

    let json = JsonReport {
        version: report.version.clone(),
        scanned_at: report.scanned_at.to_rfc3339(),
        root: report.root.display().to_string(),
        stats: JsonStats {
            files_scanned: report.stats.files_scanned,
            duration_ms: report.stats.duration_ms,
            unique_vars_found: report.stats.unique_vars_referenced,
        },
        findings: JsonFindings {
            missing,
            secrets,
            undocumented,
            unused,
            empty,
        },
        git: JsonGit {
            env_files_tracked: tracked,
            env_files_in_history: vec![],
        },
    };

    Ok(serde_json::to_string_pretty(&json)?)
}
