use std::path::Path;

/// Check if a file path is covered by .gitignore rules.
/// We do this by reading .gitignore and checking for patterns.
pub fn is_in_gitignore(file_name: &str, repo_root: &Path) -> bool {
    let gitignore_path = repo_root.join(".gitignore");
    if !gitignore_path.exists() {
        return false;
    }

    let content = match std::fs::read_to_string(&gitignore_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Simple pattern matching — covers common cases
        if line == file_name {
            return true;
        }
        // Glob-style: .env* or .env.local
        if let Some(prefix) = line.strip_suffix('*') {
            if file_name.starts_with(prefix) {
                return true;
            }
        }
    }

    false
}

/// Check which .env files are NOT in .gitignore.
pub fn find_missing_from_gitignore(env_file_names: &[&str], repo_root: &Path) -> Vec<String> {
    env_file_names
        .iter()
        .filter(|name| !is_in_gitignore(name, repo_root))
        .map(|name| name.to_string())
        .collect()
}
