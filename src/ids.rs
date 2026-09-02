//! Node identity (spec §4.2): slug derivation and declared-id legality.

pub const MAX_ID_LEN: usize = 64;

/// The id charset is `[a-z0-9_-]`, length ≤ 64.
pub fn is_valid_declared_id(id: &str) -> bool {
    !id.is_empty()
        && id.chars().count() <= MAX_ID_LEN
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Derives a slug from the heading text by the deterministic algorithm of §4.2.
///
/// Lowercasing approximates Unicode 15.1 Simple Lowercase Mapping by taking
/// the first code point of `char::to_lowercase()` (full mapping); divergent
/// cases are pinned down by tests (ADR-0001).
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for ch in title.chars() {
        let lower = ch.to_lowercase().next().unwrap_or(ch);
        if lower.is_ascii_lowercase() || lower.is_ascii_digit() || lower == '_' || lower == '-' {
            out.push(lower);
        } else {
            out.push('-');
        }
    }
    // Collapse runs of `-` into a single one
    let mut collapsed = String::with_capacity(out.len());
    let mut prev_dash = false;
    for c in out.chars() {
        if c == '-' {
            if !prev_dash {
                collapsed.push('-');
            }
            prev_dash = true;
        } else {
            collapsed.push(c);
            prev_dash = false;
        }
    }
    // Trim leading/trailing `-`; truncate past 64 chars
    collapsed
        .trim_matches('-')
        .chars()
        .take(MAX_ID_LEN)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_rules() {
        assert_eq!(slug("Release Notes v2!"), "release-notes-v2");
        assert_eq!(slug("hello__world"), "hello__world");
        assert_eq!(slug("--a---b--"), "a-b");
        assert_eq!(slug(""), "");
        assert_eq!(slug("日本語"), "");
    }

    #[test]
    fn slug_truncates_to_64() {
        let long = "a".repeat(80);
        assert_eq!(slug(&long).chars().count(), 64);
    }

    #[test]
    fn turkish_dotted_i() {
        // U+0130 Simple Lowercase Mapping is U+0069
        assert_eq!(slug("\u{0130}stanbul"), "istanbul");
    }

    #[test]
    fn declared_id_charset() {
        assert!(is_valid_declared_id("exp-01_config"));
        assert!(!is_valid_declared_id("Upper"));
        assert!(!is_valid_declared_id(""));
        assert!(!is_valid_declared_id(&"a".repeat(65)));
    }
}
