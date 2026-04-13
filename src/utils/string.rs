/// Mask a secret value: show first 4 chars + "***".
pub fn mask_value(value: &str) -> String {
    if value.len() <= 4 {
        return "***".to_string();
    }
    format!("{}***", &value[..4])
}

/// Check if a value looks like a placeholder.
pub fn is_placeholder(value: &str) -> bool {
    let lower = value.to_lowercase();
    let placeholders = [
        "your_secret_here",
        "your_api_key_here",
        "your_key_here",
        "changeme",
        "change_me",
        "replace_me",
        "replaceme",
        "xxx",
        "todo",
        "fixme",
        "your-key",
        "your-secret",
        "your-api-key",
        "placeholder",
        "insert_here",
        "update_me",
        "set_me",
    ];
    if placeholders.iter().any(|p| lower == *p) {
        return true;
    }
    // Check for patterns like <your-key>, <replace-me>
    if lower.starts_with('<') && lower.ends_with('>') {
        return true;
    }
    false
}
