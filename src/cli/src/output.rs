//! 结构化命令输出模型。

use serde::Serialize;

use crate::error::CliError;

/// 所有已迁移命令的 JSON 成功结果外层结构。
#[derive(Debug, Serialize)]
pub struct SuccessEnvelope<T> {
    pub ok: bool,
    pub command: String,
    pub data: T,
}

impl<T> SuccessEnvelope<T> {
    pub fn new(command: impl Into<String>, data: T) -> Self {
        Self {
            ok: true,
            command: command.into(),
            data,
        }
    }
}

/// 打印统一 JSON 成功结果。
pub fn print_success<T: Serialize>(command: &str, data: T) -> Result<(), CliError> {
    let envelope = SuccessEnvelope::new(command, data);
    let output = serde_json::to_string(&envelope)
        .map_err(|err| CliError::new(format!("序列化 JSON 成功结果失败: {err}")))?;
    println!("{output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_envelope_serializes_with_stable_top_level_fields() {
        let envelope = SuccessEnvelope::new(
            "pipeline list",
            serde_json::json!({
                "items": ["normalize"],
            }),
        );

        let value = serde_json::to_value(envelope).unwrap();

        assert_eq!(value["ok"], true);
        assert_eq!(value["command"], "pipeline list");
        assert_eq!(value["data"]["items"][0], "normalize");
    }
}
