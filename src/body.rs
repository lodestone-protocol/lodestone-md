//! Derived fields (spec §8.1): chars / body_start / body_end.
//!
//! Computed uniformly for valid and invalid nodes. body_start/body_end are
//! physical line numbers in the source file (1-based).

use crate::lines::is_blank;
use crate::nodemeta::{PREFIX, SUFFIX};

/// Returns (chars, body_start, body_end).
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

    // Exclusion line: the first non-empty line after the heading, when it is a
    // single-line, well-formed mddag comment. Unclosed, multi-line, and
    // near-prefix variants are not exclusion lines.
    // (The first non-empty line cannot be fenced content: a fence opener would
    // precede any inner line.)
    let _ = in_code;
    let first_non_empty = (start..=end_line).find(|&i| !is_blank(&lines[i - 1]));
    let exclude = first_non_empty.filter(|&i| {
        let l = &lines[i - 1];
        l.starts_with(PREFIX) && l.ends_with(SUFFIX)
    });

    // The exclusion line is the first non-empty line, so any lines before it
    // are blank; the candidate body begins after it.
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

    // chars = count of Unicode scalar values in the body lines joined by
    // U+000A (line endings stripped from each line).
    let mut chars: usize = 0;
    for i in lo..=hi {
        chars += lines[i - 1].chars().count();
    }
    chars += hi - lo; // joining newlines
    (chars, Some(lo), Some(hi))
}
