use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::cli::args::CheckArgs;
use crate::config::Config;
use crate::detector::entropy::shannon_entropy;
use crate::parser::dotenv;
use crate::scanner::walker;

const DIV: usize = 60;

fn divider() -> String {
    "─".repeat(DIV)
}

pub fn run(args: &CheckArgs) -> Result<()> {
    let root = PathBuf::from(&args.root).canonicalize()?;
    let config = Config::load(&root)?;
    let var = &args.var_name;

    println!();
    println!("  {}", divider().dimmed());
    println!(
        "  {}  {}",
        "checking".dimmed(),
        var.bold().bright_yellow(),
    );
    println!("  {}", divider().dimmed());

    // ── Source references ──────────────────────────────────────────────────
    let scan = walker::scan_directory(&root, &config)?;
    let refs: Vec<_> = scan.references.iter().filter(|r| r.key == *var).collect();

    println!();
    if refs.is_empty() {
        println!(
            "  {}  {}",
            "source".dimmed(),
            "not referenced in any source file".dimmed(),
        );
    } else {
        println!(
            "  {}  {} {}",
            "source".dimmed(),
            refs.len().to_string().bold(),
            format!("{}", if refs.len() == 1 { "location" } else { "locations" }).dimmed(),
        );
        for r in &refs {
            let path_str = r.file.display().to_string();
            let short = path_str
                .strip_prefix(&root.display().to_string())
                .map(|s| s.trim_start_matches('/'))
                .unwrap_or(&path_str);
            println!(
                "    {}  {}{}  {}",
                "·".dimmed(),
                short.dimmed(),
                format!(":{}", r.line).dimmed(),
                format!("{:?}", r.pattern_type).dimmed(),
            );
        }
    }

    // ── Env files ──────────────────────────────────────────────────────────
    println!();
    let env_files = dotenv::find_env_files(&root)?;
    let mut found_in_env = false;

    for ef in &env_files {
        for entry in &ef.entries {
            if entry.key == *var {
                if !found_in_env {
                    println!("  {}  env files", "defined".dimmed().to_string().dimmed());
                }
                found_in_env = true;

                let path_str = ef.path.display().to_string();
                let short = path_str
                    .strip_prefix(&root.display().to_string())
                    .map(|s| s.trim_start_matches('/'))
                    .unwrap_or(&path_str);

                let entropy = shannon_entropy(&entry.value);
                let entropy_label = if entropy > 4.5 {
                    format!("{:.2} bits  {}", entropy, "[high]".red())
                } else if entropy > 3.0 {
                    format!("{:.2} bits  {}", entropy, "[medium]".yellow())
                } else {
                    format!("{:.2} bits", entropy).dimmed().to_string()
                };

                println!(
                    "    {}  {}{}  value: {}  entropy: {}",
                    "·".dimmed(),
                    short.dimmed(),
                    format!(":{}", entry.line).dimmed(),
                    entry.value_masked.dimmed(),
                    entropy_label,
                );

                if entry.is_empty {
                    println!("         {} empty value", "→".yellow());
                } else if entry.is_placeholder {
                    println!("         {} placeholder value", "→".yellow());
                }
            }
        }
    }

    if !found_in_env {
        println!(
            "  {}  {}",
            "env files".dimmed(),
            "not defined in any .env file".dimmed(),
        );
    }

    // ── Shell ──────────────────────────────────────────────────────────────
    println!();
    let shell_env: HashMap<String, String> = std::env::vars().collect();
    if shell_env.contains_key(var) {
        println!(
            "  {}  {} {}",
            "shell".dimmed(),
            "[+]".bright_yellow(),
            "present in current shell environment".dimmed(),
        );
    } else {
        println!(
            "  {}  {}",
            "shell".dimmed(),
            "not present in current shell environment".dimmed(),
        );
    }

    println!();
    println!("  {}", divider().dimmed());
    println!();

    Ok(())
}
