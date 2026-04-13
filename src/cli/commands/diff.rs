use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Result;

use crate::cli::args::DiffArgs;
use crate::parser::dotenv;

pub fn run(args: &DiffArgs) -> Result<()> {
    let ef1 = dotenv::parse_env_file(&PathBuf::from(&args.file1))?;
    let ef2 = dotenv::parse_env_file(&PathBuf::from(&args.file2))?;

    let keys1: BTreeSet<String> = ef1.entries.iter().map(|e| e.key.clone()).collect();
    let keys2: BTreeSet<String> = ef2.entries.iter().map(|e| e.key.clone()).collect();

    let only_in_1: Vec<_> = keys1.difference(&keys2).collect();
    let only_in_2: Vec<_> = keys2.difference(&keys1).collect();
    let in_both: Vec<_> = keys1.intersection(&keys2).collect();

    println!("Comparing {} vs {}\n", args.file1, args.file2);

    if !only_in_1.is_empty() {
        println!("Only in {}:", args.file1);
        for k in &only_in_1 {
            println!("  + {k}");
        }
        println!();
    }

    if !only_in_2.is_empty() {
        println!("Only in {}:", args.file2);
        for k in &only_in_2 {
            println!("  + {k}");
        }
        println!();
    }

    if !in_both.is_empty() {
        println!("In both ({} keys):", in_both.len());
        for k in &in_both {
            println!("  = {k}");
        }
    }

    if only_in_1.is_empty() && only_in_2.is_empty() {
        println!("Files have identical keys.");
    }

    Ok(())
}
