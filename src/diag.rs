//! Diagnostics (v2.0-draft §8): inherited codes + new ones. All diagnostics
//! are a pure function of the document bytes — the determinism contract.

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

/// A diagnostic code with a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    pub severity: Severity,
    pub code: &'static str,
    pub line: usize,
    pub message: String,
}

impl Diag {
    pub fn warn(code: &'static str, line: usize, message: impl Into<String>) -> Self {
        Diag { severity: Severity::Warning, code, line, message: message.into() }
    }
    pub fn error(code: &'static str, line: usize, message: impl Into<String>) -> Self {
        Diag { severity: Severity::Error, code, line, message: message.into() }
    }
}

/// The stable diagnostic codes (v2.0-draft §8).
pub const W_STATUS_MISSING: &str = "W-STATUS-MISSING";
pub const W_REF_NOT_FOUND: &str = "W-REF-NOT-FOUND";
pub const W_SELF_REF: &str = "W-SELF-REF";
pub const W_SEDIMENT_REF: &str = "W-SEDIMENT-REF";
pub const E_MISSING_ID: &str = "E-MISSING-ID";
pub const E_DUP_ID: &str = "E-DUP-ID";
pub const E_CYCLE: &str = "E-CYCLE";
pub const E_STATUS_TRANSITION: &str = "E-STATUS-TRANSITION";
pub const E_SEDIMENT_REF: &str = "E-SEDIMENT-REF";
pub const E_ABSORB_ALIGNED: &str = "E-ABSORB-ALIGNED";
