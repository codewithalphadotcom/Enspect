use crate::audit::findings::{SecretFinding, Severity};
use crate::config::Config;
use crate::parser::EnvFile;
use crate::utils::string::mask_value;

use super::entropy::shannon_entropy;
use super::patterns::known_secret_patterns;
use super::placeholder::is_placeholder;

/// Run secret detection across all provided .env files.
pub fn detect_secrets(env_files: &[EnvFile], config: &Config) -> Vec<SecretFinding> {
    if !config.secrets.enabled {
        return vec![];
    }

    let patterns = known_secret_patterns();
    let mut findings = Vec::new();

    for ef in env_files {
        for entry in &ef.entries {
            if entry.value.is_empty() {
                continue;
            }
            if is_placeholder(&entry.value) {
                continue;
            }

            let val = &entry.value;

            // Pattern-based detection
            if config.secrets.check_patterns {
                for pat in &patterns {
                    if pat.regex.is_match(val) {
                        findings.push(SecretFinding {
                            key: entry.key.clone(),
                            file: ef.path.clone(),
                            line: entry.line,
                            severity: Severity::Critical,
                            reason: format!("Matches {} pattern", pat.name),
                            pattern_name: Some(pat.name.to_string()),
                            entropy: shannon_entropy(val),
                            value_preview: mask_value(val),
                        });
                        break; // Only report first matching pattern
                    }
                }
            }

            // Entropy-based detection (only if no pattern match found for this key)
            if !findings.iter().any(|f: &SecretFinding| f.key == entry.key && f.file == ef.path) {
                let entropy = shannon_entropy(val);
                let len = val.len();

                if len >= config.secrets.min_length_for_entropy_check
                    && entropy >= config.secrets.entropy_threshold
                {
                    let severity = if len >= 32 {
                        Severity::High
                    } else {
                        Severity::Medium
                    };
                    findings.push(SecretFinding {
                        key: entry.key.clone(),
                        file: ef.path.clone(),
                        line: entry.line,
                        severity,
                        reason: format!(
                            "Shannon entropy {:.1} bits/char, length {len}",
                            entropy
                        ),
                        pattern_name: None,
                        entropy,
                        value_preview: mask_value(val),
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
    use std::path::PathBuf;

    fn make_env_file(entries: Vec<(&str, &str)>) -> EnvFile {
        let entries = entries
            .into_iter()
            .enumerate()
            .map(|(i, (k, v))| EnvEntry::new(k.to_string(), v.to_string(), i + 1))
            .collect();
        EnvFile {
            path: PathBuf::from(".env.local"),
            file_type: EnvFileType::Local,
            entries,
            is_git_tracked: false,
            parse_errors: vec![],
        }
    }

    #[test]
    fn test_detects_stripe_key() {
        let ef = make_env_file(vec![("STRIPE_KEY", "sk_live_51Abc2Def3Ghi4Jkl5Mno6Pqr")]);
        let config = Config::default();
        let findings = detect_secrets(&[ef], &config);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_skips_placeholder() {
        let ef = make_env_file(vec![("API_KEY", "changeme")]);
        let config = Config::default();
        let findings = detect_secrets(&[ef], &config);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_masks_value_in_output() {
        let ef = make_env_file(vec![("STRIPE_KEY", "sk_live_51Abc2Def3Ghi4Jkl5Mno6Pqr")]);
        let config = Config::default();
        let findings = detect_secrets(&[ef], &config);
        assert!(!findings[0].value_preview.contains("51Abc2Def"));
        assert!(findings[0].value_preview.starts_with("sk_l"));
    }

    #[test]
    fn test_high_entropy_detection() {
        // A random-looking string that doesn't match patterns but has high entropy
        let ef = make_env_file(vec![(
            "SECRET",
            "x8Kp2mNq5rTv7wYz0AcEfHjLnPsUvXz",
        )]);
        let config = Config::default();
        let findings = detect_secrets(&[ef], &config);
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_disabled_secrets() {
        let ef = make_env_file(vec![("STRIPE_KEY", "sk_live_51Abc2Def3Ghi4Jkl5Mno6Pqr")]);
        let mut config = Config::default();
        config.secrets.enabled = false;
        let findings = detect_secrets(&[ef], &config);
        assert!(findings.is_empty());
    }
}
