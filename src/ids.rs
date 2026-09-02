//! 节点标识（规范 §4.2）：slug 派生与 id 字符合法性。

pub const MAX_ID_LEN: usize = 64;

/// id 字符集为 `[a-z0-9_-]`，长度 ≤ 64。
pub fn is_valid_declared_id(id: &str) -> bool {
    !id.is_empty()
        && id.chars().count() <= MAX_ID_LEN
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// 由标题文本按 §4.2 确定性算法派生 slug。
///
/// 小写映射：以 `char::to_lowercase()`（全小写映射）取首码位近似
/// Unicode 15.1 Simple Lowercase Mapping（ADR-0001，偏差个案以测试收敛）。
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
    // 连续 `-` 压缩为单个
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
    // 去除首尾 `-`，超过 64 字符截断
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
        // U+0130 的 Simple Lowercase Mapping 为 U+0069
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
