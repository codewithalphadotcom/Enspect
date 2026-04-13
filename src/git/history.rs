use std::path::Path;
use std::process::Command;

#[allow(dead_code)]
/// Check if .env files have ever been committed in git history.
pub fn env_files_in_history(repo_root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .args(["log", "--all", "--full-history", "--diff-filter=A", "--name-only", "--pretty=format:", "--", "*.env*"])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.trim().to_string())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect()
        }
        _ => vec![],
    }
}
