//! slug derivation — inherited verbatim from v1.3 §4.2 (ADR-0002 D2).
//!
//! Deterministic: Unicode 15.1 Simple Lowercase Mapping (codepoint to
//! codepoint, no full case folding) -> non `[a-z0-9_-]` -> `-` -> collapse
//! runs -> trim leading/trailing `-` -> truncate to 64.

/// Derive the slug for a heading text. `None` when the result is empty
/// (the lode then has no usable id -> E-MISSING-ID).
pub fn slugify(title: &str) -> Option<String> {
    let mut out = String::with_capacity(title.len());
    for ch in title.chars() {
        for lower in ch.to_lowercase() {
            // Keep ASCII letters/digits/`_`/`-`; keep any non-ASCII char
            // (CJK etc.) as-is; everything else (spaces, punctuation) -> `-`.
            if lower.is_ascii_lowercase()
                || lower.is_ascii_digit()
                || lower == '_'
                || lower == '-'
                || !lower.is_ascii()
            {
                out.push(lower);
            } else {
                out.push('-');
            }
        }
    }
    // collapse runs, trim, truncate
    let mut collapsed = String::with_capacity(out.len());
    let mut prev_dash = false;
    for ch in out.chars() {
        if ch == '-' {
            if prev_dash {
                continue;
            }
            prev_dash = true;
        } else {
            prev_dash = false;
        }
        collapsed.push(ch);
    }
    let trimmed = collapsed.trim_matches('-');
    if trimmed.is_empty() {
        return None;
    }
    let mut s = trimmed.to_string();
    if s.chars().count() > 64 {
        s = s.chars().take(64).collect();
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_lowercase_and_spaces() {
        assert_eq!(slugify("Fluid Experiment Log"), Some("fluid-experiment-log".into()));
    }

    #[test]
    fn chinese_heading_slugs() {
        assert_eq!(slugify("流体实验记录"), Some("流体实验记录".into()));
    }

    #[test]
    fn collapses_and_trims_dashes() {
        assert_eq!(slugify("a  --  b--"), Some("a-b".into()));
        assert_eq!(slugify("--"), None);
        assert_eq!(slugify("   "), None);
    }

    #[test]
    fn truncates_to_64() {
        let long = "x".repeat(100);
        let s = slugify(&long).unwrap();
        assert_eq!(s.chars().count(), 64);
    }

    #[test]
    fn punctuation_becomes_dash() {
        assert_eq!(slugify("a/b?c"), Some("a-b-c".into()));
    }
}
