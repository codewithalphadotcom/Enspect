use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::cli::args::DiffArgs;
use crate::parser::dotenv;

const DIV: usize = 60;

fn divider() -> String {
    "─".repeat(DIV)
}

pub fn run(args: &DiffArgs) -> Result<()> {
    let ef1 = dotenv::parse_env_file(&PathBuf::from(&args.file1))?;
    let ef2 = dotenv::parse_env_file(&PathBuf::from(&args.file2))?;

    let keys1: BTreeSet<String> = ef1.entries.iter().map(|e| e.key.clone()).collect();
    let keys2: BTreeSet<String> = ef2.entries.iter().map(|e| e.key.clone()).collect();

    let only_1: Vec<_> = keys1.difference(&keys2).collect();
    let only_2: Vec<_> = keys2.difference(&keys1).collect();
    let shared: Vec<_> = keys1.intersection(&keys2).collect();

    let f1 = &args.file1;
    let f2 = &args.file2;

    println!();
    println!("  {}", divider().dimmed());
    println!(
        "  {}  {}  {}  {}",
        "diff".dimmed(),
        f1.bold(),
        "vs".dimmed(),
        f2.bold(),
    );
    println!("  {}", divider().dimmed());

    if only_1.is_empty() && only_2.is_empty() {
        println!();
        println!(
            "  {}  {}",
            "[/]".bright_yellow(),
            "Files have identical keys.".dimmed(),
        );
        println!();
        println!("  {}", divider().dimmed());
        println!();
        return Ok(());
    }

    println!();

    // Only in file 1
    if !only_1.is_empty() {
        println!(
            "  {}  {} {}",
            "[+]".bright_yellow(),
            only_1.len().to_string().bold(),
            format!("{} only in {}", if only_1.len() == 1 { "key" } else { "keys" }, f1).dimmed(),
        );
        for k in &only_1 {
            println!("    {}  {}", "+".green(), k.bold());
        }
        println!();
    }

    // Only in file 2
    if !only_2.is_empty() {
        println!(
            "  {}  {} {}",
            "[+]".bright_yellow(),
            only_2.len().to_string().bold(),
            format!("{} only in {}", if only_2.len() == 1 { "key" } else { "keys" }, f2).dimmed(),
        );
        for k in &only_2 {
            println!("    {}  {}", "+".yellow(), k.bold());
        }
        println!();
    }

    // Shared
    if !shared.is_empty() {
        println!(
            "  {}  {} {}",
            "[=]".dimmed(),
            shared.len().to_string().bold(),
            format!("shared {}", if shared.len() == 1 { "key" } else { "keys" }).dimmed(),
        );
        for k in &shared {
            println!("    {}  {}", "=".dimmed(), k.dimmed());
        }
        println!();
    }

    println!("  {}", divider().dimmed());
    println!();

    Ok(())
}
