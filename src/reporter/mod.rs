pub mod github;
pub mod json;
pub mod pretty;
pub mod sarif;

use crate::audit::AuditReport;
use anyhow::Result;

pub fn render(report: &AuditReport, format: &str, color: bool) -> Result<String> {
    match format {
        "json" => json::render(report),
        "sarif" => sarif::render(report),
        "github" => Ok(github::render(report)),
        _ => Ok(pretty::render(report, color)),
    }
}
