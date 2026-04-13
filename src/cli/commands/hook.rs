use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::cli::args::HookAction;

const HOOK_SCRIPT: &str = r#"#!/bin/sh
# Enspect pre-commit hook
enspect audit --fail-on missing,secret --quiet
exit $?
"#;

pub fn run(action: &HookAction) -> Result<()> {
    let git_dir = find_git_dir()?;
    let hooks_dir = git_dir.join("hooks");
    let hook_path = hooks_dir.join("pre-commit");

    match action {
        HookAction::Install => {
            fs::create_dir_all(&hooks_dir)
                .context("Failed to create hooks directory")?;

            if hook_path.exists() {
                let existing = fs::read_to_string(&hook_path)?;
                if existing.contains("enspect") {
                    println!("Enspect hook is already installed.");
                    return Ok(());
                }
                // Back up existing hook
                let backup = hooks_dir.join("pre-commit.backup");
                fs::rename(&hook_path, &backup)?;
                println!("Backed up existing pre-commit hook to pre-commit.backup");
            }

            fs::write(&hook_path, HOOK_SCRIPT)?;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
            println!("Installed Enspect pre-commit hook at {}", hook_path.display());
        }
        HookAction::Uninstall => {
            if !hook_path.exists() {
                println!("No pre-commit hook found.");
                return Ok(());
            }
            let content = fs::read_to_string(&hook_path)?;
            if !content.contains("enspect") {
                println!("Pre-commit hook exists but was not installed by Enspect.");
                return Ok(());
            }
            fs::remove_file(&hook_path)?;

            // Restore backup if exists
            let backup = hooks_dir.join("pre-commit.backup");
            if backup.exists() {
                fs::rename(&backup, &hook_path)?;
                println!("Restored previous pre-commit hook from backup.");
            }
            println!("Uninstalled Enspect pre-commit hook.");
        }
        HookAction::Run => {
            // Run audit in hook mode
            let args = crate::cli::args::AuditArgs {
                root: ".".to_string(),
                config: None,
                format: "pretty".to_string(),
                fail_on: Some("missing,secret".to_string()),
                no_color: false,
                quiet: true,
                verbose: false,
                no_secrets: false,
                no_git: false,
                no_unused: true,
                no_empty: true,
                show_values: false,
                ci: false,
                show_all: false,
            };
            let exit_code = crate::cli::commands::audit::run(&args)?;
            std::process::exit(exit_code as i32);
        }
    }

    Ok(())
}

fn find_git_dir() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let git_dir = dir.join(".git");
        if git_dir.is_dir() {
            return Ok(git_dir);
        }
        if !dir.pop() {
            bail!("Not a git repository (or any parent up to root)");
        }
    }
}
