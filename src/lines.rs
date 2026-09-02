//! 物理行切分（规范 §8.1）：LF / CRLF / CR 均视为一行结束。
//!
//! 文件末尾的换行符终止末行而不产生新空行；BOM 由调用方先行剥离。

pub fn split_lines(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\n' => out.push(std::mem::take(&mut cur)),
            '\r' => {
                if it.peek() == Some(&'\n') {
                    it.next();
                }
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// CommonMark 空行定义：仅含空格 / 制表符。
pub fn is_blank(line: &str) -> bool {
    line.chars().all(|c| c == ' ' || c == '\t')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_endings() {
        assert_eq!(split_lines("a\r\nb\nc\rd"), vec!["a", "b", "c", "d"]);
        assert_eq!(split_lines("a\n"), vec!["a"]);
        assert_eq!(split_lines(""), Vec::<String>::new());
        assert_eq!(split_lines("\n"), vec![""]);
    }

    #[test]
    fn blank_detection() {
        assert!(is_blank(""));
        assert!(is_blank("  \t "));
        assert!(!is_blank(" \u{00A0}"));
    }
}
