use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::cli::args::ScanArgs;
use crate::config::Config;
use crate::scanner::walker;

const DIV: usize = 60;

fn divider() -> String {
    "─".repeat(DIV)
}

pub fn run(args: &ScanArgs) -> Result<()> {
    let root = PathBuf::from(&args.root).canonicalize()?;
    let config = Config::load(&root)?;
    let result = walker::scan_directory(&root, &config)?;

    if args.json {
        let json = serde_json::to_string_pretty(&result.references)?;
        println!("{json}");
        return Ok(());
    }

    let total = result.references.len();

    // Group by key
    let mut by_key: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for r in &result.references {
        by_key.entry(r.key.clone()).or_default().push(r);
    }

    let unique = by_key.len();

    // Header
    println!();
    println!("  {}", divider().dimmed());
    println!(
        "  {}  {}{}  {}{}  {}{}",
        "[*]".bright_yellow(),
        result.files_scanned.to_string().bright_yellow().bold(),
        " files scanned".white(),
        total.to_string().bright_yellow().bold(),
        " references".white(),
        unique.to_string().bright_yellow().bold(),
        " unique".white(),
    );
    println!("  {}", divider().dimmed());

    if by_key.is_empty() {
        println!(
            "\n  {} {}\n",
            "[/]".bright_yellow(),
            "No env var references found.".dimmed(),
        );
        return Ok(());
    }

    println!();
    for (key, refs) in &by_key {
        let count = refs.len();
        println!(
            "  {}  {}",
            key.bold(),
            format!("{} {}", count, if count == 1 { "reference" } else { "references" }),
        );
        for r in refs {
            let path_str = r.file.display().to_string();
            // Trim root prefix for shorter paths
            let short_path = path_str
                .strip_prefix(&root.display().to_string())
                .map(|s| s.trim_start_matches('/'))
                .unwrap_or(&path_str);

            println!(
                "    {}  {}{}  {}",
                "·".dimmed(),
                short_path.dimmed(),
                format!(":{}", r.line).dimmed(),
                format!("{:?}", r.pattern_type).dimmed(),
            );
        }
        println!();
    }

    if !result.dynamic_warnings.is_empty() {
        println!("  {}", divider().dimmed());
        println!("  {}", "Dynamic access detected — variable names cannot be statically resolved:".yellow());
        for (file, line) in &result.dynamic_warnings {
            let path_str = file.display().to_string();
            let short_path = path_str
                .strip_prefix(&root.display().to_string())
                .map(|s| s.trim_start_matches('/'))
                .unwrap_or(&path_str);
            println!("    {} {}:{}", "·".dimmed(), short_path.dimmed(), line);
        }
        println!();
    }

    println!("  {}", divider().dimmed());
    println!();

    Ok(())
}
