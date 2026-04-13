use std::path::Path;

use anyhow::Result;

use crate::utils::fs::strip_bom;

use super::envfile::{EnvEntry, EnvFile};

/// Parse a .env file into an EnvFile struct.
pub fn parse_env_file(path: &Path) -> Result<EnvFile> {
    let content = std::fs::read_to_string(path)?;
    let content = strip_bom(content.as_str());
    let mut env_file = EnvFile::new(path.to_path_buf());

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line_num = i + 1;
        let line = lines[i].trim_end_matches('\r');

        // Skip empty lines and comments
        if line.trim().is_empty() || line.trim().starts_with('#') {
            i += 1;
            continue;
        }

        // Strip optional `export ` prefix
        let line = if let Some(rest) = line.strip_prefix("export ") {
            rest
        } else {
            line
        };

        // Find the = separator
        let Some(eq_pos) = line.find('=') else {
            env_file
                .parse_errors
                .push(format!("Line {line_num}: no '=' found"));
            i += 1;
            continue;
        };

        let key = line[..eq_pos].trim().to_string();
        if key.is_empty() {
            env_file
                .parse_errors
                .push(format!("Line {line_num}: empty key"));
            i += 1;
            continue;
        }

        let raw_value = &line[eq_pos + 1..];
        let value = parse_value(raw_value, &lines, &mut i);

        env_file.entries.push(EnvEntry::new(key, value, line_num));
        i += 1;
    }

    Ok(env_file)
}

/// Parse the value portion of a KEY=value line.
/// Handles quoted strings, multiline continuations, and bare values.
fn parse_value(raw: &str, lines: &[&str], current_line: &mut usize) -> String {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return String::new();
    }

    // Double-quoted value
    if trimmed.starts_with('"') {
        return parse_quoted_value(trimmed, '"', lines, current_line);
    }

    // Single-quoted value
    if trimmed.starts_with('\'') {
        return parse_quoted_value(trimmed, '\'', lines, current_line);
    }

    // Bare value — handle backslash continuations
    let mut value = String::new();
    let mut remaining = trimmed.to_string();

    loop {
        if let Some(stripped) = remaining.strip_suffix('\\') {
            value.push_str(stripped);
            *current_line += 1;
            if *current_line < lines.len() {
                remaining = lines[*current_line].trim_end_matches('\r').to_string();
            } else {
                break;
            }
        } else {
            // Strip inline comments for bare values
            let val = if let Some(comment_pos) = remaining.find(" #") {
                &remaining[..comment_pos]
            } else {
                &remaining
            };
            value.push_str(val.trim());
            break;
        }
    }

    value
}

/// Parse a quoted value, handling multiline and escape sequences.
fn parse_quoted_value(
    trimmed: &str,
    quote: char,
    lines: &[&str],
    current_line: &mut usize,
) -> String {
    // Remove the opening quote
    let after_quote = &trimmed[1..];
    let mut value = String::new();
    let mut remaining = after_quote.to_string();

    loop {
        if let Some(end_pos) = find_unescaped_quote(&remaining, quote) {
            value.push_str(&remaining[..end_pos]);
            break;
        } else {
            value.push_str(&remaining);
            value.push('\n');
            *current_line += 1;
            if *current_line < lines.len() {
                remaining = lines[*current_line].trim_end_matches('\r').to_string();
            } else {
                break;
            }
        }
    }

    // Process escape sequences for double-quoted strings
    if quote == '"' {
        value = value
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
    }

    value
}

/// Find the position of the closing quote that isn't escaped.
fn find_unescaped_quote(s: &str, quote: char) -> Option<usize> {
    let mut chars = s.char_indices();
    let mut escaped = false;

    while let Some((i, c)) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == quote {
            return Some(i);
        }
    }
    None
}

/// Discover all .env* files in a directory (non-recursive, just the root).
pub fn find_env_files(root: &Path) -> Result<Vec<EnvFile>> {
    let mut env_files = Vec::new();

    if !root.is_dir() {
        return Ok(env_files);
    }

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };
        if file_name.starts_with(".env") {
            match parse_env_file(&path) {
                Ok(ef) => env_files.push(ef),
                Err(e) => {
                    eprintln!("Warning: failed to parse {}: {e}", path.display());
                }
            }
        }
    }

    Ok(env_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn parse_string(content: &str) -> EnvFile {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{content}").unwrap();
        parse_env_file(tmp.path()).unwrap()
    }

    #[test]
    fn test_simple_key_value() {
        let ef = parse_string("FOO=bar\nBAZ=123\n");
        assert_eq!(ef.entries.len(), 2);
        assert_eq!(ef.entries[0].key, "FOO");
        assert_eq!(ef.entries[0].value, "bar");
        assert_eq!(ef.entries[1].key, "BAZ");
        assert_eq!(ef.entries[1].value, "123");
    }

    #[test]
    fn test_quoted_values() {
        let ef = parse_string("A=\"hello world\"\nB='single quoted'\n");
        assert_eq!(ef.entries[0].value, "hello world");
        assert_eq!(ef.entries[1].value, "single quoted");
    }

    #[test]
    fn test_empty_value() {
        let ef = parse_string("EMPTY=\n");
        assert_eq!(ef.entries[0].value, "");
        assert!(ef.entries[0].is_empty);
    }

    #[test]
    fn test_comments_skipped() {
        let ef = parse_string("# this is a comment\nKEY=val\n");
        assert_eq!(ef.entries.len(), 1);
        assert_eq!(ef.entries[0].key, "KEY");
    }

    #[test]
    fn test_export_prefix() {
        let ef = parse_string("export MY_VAR=hello\n");
        assert_eq!(ef.entries[0].key, "MY_VAR");
        assert_eq!(ef.entries[0].value, "hello");
    }

    #[test]
    fn test_placeholder_detection() {
        let ef = parse_string("SECRET=changeme\nAPI=<your-key>\n");
        assert!(ef.entries[0].is_placeholder);
        assert!(ef.entries[1].is_placeholder);
    }

    #[test]
    fn test_value_masking() {
        let ef = parse_string("KEY=sk_live_1234567890abcdef\n");
        assert_eq!(ef.entries[0].value_masked, "sk_l***");
    }

    #[test]
    fn test_crlf_handling() {
        let ef = parse_string("A=1\r\nB=2\r\n");
        assert_eq!(ef.entries.len(), 2);
        assert_eq!(ef.entries[0].value, "1");
    }

    #[test]
    fn test_utf8_bom() {
        let ef = parse_string("\u{FEFF}KEY=val\n");
        assert_eq!(ef.entries[0].key, "KEY");
    }

    #[test]
    fn test_multiline_value() {
        let ef = parse_string("URL=\"postgresql://user:pass@host:5432/db?\nsslmode=require\"\n");
        assert!(ef.entries[0].value.contains("sslmode=require"));
    }

    #[test]
    fn test_empty_file() {
        let ef = parse_string("");
        assert!(ef.entries.is_empty());
    }
}
