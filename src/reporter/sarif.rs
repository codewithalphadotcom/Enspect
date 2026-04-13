use anyhow::Result;
use serde::Serialize;

use crate::audit::findings::{AuditReport, Finding, Severity};

#[derive(Serialize)]
struct Sarif {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: &'static str,
    version: String,
    #[serde(rename = "informationUri")]
    information_uri: &'static str,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
struct SarifRule {
    id: String,
    name: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: &'static str,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    region: Option<SarifRegion>,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "startColumn")]
    start_column: Option<usize>,
}

pub fn render(report: &AuditReport) -> Result<String> {
    let rules = vec![
        SarifRule {
            id: "ENV001".to_string(),
            name: "MissingVariable".to_string(),
            short_description: SarifMessage {
                text: "Environment variable referenced in code but not defined".to_string(),
            },
        },
        SarifRule {
            id: "ENV002".to_string(),
            name: "SecretDetected".to_string(),
            short_description: SarifMessage {
                text: "Possible secret or credential detected in env file".to_string(),
            },
        },
        SarifRule {
            id: "ENV003".to_string(),
            name: "UndocumentedVariable".to_string(),
            short_description: SarifMessage {
                text: "Environment variable not documented in .env.example".to_string(),
            },
        },
        SarifRule {
            id: "ENV004".to_string(),
            name: "UnusedVariable".to_string(),
            short_description: SarifMessage {
                text: "Environment variable defined but never referenced".to_string(),
            },
        },
        SarifRule {
            id: "ENV005".to_string(),
            name: "EmptyVariable".to_string(),
            short_description: SarifMessage {
                text: "Environment variable has empty or placeholder value".to_string(),
            },
        },
        SarifRule {
            id: "ENV006".to_string(),
            name: "GitTrackedEnvFile".to_string(),
            short_description: SarifMessage {
                text: "Env file is tracked by git".to_string(),
            },
        },
    ];

    let mut results = Vec::new();

    for f in &report.findings {
        match f {
            Finding::Missing { key, references, .. } => {
                for r in references {
                    results.push(SarifResult {
                        rule_id: "ENV001".to_string(),
                        level: "error",
                        message: SarifMessage {
                            text: format!("{key} is not defined in any .env file"),
                        },
                        locations: vec![SarifLocation {
                            physical_location: SarifPhysicalLocation {
                                artifact_location: SarifArtifactLocation {
                                    uri: r.file.display().to_string(),
                                },
                                region: Some(SarifRegion {
                                    start_line: r.line,
                                    start_column: Some(r.col),
                                }),
                            },
                        }],
                    });
                }
            }
            Finding::Secret(s) => {
                let level = match s.severity {
                    Severity::Critical | Severity::High => "error",
                    Severity::Medium => "warning",
                    Severity::Low => "note",
                };
                results.push(SarifResult {
                    rule_id: "ENV002".to_string(),
                    level,
                    message: SarifMessage {
                        text: format!("{}: {}", s.key, s.reason),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: s.file.display().to_string(),
                            },
                            region: Some(SarifRegion {
                                start_line: s.line,
                                start_column: None,
                            }),
                        },
                    }],
                });
            }
            Finding::Undocumented { key, references, .. } => {
                for r in references {
                    results.push(SarifResult {
                        rule_id: "ENV003".to_string(),
                        level: "warning",
                        message: SarifMessage {
                            text: format!("{key} is not documented in .env.example"),
                        },
                        locations: vec![SarifLocation {
                            physical_location: SarifPhysicalLocation {
                                artifact_location: SarifArtifactLocation {
                                    uri: r.file.display().to_string(),
                                },
                                region: Some(SarifRegion {
                                    start_line: r.line,
                                    start_column: Some(r.col),
                                }),
                            },
                        }],
                    });
                }
            }
            Finding::Unused { key, defined_in, .. } => {
                for (path, line) in defined_in {
                    results.push(SarifResult {
                        rule_id: "ENV004".to_string(),
                        level: "note",
                        message: SarifMessage {
                            text: format!("{key} is defined but never referenced"),
                        },
                        locations: vec![SarifLocation {
                            physical_location: SarifPhysicalLocation {
                                artifact_location: SarifArtifactLocation {
                                    uri: path.display().to_string(),
                                },
                                region: Some(SarifRegion {
                                    start_line: *line,
                                    start_column: None,
                                }),
                            },
                        }],
                    });
                }
            }
            Finding::Empty { key, file, line, .. } => {
                results.push(SarifResult {
                    rule_id: "ENV005".to_string(),
                    level: "warning",
                    message: SarifMessage {
                        text: format!("{key} has empty or placeholder value"),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: file.display().to_string(),
                            },
                            region: Some(SarifRegion {
                                start_line: *line,
                                start_column: None,
                            }),
                        },
                    }],
                });
            }
            Finding::GitTracked { file } => {
                results.push(SarifResult {
                    rule_id: "ENV006".to_string(),
                    level: "error",
                    message: SarifMessage {
                        text: format!("{} is tracked by git", file.display()),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation {
                                uri: file.display().to_string(),
                            },
                            region: None,
                        },
                    }],
                });
            }
            Finding::NotInGitignore { .. } => {}
        }
    }

    let sarif = Sarif {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "enspect",
                    version: report.version.clone(),
                    information_uri: "https://github.com/codewithalpha/enspect",
                    rules,
                },
            },
            results,
        }],
    };

    Ok(serde_json::to_string_pretty(&sarif)?)
}
