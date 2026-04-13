use std::path::PathBuf;

use anyhow::Result;

use crate::cli::args::ScanArgs;
use crate::config::Config;
use crate::scanner::walker;

pub fn run(args: &ScanArgs) -> Result<()> {
    let root = PathBuf::from(&args.root).canonicalize()?;
    let config = Config::load(&root)?;

    let result = walker::scan_directory(&root, &config)?;

    if args.json {
        let json = serde_json::to_string_pretty(&result.references)?;
        println!("{json}");
    } else {
        println!("Scanned {} files", result.files_scanned);
        println!("Found {} env var references:\n", result.references.len());

        let mut sorted = result.references;
        sorted.sort_by(|a, b| a.key.cmp(&b.key));

        let mut current_key = String::new();
        for r in &sorted {
            if r.key != current_key {
                if !current_key.is_empty() {
                    println!();
                }
                println!("  {}", r.key);
                current_key = r.key.clone();
            }
            println!(
                "    {}:{} ({:?})",
                r.file.display(),
                r.line,
                r.pattern_type,
            );
        }

        if !result.dynamic_warnings.is_empty() {
            println!("\nDynamic access warnings:");
            for (file, line) in &result.dynamic_warnings {
                println!("  {}:{} — cannot statically determine variable name", file.display(), line);
            }
        }
    }

    Ok(())
}
