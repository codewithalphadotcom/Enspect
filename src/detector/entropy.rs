use std::collections::HashMap;

/// Calculate Shannon entropy (bits per character) of a string.
pub fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }

    let len = s.len() as f64;
    let mut freq: HashMap<char, usize> = HashMap::new();
    for c in s.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }

    freq.values()
        .map(|&count| {
            let p = count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(shannon_entropy(""), 0.0);
    }

    #[test]
    fn test_single_char() {
        assert_eq!(shannon_entropy("aaaa"), 0.0);
    }

    #[test]
    fn test_low_entropy_word() {
        let e = shannon_entropy("password");
        assert!(e < 3.5, "expected < 3.5, got {e}");
    }

    #[test]
    fn test_high_entropy_api_key() {
        let e = shannon_entropy("sk_live_51Abc2Def3Ghi4Jkl5Mno6Pqr");
        assert!(e > 4.0, "expected > 4.0, got {e}");
    }

    #[test]
    fn test_base64_high_entropy() {
        let e = shannon_entropy("dGhpcyBpcyBhIHRlc3Qgc3RyaW5nIHdpdGggaGlnaCBlbnRyb3B5IQ==");
        assert!(e > 4.0, "expected > 4.0, got {e}");
    }

    #[test]
    fn test_hex_string_entropy() {
        let e = shannon_entropy("a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6");
        assert!(e > 3.5, "expected > 3.5, got {e}");
    }
}
