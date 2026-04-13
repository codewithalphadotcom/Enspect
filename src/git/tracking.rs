use std::path::Path;
use std::process::Command;

/// Check if a file is tracked by git (in the index).
pub fn is_git_tracked(file: &Path, repo_root: &Path) -> bool {
    let output = Command::new("git")
        .arg("ls-files")
        .arg("--cached")
        .arg(file)
        .current_dir(repo_root)
        .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            o.status.success() && !stdout.trim().is_empty()
        }
        Err(_) => false,
    }
}

/// Check which .env files are tracked by git.
pub fn find_tracked_env_files(env_file_paths: &[&Path], repo_root: &Path) -> Vec<std::path::PathBuf> {
    env_file_paths
        .iter()
        .filter(|p| is_git_tracked(p, repo_root))
        .map(|p| p.to_path_buf())
        .collect()
}
