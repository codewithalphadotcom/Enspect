use std::fs;
use std::io;
use std::path::Path;

/// Check if a file is binary by looking for null bytes in the first 8KB.
pub fn is_binary_file(path: &Path) -> io::Result<bool> {
    let bytes = fs::read(path)?;
    let check_len = bytes.len().min(8192);
    Ok(bytes[..check_len].contains(&0))
}

/// Strip UTF-8 BOM from the beginning of a string.
pub fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{FEFF}').unwrap_or(s)
}
