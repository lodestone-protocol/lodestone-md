//! Document model (v2.0-draft §2/§3): lodes, magnetic lines, sediment.
//! Pure data — built by `scan`, projected by `project`, mutated by `ops`.

use crate::diag::Diag;

/// Interaction-mode status of a lode (v2.0-draft §3.3): draft -> converged
/// -> aligned, one-way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Draft,
    Converged,
    Aligned,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Draft => "draft",
            Status::Converged => "converged",
            Status::Aligned => "aligned",
        }
    }
    /// One-way state machine; `None` = illegal transition.
    pub fn advance(self) -> Option<Status> {
        match self {
            Status::Draft => Some(Status::Converged),
            Status::Converged => Some(Status::Aligned),
            Status::Aligned => None,
        }
    }
}

/// A magnetic line (edge): a link `[label](#target-slug)` in a lode body
/// whose target is another lode (v2.0-draft §3.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagneticLine {
    pub from: String,
    pub to: String,
    pub label: String,
    pub line: usize,
}

/// A cross-document reference `[label](path#slug)` (v2.0-draft §3.7).
/// The document layer only checks its shape (W-CROSS-DOC); target
/// resolution belongs to the library layer (mddag index).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossLink {
    pub from_slug: String,
    pub path: String,
    pub to_slug: String,
    pub label: String,
    pub line: usize,
}

/// One lodestone (node): a root-level `#` heading region (v2.0-draft §3.1).
/// Named after the magnet metaphor — the lodestone is the lode itself.
#[derive(Debug, Clone)]
pub struct Lodestone {
    pub slug: String,
    pub title: String,
    pub title_line: usize,
    pub status: Status,
    pub summary: Option<String>,
    /// 1-based line range of the lode region (title .. next lode / EOF).
    pub start_line: usize,
    pub end_line: usize,
    /// Sub-heading tree (lines of `##`/`###`... headings inside the lodestone).
    pub subheadings: Vec<(usize, usize, String)>, // (line, level, heading text)
    /// Magnetic lines found in the whole lode region.
    pub lines: Vec<MagneticLine>,
    /// Body lines (region minus title line, minus the status list block).
    pub body: Vec<usize>, // line numbers
}

/// The parsed document (v2.0-draft §2): activity zone + sediment zone.
/// Balls are called lodestones (磁石) — the protocol's native term.
#[derive(Debug, Clone)]
pub struct Doc {
    /// Raw document text (byte-identical copy) — L2 body extraction reads it.
    pub text: String,
    /// Optional document-level header metadata: `- session: <id>`,
    /// `- created: <date>` (values written by the consumer; protocol only
    /// defines the shape — zero hardcoded values).
    pub meta: Vec<(String, String)>,
    /// Cross-document references collected during scan (library layer input).
    pub cross_links: Vec<CrossLink>,
    pub lodestones: Vec<Lodestone>,
    /// Sediment zone: `# 沉淀区` region — converged bodies archived here.
    pub sediment: Option<Sediment>,
    pub diagnostics: Vec<Diag>,
}

/// The sediment zone (`# 沉淀区`, reserved heading — never a lode, never
/// referenced by magnetic lines, v2.0-draft §3.1/§3.5).
#[derive(Debug, Clone)]
pub struct Sediment {
    pub start_line: usize,
    pub end_line: usize,
    /// `## <slug>-full` entries (converged bodies).
    pub entries: Vec<SedimentEntry>,
}

#[derive(Debug, Clone)]
pub struct SedimentEntry {
    pub slug: String, // `<slug>-full`
    pub title: String,
    pub line: usize,
}
