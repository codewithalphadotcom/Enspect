use std::path::PathBuf;

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::audit::findings::Severity;
use crate::cli::args::SecretsArgs;
use crate::config::Config;
use crate::detector::secret::detect_secrets;
use crate::parser::dotenv;

const DIV: usize = 60;

fn divider() -> String {
    "─".repeat(DIV)
}

pub fn run(args: &SecretsArgs) -> Result<()> {
    let root = PathBuf::from(&args.root).canonicalize()?;
    let config = Config::load(&root)?;

    let env_files = if let Some(ref path) = args.path {
        vec![dotenv::parse_env_file(&PathBuf::from(path))?]
    } else {
        dotenv::find_env_files(&root)?
    };

    let findings = detect_secrets(&env_files, &config);

    println!();
    println!("  {}", divider().dimmed());

    if findings.is_empty() {
        println!(
            "  {}  {}",
            "[/]".bright_yellow(),
            "No secrets detected.".dimmed(),
        );
        println!("  {}", divider().dimmed());
        println!();
        return Ok(());
    }

    println!(
        "  {}  {}  {}",
        "[!]".red().bold(),
        findings.len().to_string().bold(),
        format!("potential {} found", if findings.len() == 1 { "secret" } else { "secrets" }).dimmed(),
    );
    println!("  {}", divider().dimmed());
    println!();

    for f in &findings {
        let path_str = f.file.display().to_string();
        let short = path_str
            .strip_prefix(&root.display().to_string())
            .map(|s| s.trim_start_matches('/'))
            .unwrap_or(&path_str);

        let sev_str = match f.severity {
            Severity::Critical => f.severity.to_string().red().bold().to_string(),
            Severity::High     => f.severity.to_string().red().to_string(),
            Severity::Medium   => f.severity.to_string().yellow().to_string(),
            Severity::Low      => f.severity.to_string().dimmed().to_string(),
        };

        println!(
            "  {}  {}  {}",
            "▸".red().bold(),
            f.key.red().bold(),
            format!("{}:{}", short, f.line).dimmed(),
        );
        println!("    {}  {}", "severity".dimmed(), sev_str);
        println!("    {}  {}", "reason  ".dimmed(), f.reason.dimmed());
        println!("    {}  {}", "value   ".dimmed(), f.value_preview.dimmed());
        println!();
    }

    println!("  {}", divider().dimmed());
    println!();

    Ok(())
}
