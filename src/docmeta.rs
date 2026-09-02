//! 文档级元数据（规范 §3.1）：行为钉死，全部失败路径收敛于"警告 + 继续解析"。

use crate::diag::{self, Diagnostic};
use crate::jsonutil;
use crate::output::DocMetaOut;

/// 顶格、单行、精确前缀（两个空格位置精确匹配）。
pub const PREFIX: &str = "<!-- mddag: ";
pub const SUFFIX: &str = "-->";
pub const SUPPORTED_VERSION: &str = "1.3";

/// 从文件第一个物理行（BOM 已剥离）提取文档级元数据。
///
/// 返回 (Option<DocMetaOut>，是否被采纳)。未采纳时调用方继续尽力解析。
pub fn extract(line1: Option<&str>, diag: &mut Vec<Diagnostic>) -> Option<DocMetaOut> {
    let line = line1?;
    // 提取规则：第一个块为顶格单行且精确匹配前缀时采纳；否则普通 HTML 块，无警告。
    if !line.starts_with(PREFIX) || !line.ends_with(SUFFIX) {
        return None;
    }
    let body = &line[PREFIX.len()..line.len() - SUFFIX.len()];
    // 注释体 MUST NOT 含 "-->"（与节点元数据一致）；违反收敛于 W-DOC-META + 忽略。
    if body.contains(SUFFIX) {
        diag.push(Diagnostic::warning(
            diag::W_DOC_META,
            None,
            None,
            "document-level metadata body contains \"-->\"; ignored",
        ));
        return None;
    }
    // JSON 重复键 → W-DOC-META，忽略之，继续解析。
    if jsonutil::duplicate_keys(body) {
        diag.push(Diagnostic::warning(
            diag::W_DOC_META,
            None,
            None,
            "document-level metadata JSON has duplicate keys; ignored",
        ));
        return None;
    }
    let value: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => {
            diag.push(Diagnostic::warning(
                diag::W_DOC_META,
                None,
                None,
                "document-level metadata JSON parse failed; ignored",
            ));
            return None;
        }
    };
    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            diag.push(Diagnostic::warning(
                diag::W_DOC_META,
                None,
                None,
                "document-level metadata JSON root is not an object; ignored",
            ));
            return None;
        }
    };
    match obj.get("version") {
        // version 未声明：未声明即未承诺，无警告，继续尽力解析。
        None => Some(DocMetaOut { version: None }),
        Some(serde_json::Value::String(s)) => {
            if s != SUPPORTED_VERSION {
                diag.push(Diagnostic::warning(
                    diag::W_VERSION_MISMATCH,
                    None,
                    None,
                    format!(
                        "document version {:?} is not supported ({:?}); parsing best-effort",
                        s, SUPPORTED_VERSION
                    ),
                ));
            }
            Some(DocMetaOut {
                version: Some(s.clone()),
            })
        }
        // version 类型非法 → W-DOC-META，忽略。
        Some(_) => {
            diag.push(Diagnostic::warning(
                diag::W_DOC_META,
                None,
                None,
                "document-level metadata \"version\" is not a string; ignored",
            ));
            None
        }
    }
}
