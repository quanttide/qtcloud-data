//! Bash 运行时：执行（`.sh` 脚本），不做 codegen。

use std::path::Path;

use super::Runtime;

/// Bash 运行时（execute-only）
pub struct BashRuntime;

impl Runtime for BashRuntime {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn extension(&self) -> &'static str {
        "sh"
    }

    fn command(&self) -> &'static str {
        "bash"
    }

    fn execute_with(
        &self,
        script: &Path,
        input: &str,
        output: &str,
        work_dir: &str,
        cmd: &str,
    ) -> Result<String, String> {
        let status = std::process::Command::new(cmd)
            .arg(script)
            .arg(input)
            .arg(output)
            .current_dir(work_dir)
            .status()
            .map_err(|err| format!("执行 Bash 脚本失败: {err}"))?;
        if !status.success() {
            return Err("Bash 脚本执行失败".to_string());
        }
        Ok(String::new())
    }
}
