use crate::audit::findings::{AuditReport, Finding};

/// Render findings as GitHub Actions workflow commands.
pub fn render(report: &AuditReport) -> String {
    let mut out = String::new();

    for f in &report.findings {
        match f {
            Finding::Missing { key, references, .. } => {
                for r in references {
                    out.push_str(&format!(
                        "::error file={},line={}::{} is not defined in any .env file or shell environment\n",
                        r.file.display(),
                        r.line,
                        key,
                    ));
                }
            }
            Finding::Secret(s) => {
                out.push_str(&format!(
                    "::error file={},line={}::Possible secret detected in {} ({}): {}\n",
                    s.file.display(),
                    s.line,
                    s.key,
                    s.severity,
                    s.reason,
                ));
            }
            Finding::Undocumented { key, references, .. } => {
                for r in references {
                    out.push_str(&format!(
                        "::warning file={},line={}::{} is used but not documented in .env.example\n",
                        r.file.display(),
                        r.line,
                        key,
                    ));
                }
            }
            Finding::Unused { key, defined_in, .. } => {
                for (path, line) in defined_in {
                    out.push_str(&format!(
                        "::notice file={},line={}::{} is defined but never referenced in source code\n",
                        path.display(),
                        line,
                        key,
                    ));
                }
            }
            Finding::Empty {
                key, file, line, ..
            } => {
                out.push_str(&format!(
                    "::warning file={},line={}::{} has an empty or placeholder value\n",
                    file.display(),
                    line,
                    key,
                ));
            }
            Finding::GitTracked { file } => {
                out.push_str(&format!(
                    "::error file={}::{} is tracked by git — this may expose secrets\n",
                    file.display(),
                    file.display(),
                ));
            }
            Finding::NotInGitignore { file } => {
                out.push_str(&format!(
                    "::warning file=.gitignore::{} is not listed in .gitignore\n",
                    file.display(),
                ));
            }
        }
    }

    out
}
