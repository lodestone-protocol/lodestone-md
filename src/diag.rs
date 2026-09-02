//! 诊断码常量（规范 §11 一一对应）与诊断条目结构（§8.3）。
//!
//! DNA 特有铁律 2/7：警告绝不阻断解析；错误码以常量定义，禁止字符串散落。

use serde::Serialize;

pub const E_MISSING_ID: &str = "E-MISSING-ID";
pub const E_META_SYNTAX: &str = "E-META-SYNTAX";
pub const E_META_FIELD: &str = "E-META-FIELD";
pub const E_DUP_ID: &str = "E-DUP-ID";
pub const E_REF_NOT_FOUND: &str = "E-REF-NOT-FOUND";
pub const E_CYCLE: &str = "E-CYCLE";
pub const W_VERSION_MISMATCH: &str = "W-VERSION-MISMATCH";
pub const W_DOC_META: &str = "W-DOC-META";
pub const W_CYCLE_DECLARED: &str = "W-CYCLE-DECLARED";
pub const W_REDUNDANT_EDGE: &str = "W-REDUNDANT-EDGE";
pub const W_META_PLACEMENT: &str = "W-META-PLACEMENT";
pub const W_REDUNDANT_META: &str = "W-REDUNDANT-META";
pub const W_UPSTREAM_PENDING: &str = "W-UPSTREAM-PENDING";
pub const W_NFC_VIOLATION: &str = "W-NFC-VIOLATION";

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub code: String,
    pub level: String,
    pub node_id: Option<String>,
    pub edge: Option<String>,
    pub message: String,
}

impl Diagnostic {
    pub fn error(
        code: &str,
        node_id: Option<String>,
        edge: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Diagnostic {
            code: code.to_string(),
            level: "error".to_string(),
            node_id,
            edge,
            message: message.into(),
        }
    }

    pub fn warning(
        code: &str,
        node_id: Option<String>,
        edge: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Diagnostic {
            code: code.to_string(),
            level: "warning".to_string(),
            node_id,
            edge,
            message: message.into(),
        }
    }
}
