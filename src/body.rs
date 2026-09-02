//! 派生字段（规范 §8.1）：chars / body_start / body_end。
//!
//! 对有效与无效节点统一适用。body_start/body_end 为原始文件物理行号（1-based）。

use crate::lines::is_blank;
use crate::nodemeta::{PREFIX, SUFFIX};

/// 返回 (chars, body_start, body_end)。
pub fn compute(
    lines: &[String],
    title_line: usize,
    end_line: usize,
    in_code: &dyn Fn(usize) -> bool,
) -> (usize, Option<usize>, Option<usize>) {
    let start = title_line + 1;
    if start > end_line {
        return (0, None, None);
    }

    // 排除行：标题行之后第一个非空行，且为单行完整形态 mddag 注释。
    // 未闭合、跨行、近似前缀变体均不构成排除行。
    // （首个非空行不可能是围栏内容：围栏开启行本身先于其内部任何行出现。）
    let _ = in_code;
    let first_non_empty = (start..=end_line).find(|&i| !is_blank(&lines[i - 1]));
    let exclude = first_non_empty.filter(|&i| {
        let l = &lines[i - 1];
        l.starts_with(PREFIX) && l.ends_with(SUFFIX)
    });

    // 排除行是首个非空行，故其前（若有）全为空行；候选正文自排除行之后开始。
    let mut lo = match exclude {
        Some(e) => e + 1,
        None => start,
    };
    let mut hi = end_line;
    while lo <= hi && is_blank(&lines[lo - 1]) {
        lo += 1;
    }
    while lo <= hi && is_blank(&lines[hi - 1]) {
        hi -= 1;
    }
    if lo > hi {
        return (0, None, None);
    }

    // chars = 各行去行尾换行序列后按 U+000A 连接的 Unicode 码点数。
    let mut chars: usize = 0;
    for i in lo..=hi {
        chars += lines[i - 1].chars().count();
    }
    chars += hi - lo; // 连接用换行符
    (chars, Some(lo), Some(hi))
}
