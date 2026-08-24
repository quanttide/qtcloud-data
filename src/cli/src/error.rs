//! CLI 统一错误类型：命令入口返回 `Result<(), CliError>`，`main` 顶层统一格式化。

use std::fmt;
use std::io;

/// 未显式分类时使用的通用错误码。
pub const DEFAULT_ERROR_CODE: &str = "cli_error";

/// `io::Error` 转换使用的错误码。
pub const IO_ERROR_CODE: &str = "io_error";

/// CLI 统一错误，携带稳定错误码和用户可读消息。
#[derive(Debug)]
pub struct CliError {
    code: String,
    message: String,
}

impl CliError {
    /// 创建通用 CLI 错误，保持旧有文本错误行为。
    pub fn new(message: impl Into<String>) -> Self {
        Self::with_code(DEFAULT_ERROR_CODE, message)
    }

    /// 使用调用方指定的稳定错误码创建错误。
    pub fn with_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// 返回机器可读错误码。
    pub fn code(&self) -> &str {
        &self.code
    }

    /// 返回用户可读错误消息。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 返回供 `--json` 输出复用的错误对象。
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::json!({
            "code": self.code,
            "message": self.message,
        })
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        Self::with_code(IO_ERROR_CODE, err.to_string())
    }
}

impl From<String> for CliError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for CliError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_error_roundtrips_through_display() {
        let err = CliError::new("pipeline failed");
        assert_eq!(err.to_string(), "pipeline failed");
    }

    #[test]
    fn cli_error_converts_from_string_and_str() {
        assert_eq!(CliError::from("abc").to_string(), "abc");
        assert_eq!(CliError::from("abc".to_string()).to_string(), "abc");
    }

    #[test]
    fn cli_error_implements_error_trait_and_debug() {
        let err = CliError::new("boom");
        let dyn_err: &dyn std::error::Error = &err;
        assert_eq!(dyn_err.to_string(), "boom");
        assert!(format!("{err:?}").contains("boom"));
    }

    #[test]
    fn cli_error_converts_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "no such file");
        let err: CliError = io_err.into();
        assert_eq!(err.to_string(), "no such file");
    }

    #[test]
    fn cli_error_new_accepts_string_like_values() {
        let a = CliError::new(String::from("owned"));
        let b = CliError::new("borrowed");
        let c = CliError::new(format!("fmt {}", 42));
        assert_eq!(a.to_string(), "owned");
        assert_eq!(b.to_string(), "borrowed");
        assert_eq!(c.to_string(), "fmt 42");
    }

    #[test]
    fn cli_error_exposes_default_code_and_json_value() {
        let err = CliError::new("pipeline failed");

        assert_eq!(err.code(), "cli_error");
        assert_eq!(err.message(), "pipeline failed");
        assert_eq!(
            err.to_json_value(),
            serde_json::json!({
                "code": "cli_error",
                "message": "pipeline failed"
            })
        );
    }

    #[test]
    fn cli_error_with_code_preserves_text_display() {
        let err = CliError::with_code("not_found", "missing blueprint");

        assert_eq!(err.code(), "not_found");
        assert_eq!(err.to_string(), "missing blueprint");
        assert_eq!(err.to_json_value()["code"], "not_found");
    }

    #[test]
    fn io_error_uses_io_code() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "no such file");
        let err: CliError = io_err.into();

        assert_eq!(err.code(), "io_error");
        assert_eq!(err.to_json_value()["message"], "no such file");
    }
}
