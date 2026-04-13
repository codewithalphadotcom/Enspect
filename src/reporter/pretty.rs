use crate::audit::findings::{AuditReport, Finding, SecretFinding, Severity};
use owo_colors::OwoColorize;

// ── ASCII art banner ────────────────────────────────────────────────────────

const BANNER: &str = r#"
  ███████╗███╗   ██╗███████╗██████╗ ███████╗ ██████╗████████╗
  ██╔════╝████╗  ██║██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝
  █████╗  ██╔██╗ ██║███████╗██████╔╝█████╗  ██║        ██║
  ██╔══╝  ██║╚██╗██║╚════██║██╔═══╝ ██╔══╝  ██║        ██║
  ███████╗██║ ╚████║███████║██║     ███████╗╚██████╗   ██║
  ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝     ╚══════╝ ╚═════╝   ╚═╝
"#;

const DIVIDER_WIDTH: usize = 64;

fn divider() -> String {
    "─".repeat(DIVIDER_WIDTH)
}

fn section_header(icon: &str, title: &str, count: usize) -> String {
    let label = format!("{icon} {title} ({count})");
    let pad = DIVIDER_WIDTH.saturating_sub(label.len() + 4);
    format!("  {label} {}", "─".repeat(pad))
}

// ── Public entry ────────────────────────────────────────────────────────────

pub fn render(report: &AuditReport, color: bool) -> String {
    if color {
        render_colored(report)
    } else {
        render_plain(report)
    }
}

// ── Colored output ──────────────────────────────────────────────────────────

fn render_colored(report: &AuditReport) -> String {
    let mut out = String::new();

    // Banner
    for line in BANNER.lines() {
        out.push_str(&format!("{}\n", line.bright_yellow().bold().to_string()));
    }
    out.push_str(&format!(
        "  {}\n",
        "Environment Variable Auditor".dimmed().to_string()
    ));
    out.push_str(&format!(
        "  {}\n\n",
        format!("v{}", report.version).dimmed().to_string()
    ));

    // Stats bar
    out.push_str(&format!("  {}\n", divider().dimmed().to_string()));
    out.push_str(&format!(
        "  {} Scanned {} files in {:.2}s\n",
        "[*]".bright_yellow().to_string(),
        report.stats.files_scanned.to_string().bold().to_string(),
        report.stats.duration_ms as f64 / 1000.0,
    ));
    out.push_str(&format!(
        "  {} Found {} env references ({} unique)\n",
        "[*]".bright_yellow().to_string(),
        report.stats.unique_vars_referenced.to_string().bold().to_string(),
        report.stats.unique_vars_defined,
    ));
    out.push_str(&format!(
        "  {} Parsed {} env files\n",
        "[*]".bright_yellow().to_string(),
        report.stats.env_files_found.to_string().bold().to_string(),
    ));
    out.push_str(&format!("  {}\n\n", divider().dimmed().to_string()));

    // Findings sections
    let missing = report.missing();
    let secrets = report.secrets();
    let undoc = report.undocumented();
    let unused = report.unused();
    let empty = report.empty();
    let git_findings = report.git_findings();

    let total_findings = missing.len() + secrets.len() + undoc.len()
        + unused.len() + empty.len() + git_findings.len();

    if total_findings == 0 {
        out.push_str(&format!(
            "  {} {}\n\n",
            "[/]".bright_yellow().bold().to_string(),
            "All clear -- no issues found.".green().to_string(),
        ));
    }

    // Missing
    if !missing.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[x]", "MISSING", missing.len())
                .red()
                .bold()
                .to_string(),
        ));
        for f in &missing {
            render_finding_colored(f, &mut out);
        }
        out.push('\n');
    }

    // Secrets
    if !secrets.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[!]", "SECRETS DETECTED", secrets.len())
                .red()
                .bold()
                .to_string(),
        ));
        for f in &secrets {
            render_finding_colored(f, &mut out);
        }
        out.push('\n');
    }

    // Undocumented
    if !undoc.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[~]", "UNDOCUMENTED", undoc.len())
                .yellow()
                .bold()
                .to_string(),
        ));
        for f in &undoc {
            render_finding_colored(f, &mut out);
        }
        out.push('\n');
    }

    // Unused
    if !unused.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[-]", "UNUSED", unused.len())
                .dimmed()
                .bold()
                .to_string(),
        ));
        for f in &unused {
            render_finding_colored(f, &mut out);
        }
        out.push('\n');
    }

    // Empty
    if !empty.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[.]", "EMPTY / PLACEHOLDER", empty.len())
                .dimmed()
                .bold()
                .to_string(),
        ));
        for f in &empty {
            render_finding_colored(f, &mut out);
        }
        out.push('\n');
    }

    // Git
    if !git_findings.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[!]", "GIT ISSUES", git_findings.len())
                .red()
                .bold()
                .to_string(),
        ));
        for f in &git_findings {
            render_finding_colored(f, &mut out);
        }
        out.push('\n');
    }

    // Summary box
    out.push_str(&format!(
        "  {}\n",
        "╭──────────────────────────────────────────────╮"
            .dimmed()
            .to_string()
    ));
    out.push_str(&format!(
        "  {}  {}\n",
        "│".dimmed().to_string(),
        "SUMMARY".bright_yellow().bold().to_string(),
    ));
    out.push_str(&format!(
        "  {}\n",
        "├──────────────────────────────────────────────┤"
            .dimmed()
            .to_string()
    ));

    let rows: Vec<(&str, usize, &str)> = vec![
        ("Missing", missing.len(), "[x]"),
        ("Secrets", secrets.len(), "[!]"),
        ("Undocumented", undoc.len(), "[~]"),
        ("Unused", unused.len(), "[-]"),
        ("Empty", empty.len(), "[.]"),
        ("Git issues", git_findings.len(), "[!]"),
    ];

    for (label, count, icon) in &rows {
        let count_str = count.to_string();
        let line = format!(
            "  {}  {:<14} {:>3}  {}",
            "│".dimmed().to_string(),
            label,
            if *count > 0 {
                count_str.bold().to_string()
            } else {
                count_str.dimmed().to_string()
            },
            if *count > 0 {
                icon.to_string()
            } else {
                " ".to_string()
            }
        );
        out.push_str(&line);
        out.push('\n');
    }

    out.push_str(&format!(
        "  {}\n",
        "╰──────────────────────────────────────────────╯"
            .dimmed()
            .to_string()
    ));

    // Exit code
    out.push('\n');
    if report.exit_code == 0 {
        out.push_str(&format!(
            "  {} Exit code: 0 (clean)\n\n",
            "[/]".bright_yellow().bold().to_string(),
        ));
    } else {
        out.push_str(&format!(
            "  {} Exit code: {} (critical issues found)\n\n",
            "[x]".red().bold().to_string(),
            report.exit_code.to_string().red().bold().to_string(),
        ));
    }

    out
}

// ── Finding renderers (colored) ─────────────────────────────────────────────

fn render_finding_colored(finding: &Finding, out: &mut String) {
    match finding {
        Finding::Missing {
            key,
            references,
            in_shell,
        } => {
            out.push_str(&format!(
                "    {} {}\n",
                ">".red().to_string(),
                key.red().bold().to_string(),
            ));
            for r in references {
                out.push_str(&format!(
                    "      Referenced in: {}:{}\n",
                    r.file.display().to_string().dimmed().to_string(),
                    r.line,
                ));
            }
            if *in_shell {
                out.push_str(&format!(
                    "      {} Found in shell env -- add to .env.example\n",
                    "->".yellow().to_string(),
                ));
            } else {
                out.push_str("      Not found in: .env.local, .env.example, shell\n");
                out.push_str(&format!(
                    "      {} Add to .env.local and document in .env.example\n",
                    "->".yellow().to_string(),
                ));
            }
        }
        Finding::Secret(SecretFinding {
            key,
            file,
            line,
            severity,
            reason,
            value_preview,
            ..
        }) => {
            out.push_str(&format!(
                "    {} {}:{} -- {}\n",
                "!".red().to_string(),
                file.display().to_string().dimmed().to_string(),
                line,
                key.red().bold().to_string(),
            ));
            let sev_str = match severity {
                Severity::Critical => severity.to_string().red().bold().to_string(),
                Severity::High => severity.to_string().red().to_string(),
                Severity::Medium => severity.to_string().yellow().to_string(),
                Severity::Low => severity.to_string().dimmed().to_string(),
            };
            out.push_str(&format!("      Severity: {sev_str}\n"));
            out.push_str(&format!("      Reason:   {reason}\n"));
            out.push_str(&format!("      Value:    {value_preview}\n"));
        }
        Finding::Undocumented {
            key,
            defined_in,
            references,
        } => {
            out.push_str(&format!(
                "    {} {}\n",
                "~".yellow().to_string(),
                key.yellow().to_string(),
            ));
            for path in defined_in {
                out.push_str(&format!(
                    "      Defined in: {}\n",
                    path.display().to_string().dimmed().to_string(),
                ));
            }
            out.push_str("      Missing from: .env.example\n");
            for r in references {
                out.push_str(&format!(
                    "      Referenced in: {}:{}\n",
                    r.file.display().to_string().dimmed().to_string(),
                    r.line,
                ));
            }
        }
        Finding::Unused {
            key,
            defined_in,
            last_seen_in_git,
        } => {
            out.push_str(&format!(
                "    {} {}\n",
                "-".dimmed().to_string(),
                key.dimmed().to_string(),
            ));
            for (path, line) in defined_in {
                out.push_str(&format!(
                    "      Defined in: {}:{}\n",
                    path.display().to_string().dimmed().to_string(),
                    line,
                ));
            }
            out.push_str("      Never referenced in scanned files\n");
            if let Some(last) = last_seen_in_git {
                out.push_str(&format!("      Last seen in git: {last}\n"));
            }
        }
        Finding::Empty {
            key,
            file,
            line,
            is_placeholder,
        } => {
            out.push_str(&format!(
                "    {} {} at {}:{}\n",
                ".".dimmed().to_string(),
                key.dimmed().to_string(),
                file.display().to_string().dimmed().to_string(),
                line,
            ));
            if *is_placeholder {
                out.push_str("      Has placeholder value\n");
            } else {
                out.push_str("      Has empty value\n");
            }
        }
        Finding::GitTracked { file } => {
            out.push_str(&format!(
                "    {} {} is tracked by git!\n",
                "!".red().to_string(),
                file.display().to_string().red().bold().to_string(),
            ));
            out.push_str(&format!(
                "      {} Remove from git: git rm --cached <file>\n",
                "->".yellow().to_string(),
            ));
        }
        Finding::NotInGitignore { file } => {
            out.push_str(&format!(
                "    {} {} is not in .gitignore\n",
                "~".yellow().to_string(),
                file.display().to_string().yellow().to_string(),
            ));
            out.push_str(&format!(
                "      {} Add to .gitignore to prevent accidental commits\n",
                "->".yellow().to_string(),
            ));
        }
    }
}

// ── Plain output (no color) ─────────────────────────────────────────────────

fn render_plain(report: &AuditReport) -> String {
    let mut out = String::new();

    // Banner (plain)
    out.push_str(BANNER);
    out.push_str(&format!("  Environment Variable Auditor\n"));
    out.push_str(&format!("  v{}\n\n", report.version));

    // Stats
    out.push_str(&format!("  {}\n", divider()));
    out.push_str(&format!(
        "  [*] Scanned {} files in {:.2}s\n",
        report.stats.files_scanned,
        report.stats.duration_ms as f64 / 1000.0,
    ));
    out.push_str(&format!(
        "  [*] Found {} env references ({} unique)\n",
        report.stats.unique_vars_referenced, report.stats.unique_vars_defined,
    ));
    out.push_str(&format!(
        "  [*] Parsed {} env files\n",
        report.stats.env_files_found,
    ));
    out.push_str(&format!("  {}\n\n", divider()));

    let missing = report.missing();
    let secrets = report.secrets();
    let undoc = report.undocumented();
    let unused = report.unused();
    let empty = report.empty();
    let git_findings = report.git_findings();

    let total = missing.len() + secrets.len() + undoc.len()
        + unused.len() + empty.len() + git_findings.len();

    if total == 0 {
        out.push_str("  [/] All clear -- no issues found.\n\n");
    }

    if !missing.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[x]", "MISSING", missing.len()),
        ));
        for f in &missing {
            render_finding_plain(f, &mut out);
        }
        out.push('\n');
    }

    if !secrets.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[!]", "SECRETS DETECTED", secrets.len()),
        ));
        for f in &secrets {
            render_finding_plain(f, &mut out);
        }
        out.push('\n');
    }

    if !undoc.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[~]", "UNDOCUMENTED", undoc.len()),
        ));
        for f in &undoc {
            render_finding_plain(f, &mut out);
        }
        out.push('\n');
    }

    if !unused.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[-]", "UNUSED", unused.len()),
        ));
        for f in &unused {
            render_finding_plain(f, &mut out);
        }
        out.push('\n');
    }

    if !empty.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[.]", "EMPTY / PLACEHOLDER", empty.len()),
        ));
        for f in &empty {
            render_finding_plain(f, &mut out);
        }
        out.push('\n');
    }

    if !git_findings.is_empty() {
        out.push_str(&format!(
            "  {}\n",
            section_header("[!]", "GIT ISSUES", git_findings.len()),
        ));
        for f in &git_findings {
            render_finding_plain(f, &mut out);
        }
        out.push('\n');
    }

    // Summary
    out.push_str("  +----------------------------------------------+\n");
    out.push_str("  |  SUMMARY                                     |\n");
    out.push_str("  +----------------------------------------------+\n");
    out.push_str(&format!("  |  Missing        {:>3}  [x]                     |\n", missing.len()));
    out.push_str(&format!("  |  Secrets        {:>3}  [!]                     |\n", secrets.len()));
    out.push_str(&format!("  |  Undocumented   {:>3}  [~]                     |\n", undoc.len()));
    out.push_str(&format!("  |  Unused         {:>3}  [-]                     |\n", unused.len()));
    out.push_str(&format!("  |  Empty          {:>3}  [.]                     |\n", empty.len()));
    out.push_str(&format!("  |  Git issues     {:>3}  [!]                     |\n", git_findings.len()));
    out.push_str("  +----------------------------------------------+\n");

    out.push('\n');
    if report.exit_code == 0 {
        out.push_str("  [/] Exit code: 0 (clean)\n\n");
    } else {
        out.push_str(&format!(
            "  [x] Exit code: {} (critical issues found)\n\n",
            report.exit_code,
        ));
    }

    out
}

fn render_finding_plain(finding: &Finding, out: &mut String) {
    match finding {
        Finding::Missing {
            key,
            references,
            in_shell,
        } => {
            out.push_str(&format!("    > {key}\n"));
            for r in references {
                out.push_str(&format!(
                    "      Referenced in: {}:{}\n",
                    r.file.display(),
                    r.line,
                ));
            }
            if *in_shell {
                out.push_str("      -> Found in shell env -- add to .env.example\n");
            } else {
                out.push_str("      Not found in: .env.local, .env.example, shell\n");
                out.push_str("      -> Add to .env.local and document in .env.example\n");
            }
        }
        Finding::Secret(s) => {
            out.push_str(&format!(
                "    ! {}:{} -- {}\n",
                s.file.display(),
                s.line,
                s.key,
            ));
            out.push_str(&format!("      Severity: {}\n", s.severity));
            out.push_str(&format!("      Reason:   {}\n", s.reason));
            out.push_str(&format!("      Value:    {}\n", s.value_preview));
        }
        Finding::Undocumented {
            key,
            defined_in,
            references,
        } => {
            out.push_str(&format!("    ~ {key}\n"));
            for p in defined_in {
                out.push_str(&format!("      Defined in: {}\n", p.display()));
            }
            out.push_str("      Missing from: .env.example\n");
            for r in references {
                out.push_str(&format!(
                    "      Referenced in: {}:{}\n",
                    r.file.display(),
                    r.line,
                ));
            }
        }
        Finding::Unused {
            key,
            defined_in,
            last_seen_in_git,
        } => {
            out.push_str(&format!("    - {key}\n"));
            for (p, l) in defined_in {
                out.push_str(&format!("      Defined in: {}:{}\n", p.display(), l));
            }
            out.push_str("      Never referenced in scanned files\n");
            if let Some(last) = last_seen_in_git {
                out.push_str(&format!("      Last seen in git: {last}\n"));
            }
        }
        Finding::Empty {
            key,
            file,
            line,
            is_placeholder,
        } => {
            out.push_str(&format!("    . {key} at {}:{}\n", file.display(), line));
            if *is_placeholder {
                out.push_str("      Has placeholder value\n");
            } else {
                out.push_str("      Has empty value\n");
            }
        }
        Finding::GitTracked { file } => {
            out.push_str(&format!(
                "    ! {} is tracked by git!\n",
                file.display(),
            ));
            out.push_str("      -> Remove from git: git rm --cached <file>\n");
        }
        Finding::NotInGitignore { file } => {
            out.push_str(&format!(
                "    ~ {} is not in .gitignore\n",
                file.display(),
            ));
            out.push_str("      -> Add to .gitignore to prevent accidental commits\n");
        }
    }
}
