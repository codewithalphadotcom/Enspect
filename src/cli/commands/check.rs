use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::cli::args::CheckArgs;
use crate::config::Config;
use crate::detector::entropy::shannon_entropy;
use crate::parser::dotenv;
use crate::scanner::walker;

pub fn run(args: &CheckArgs) -> Result<()> {
    let root = PathBuf::from(&args.root).canonicalize()?;
    let config = Config::load(&root)?;
    let var = &args.var_name;

    println!("Checking variable: {var}\n");

    // Scan source
    let scan = walker::scan_directory(&root, &config)?;
    let refs: Vec<_> = scan.references.iter().filter(|r| r.key == *var).collect();

    if refs.is_empty() {
        println!("  Not referenced in any source files.");
    } else {
        println!("  Referenced in {} location(s):", refs.len());
        for r in &refs {
            println!("    {}:{}", r.file.display(), r.line);
        }
    }

    // Check env files
    let env_files = dotenv::find_env_files(&root)?;
    let mut found_in_env = false;
    for ef in &env_files {
        for entry in &ef.entries {
            if entry.key == *var {
                println!(
                    "\n  Defined in: {}:{} (value: {})",
                    ef.path.display(),
                    entry.line,
                    entry.value_masked,
                );
                let entropy = shannon_entropy(&entry.value);
                println!("    Entropy: {entropy:.2} bits/char");
                if entry.is_empty {
                    println!("    ⚠ Value is empty");
                }
                if entry.is_placeholder {
                    println!("    ⚠ Value appears to be a placeholder");
                }
                found_in_env = true;
            }
        }
    }
    if !found_in_env {
        println!("\n  Not defined in any .env file.");
    }

    // Check shell
    let shell_env: HashMap<String, String> = std::env::vars().collect();
    if shell_env.contains_key(var) {
        println!("\n  Present in current shell environment.");
    } else {
        println!("\n  Not present in current shell environment.");
    }

    Ok(())
}
