use std::path::Path;

use super::patterns::{patterns_for_extension, PatternDef};
use super::reference::EnvReference;

/// Extract all environment variable references from a single file's content.
pub fn extract_env_references(
    content: &str,
    file_path: &Path,
    extension: &str,
) -> Vec<EnvReference> {
    let patterns = patterns_for_extension(extension);
    if patterns.is_empty() {
        return vec![];
    }

    let mut refs = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Skip commented lines
        let trimmed = line.trim();
        if is_comment(trimmed, extension) {
            continue;
        }

        for pat in &patterns {
            extract_from_line(line, line_num, file_path, pat, &mut refs);
        }
    }

    refs
}

fn extract_from_line(
    line: &str,
    line_num: usize,
    file_path: &Path,
    pat: &PatternDef,
    refs: &mut Vec<EnvReference>,
) {
    for cap in pat.regex.captures_iter(line) {
        if pat.capture_group == 0 {
            // Dynamic access — log warning but still record it
            let full_match = cap.get(0).unwrap();
            refs.push(EnvReference {
                key: "<dynamic>".to_string(),
                file: file_path.to_path_buf(),
                line: line_num,
                col: full_match.start() + 1,
                pattern_type: pat.pattern_type.clone(),
                language: pat.language.clone(),
            });
        } else if let Some(m) = cap.get(pat.capture_group) {
            refs.push(EnvReference {
                key: m.as_str().to_string(),
                file: file_path.to_path_buf(),
                line: line_num,
                col: m.start() + 1,
                pattern_type: pat.pattern_type.clone(),
                language: pat.language.clone(),
            });
        }
    }
}

fn is_comment(trimmed: &str, ext: &str) -> bool {
    match ext {
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "rs" => {
            trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*")
        }
        "py" => trimmed.starts_with('#'),
        "sh" | "bash" | "zsh" => trimmed.starts_with('#'),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_env_dot() {
        let refs = extract_env_references(
            "const url = process.env.DATABASE_URL;",
            Path::new("test.ts"),
            "ts",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].key, "DATABASE_URL");
    }

    #[test]
    fn test_process_env_bracket() {
        let refs = extract_env_references(
            r#"const key = process.env['API_KEY'];"#,
            Path::new("test.js"),
            "js",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].key, "API_KEY");
    }

    #[test]
    fn test_import_meta_env() {
        let refs = extract_env_references(
            "const base = import.meta.env.VITE_API_URL;",
            Path::new("test.ts"),
            "ts",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].key, "VITE_API_URL");
    }

    #[test]
    fn test_rust_env_macro() {
        let refs = extract_env_references(
            r#"let val = env!("CARGO_PKG_VERSION");"#,
            Path::new("test.rs"),
            "rs",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].key, "CARGO_PKG_VERSION");
    }

    #[test]
    fn test_rust_std_env_var() {
        let refs = extract_env_references(
            r#"let url = std::env::var("DATABASE_URL").unwrap();"#,
            Path::new("test.rs"),
            "rs",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].key, "DATABASE_URL");
    }

    #[test]
    fn test_python_os_getenv() {
        let refs = extract_env_references(
            r#"key = os.getenv("SECRET_KEY")"#,
            Path::new("test.py"),
            "py",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].key, "SECRET_KEY");
    }

    #[test]
    fn test_shell_var() {
        let refs = extract_env_references(
            "echo $HOME and ${DATABASE_URL}",
            Path::new("test.sh"),
            "sh",
        );
        assert_eq!(refs.len(), 2);
        let keys: Vec<&str> = refs.iter().map(|r| r.key.as_str()).collect();
        assert!(keys.contains(&"HOME"));
        assert!(keys.contains(&"DATABASE_URL"));
    }

    #[test]
    fn test_skips_commented_line() {
        let refs = extract_env_references(
            "// const url = process.env.DATABASE_URL;",
            Path::new("test.ts"),
            "ts",
        );
        assert!(refs.is_empty());
    }

    #[test]
    fn test_dynamic_access() {
        let refs = extract_env_references(
            "const val = process.env[someVar];",
            Path::new("test.js"),
            "js",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].key, "<dynamic>");
    }

    #[test]
    fn test_multiple_refs_same_line() {
        let refs = extract_env_references(
            "const a = process.env.FOO + process.env.BAR;",
            Path::new("test.js"),
            "js",
        );
        assert_eq!(refs.len(), 2);
    }
}
