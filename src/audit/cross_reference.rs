use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::config::Config;
use crate::parser::EnvFile;
use crate::scanner::EnvReference;

use super::findings::Finding;

/// Cross-reference code references against env files and shell environment.
pub fn cross_reference(
    references: &[EnvReference],
    env_files: &[EnvFile],
    shell_env: &HashMap<String, String>,
    config: &Config,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Build lookup sets
    let mut refs_by_key: HashMap<String, Vec<&EnvReference>> = HashMap::new();
    for r in references {
        if r.key == "<dynamic>" {
            continue;
        }
        refs_by_key.entry(r.key.clone()).or_default().push(r);
    }

    let all_referenced: HashSet<&str> = refs_by_key.keys().map(|s| s.as_str()).collect();

    let mut defined_in_local: HashMap<&str, Vec<PathBuf>> = HashMap::new();
    let mut defined_in_example: HashSet<String> = HashSet::new();
    let mut all_defined: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

    let allowed_missing: HashSet<&str> = config
        .audit
        .allowed_missing
        .iter()
        .map(|s| s.as_str())
        .collect();

    for ef in env_files {
        for entry in &ef.entries {
            all_defined
                .entry(entry.key.clone())
                .or_default()
                .push((ef.path.clone(), entry.line));

            if ef.is_example() {
                defined_in_example.insert(entry.key.clone());
            } else {
                defined_in_local
                    .entry(entry.key.as_str())
                    .or_default()
                    .push(ef.path.clone());
            }
        }
    }

    let shell_keys: HashSet<&str> = shell_env.keys().map(|s| s.as_str()).collect();

    // MISSING: referenced in code but not in any local .env or shell
    for key in &all_referenced {
        if allowed_missing.contains(*key) {
            continue;
        }
        let in_local = defined_in_local.contains_key(*key);
        let in_shell = shell_keys.contains(*key);
        if !in_local && !in_shell {
            let refs = refs_by_key[*key]
                .iter()
                .map(|r| (*r).clone())
                .collect();
            findings.push(Finding::Missing {
                key: key.to_string(),
                references: refs,
                in_shell: false,
            });
        }
    }

    // UNDOCUMENTED: in code + in local but not in example
    for key in &all_referenced {
        let in_local = defined_in_local.contains_key(*key);
        let in_example = defined_in_example.contains(*key);
        if in_local && !in_example {
            let defined_paths = defined_in_local[*key].clone();
            let refs = refs_by_key[*key]
                .iter()
                .map(|r| (*r).clone())
                .collect();
            findings.push(Finding::Undocumented {
                key: key.to_string(),
                defined_in: defined_paths,
                references: refs,
            });
        }
    }

    // UNUSED: defined in env files but never referenced in code
    if config.output.show_unused {
        for (key, locations) in &all_defined {
            if !all_referenced.contains(key.as_str()) {
                findings.push(Finding::Unused {
                    key: key.clone(),
                    defined_in: locations.clone(),
                    last_seen_in_git: None,
                });
            }
        }
    }

    // EMPTY / PLACEHOLDER: entries with empty or placeholder values in non-example files
    if config.output.show_empty {
        for ef in env_files {
            if ef.is_example() {
                continue; // empty values in example files are expected
            }
            for entry in &ef.entries {
                if entry.is_empty || entry.is_placeholder {
                    findings.push(Finding::Empty {
                        key: entry.key.clone(),
                        file: ef.path.clone(),
                        line: entry.line,
                        is_placeholder: entry.is_placeholder,
                    });
                }
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::envfile::{EnvEntry, EnvFile, EnvFileType};
    use crate::scanner::reference::{Language, PatternType};
    use std::path::PathBuf;

    fn make_ref(key: &str, file: &str, line: usize) -> EnvReference {
        EnvReference {
            key: key.to_string(),
            file: PathBuf::from(file),
            line,
            col: 1,
            pattern_type: PatternType::ProcessEnv,
            language: Language::JavaScript,
        }
    }

    fn make_env_file(name: &str, keys: &[(&str, &str)], is_example: bool) -> EnvFile {
        let path = PathBuf::from(name);
        let file_type = if is_example {
            EnvFileType::Example
        } else {
            EnvFileType::Local
        };
        let entries = keys
            .iter()
            .enumerate()
            .map(|(i, (k, v))| EnvEntry::new(k.to_string(), v.to_string(), i + 1))
            .collect();
        EnvFile {
            path,
            file_type,
            entries,
            is_git_tracked: false,
            parse_errors: vec![],
        }
    }

    #[test]
    fn test_missing_variable() {
        let refs = vec![make_ref("DATABASE_URL", "src/db.ts", 3)];
        let env_files = vec![make_env_file(".env.example", &[("OTHER", "val")], true)];
        let shell = HashMap::new();
        let config = Config::default();
        let findings = cross_reference(&refs, &env_files, &shell, &config);
        assert!(findings
            .iter()
            .any(|f| matches!(f, Finding::Missing { key, .. } if key == "DATABASE_URL")));
    }

    #[test]
    fn test_undocumented_variable() {
        let refs = vec![make_ref("API_KEY", "src/api.ts", 5)];
        let env_files = vec![
            make_env_file(".env.local", &[("API_KEY", "secret123")], false),
            make_env_file(".env.example", &[("OTHER", "")], true),
        ];
        let shell = HashMap::new();
        let config = Config::default();
        let findings = cross_reference(&refs, &env_files, &shell, &config);
        assert!(findings
            .iter()
            .any(|f| matches!(f, Finding::Undocumented { key, .. } if key == "API_KEY")));
    }

    #[test]
    fn test_unused_variable() {
        let refs = vec![];
        let env_files = vec![make_env_file(
            ".env.local",
            &[("OLD_REDIS_URL", "redis://...")],
            false,
        )];
        let shell = HashMap::new();
        let config = Config::default();
        let findings = cross_reference(&refs, &env_files, &shell, &config);
        assert!(findings
            .iter()
            .any(|f| matches!(f, Finding::Unused { key, .. } if key == "OLD_REDIS_URL")));
    }

    #[test]
    fn test_shell_env_satisfies_missing() {
        let refs = vec![make_ref("PATH_VAR", "src/app.ts", 1)];
        let env_files = vec![];
        let mut shell = HashMap::new();
        shell.insert("PATH_VAR".to_string(), "/usr/bin".to_string());
        let config = Config::default();
        let findings = cross_reference(&refs, &env_files, &shell, &config);
        assert!(!findings
            .iter()
            .any(|f| matches!(f, Finding::Missing { key, .. } if key == "PATH_VAR")));
    }

    #[test]
    fn test_empty_value_detected() {
        let refs = vec![];
        let env_files = vec![make_env_file(".env.local", &[("SECRET", "")], false)];
        let shell = HashMap::new();
        let config = Config::default();
        let findings = cross_reference(&refs, &env_files, &shell, &config);
        assert!(findings
            .iter()
            .any(|f| matches!(f, Finding::Empty { key, .. } if key == "SECRET")));
    }
}
