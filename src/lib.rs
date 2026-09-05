//! mddag — Lodestone Protocol v2.0-draft (MD-DAG 2) reference implementation.
//!
//! Markdown-native DAG: heading = skeleton, links = magnetic lines, body =
//! iron filings, status = list. Three parts (ADR-0002):
//! - shape (`scan`): parse document bytes into the Doc model;
//! - read (`project`): L0/L1/L2 on-demand projections;
//! - runtime (`ops`): five streaming append operations with audit.
//!
//! Zero dependencies. Deterministic: every public function is a pure
//! function of its input bytes.

pub mod diag;
pub mod doc;
pub mod index;
pub mod library;
pub mod ops;
pub mod project;
pub mod scan;
pub mod slug;

pub use diag::{Diag, Severity};
pub use doc::{Doc, Lodestone, MagneticLine, Sediment, SedimentEntry, Status};
pub use scan::{find_cycle, scan};

/// Version of the implemented protocol line (draft, not frozen).
pub const PROTOCOL_VERSION: &str = "2.0-draft";
