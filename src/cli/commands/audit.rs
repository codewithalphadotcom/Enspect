use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use chrono::Utc;

use crate::audit::cross_reference::cross_reference;
use crate::audit::findings::{AuditReport, Finding, ScanStats};
use crate::cli::args::AuditArgs;
use crate::config::Config;
use crate::detector::secret::detect_secrets;
use crate::git::{gitignore, tracking};
use crate::parser::dotenv;
use crate::reporter;
use crate::scanner::walker;

pub fn run(args: &AuditArgs) -> Result<u8> {
    let start = Instant::now();

    // Load config
    let root = PathBuf::from(&args.root).canonicalize()?;
    let mut config = if let Some(cfg_path) = &args.config {
        Config::load_from(&PathBuf::from(cfg_path))?
    } else {
        Config::load(&root)?
    };

    // Apply CLI overrides
    if args.no_secrets {
        config.secrets.enabled = false;
    }
    if args.no_git {
        config.git.enabled = false;
    }
    if args.no_unused {
        config.output.show_unused = false;
    }
    if args.no_empty {
        config.output.show_empty = false;
    }
    if args.ci {
        config.output.format = "json".to_string();
    }
    if let Some(ref fail_on) = args.fail_on {
        config.output.fail_on = fail_on.split(',').map(|s| s.trim().to_string()).collect();
    }

    let format = if args.ci { "json" } else { &args.format };

    // Scan source files
    let scan_result = walker::scan_directory(&root, &config)?;

    // Print dynamic access warnings
    if !args.quiet {
        for (file, line) in &scan_result.dynamic_warnings {
            eprintln!(
                "Warning: Dynamic env access detected at {}:{} — cannot audit",
                file.display(),
                line
            );
        }
    }

    // Parse .env files
    let env_files = dotenv::find_env_files(&root)?;

    // Get shell environment
    let shell_env: HashMap<String, String> = std::env::vars().collect();

    // Cross-reference
    let mut findings = cross_reference(&scan_result.references, &env_files, &shell_env, &config);

    // Secret detection
    if config.secrets.enabled {
        let secret_findings = detect_secrets(&env_files, &config);
        for sf in secret_findings {
            findings.push(Finding::Secret(sf));
        }
    }

    // Git checks
    if config.git.enabled {
        let non_example_env_files: Vec<_> = env_files
            .iter()
            .filter(|ef| !ef.is_example())
            .map(|ef| ef.path.as_path())
            .collect();

        let tracked = tracking::find_tracked_env_files(&non_example_env_files, &root);
        for file in tracked {
            findings.push(Finding::GitTracked { file });
        }

        // Check .gitignore coverage
        let env_file_names: Vec<_> = env_files
            .iter()
            .filter(|ef| !ef.is_example())
            .filter_map(|ef| ef.path.file_name().and_then(|n| n.to_str()))
            .collect();
        let missing_from_gitignore = gitignore::find_missing_from_gitignore(&env_file_names, &root);
        for name in missing_from_gitignore {
            findings.push(Finding::NotInGitignore {
                file: PathBuf::from(name),
            });
        }
    }

    let duration = start.elapsed();

    // Compute unique var counts
    let mut unique_referenced = std::collections::HashSet::new();
    for r in &scan_result.references {
        unique_referenced.insert(r.key.clone());
    }
    let mut unique_defined = std::collections::HashSet::new();
    for ef in &env_files {
        for entry in &ef.entries {
            unique_defined.insert(entry.key.clone());
        }
    }

    let stats = ScanStats {
        files_scanned: scan_result.files_scanned,
        env_files_found: env_files.len(),
        duration_ms: duration.as_millis() as u64,
        unique_vars_referenced: unique_referenced.len(),
        unique_vars_defined: unique_defined.len(),
    };

    let exit_code = {
        let fail_on = &config.output.fail_on;
        let mut code: u8 = 0;
        for f in &findings {
            if fail_on.iter().any(|fo| fo == f.category()) {
                code = 1;
                break;
            }
        }
        code
    };

    let report = AuditReport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        scanned_at: Utc::now(),
        root: root.clone(),
        stats,
        findings,
        exit_code,
    };

    let color = !args.no_color && format == "pretty";
    let output = reporter::render(&report, format, color)?;
    print!("{output}");

    Ok(report.exit_code)
}
