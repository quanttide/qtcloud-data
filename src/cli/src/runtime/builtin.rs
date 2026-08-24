//! 内置运行时：实现 `builtin:copy` 等无需外部解释器的资源。

use std::fs;
use std::path::Path;

use super::Runtime;

pub struct BuiltinRuntime;

impl Runtime for BuiltinRuntime {
    fn name(&self) -> &'static str {
        "builtin"
    }

    fn extension(&self) -> &'static str {
        "builtin"
    }

    fn command(&self) -> &'static str {
        ""
    }

    fn doctor_command(&self) -> Option<&'static str> {
        None
    }

    fn execute_with(
        &self,
        script: &Path,
        input: &str,
        output: &str,
        _work_dir: &str,
        _cmd: &str,
    ) -> Result<String, String> {
        if script.file_name().and_then(|name| name.to_str()) != Some("copy") {
            return Err(format!("不支持的 builtin 资源: {}", script.display()));
        }

        fs::copy(input, output)
            .map(|_| String::new())
            .map_err(|err| format!("builtin:copy 执行失败: {err}"))
    }
}
