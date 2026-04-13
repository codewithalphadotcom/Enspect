use anyhow::Result;
use std::fs;

const DEFAULT_CONFIG: &str = r#"[scan]
root = "."
extensions = ["ts", "tsx", "js", "jsx", "mjs", "cjs", "rs", "py", "sh"]
ignore_dirs = ["node_modules", ".git", "dist", "build", "target", ".next", "__pycache__"]
ignore_files = []
follow_symlinks = false

[env_files]
example_files = [".env.example", ".env.sample", ".env.template"]
local_files = [".env", ".env.local", ".env.development", ".env.production"]
check_git_tracking = true

[secrets]
enabled = true
entropy_threshold = 4.5
min_length_for_entropy_check = 16
check_patterns = true
custom_patterns = []

[output]
format = "pretty"
color = "auto"
show_unused = true
show_empty = true
fail_on = ["missing", "secret"]

[git]
enabled = true
check_history = false

[audit]
allowed_missing = []
"#;

const DEFAULT_IGNORE: &str = r#"# Enspect ignore file (same syntax as .gitignore)
# Files and directories listed here will be skipped during scanning.

node_modules/
.git/
dist/
build/
target/
__pycache__/
.next/
coverage/
.tox/
*.min.js
"#;

pub fn run() -> Result<()> {
    let config_path = ".Enspect.toml";
    let ignore_path = ".Enspectignore";

    if std::path::Path::new(config_path).exists() {
        println!("{config_path} already exists — skipping.");
    } else {
        fs::write(config_path, DEFAULT_CONFIG)?;
        println!("Created {config_path}");
    }

    if std::path::Path::new(ignore_path).exists() {
        println!("{ignore_path} already exists — skipping.");
    } else {
        fs::write(ignore_path, DEFAULT_IGNORE)?;
        println!("Created {ignore_path}");
    }

    Ok(())
}
