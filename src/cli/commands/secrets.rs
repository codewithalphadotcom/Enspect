use std::path::PathBuf;

use anyhow::Result;

use crate::cli::args::SecretsArgs;
use crate::config::Config;
use crate::detector::secret::detect_secrets;
use crate::parser::dotenv;

pub fn run(args: &SecretsArgs) -> Result<()> {
    let root = PathBuf::from(&args.root).canonicalize()?;
    let config = Config::load(&root)?;

    let env_files = if let Some(ref path) = args.path {
        vec![dotenv::parse_env_file(&PathBuf::from(path))?]
    } else {
        dotenv::find_env_files(&root)?
    };

    let findings = detect_secrets(&env_files, &config);

    if findings.is_empty() {
        println!("No secrets detected.");
    } else {
        println!("Found {} potential secret(s):\n", findings.len());
        for f in &findings {
            println!(
                "  {}:{} — {} [{}]",
                f.file.display(),
                f.line,
                f.key,
                f.severity,
            );
            println!("    Reason: {}", f.reason);
            println!("    Value:  {}", f.value_preview);
            println!();
        }
    }

    Ok(())
}
