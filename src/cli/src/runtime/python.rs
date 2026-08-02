//! Python 运行时：codegen prompt / 代码提取 / 执行。
//!
//! 从原 `stage/implement.rs` 分离（命令骨架保留在 implement，语言逻辑在此）。

use std::path::Path;

use super::Runtime;

/// Python 运行时（codegen + execute）
pub struct PythonRuntime;

impl Runtime for PythonRuntime {
    fn name(&self) -> &'static str {
        "python"
    }

    fn extension(&self) -> &'static str {
        "py"
    }

    fn command(&self) -> &'static str {
        "python3"
    }

    fn step_prompt(
        &self,
        step_name: &str,
        from_desc: &str,
        to_desc: &str,
        step_desc: &str,
        prev_functions: &str,
    ) -> String {
        format!(
            r#"你是一个 Python 数据处理工程师。请根据以下步骤描述，生成一个 Python 函数。

函数名: {step_name}
输入: {from_desc}
输出: {to_desc}
处理逻辑: {step_desc}

已生成的前置函数:
{prev_section}

要求:
1. 函数名使用 snake_case: {func_name}
2. 函数接收上一步的输出作为输入参数
3. 函数返回处理后的数据
4. 添加类型注解 (from typing import ...)
5. 添加 docstring 描述输入输出
6. 只输出 Python 代码，不要解释

生成的函数:
"#,
            step_name = step_name,
            from_desc = from_desc,
            to_desc = to_desc,
            step_desc = step_desc,
            prev_section = if prev_functions.is_empty() {
                "无（这是第一步）"
            } else {
                prev_functions
            },
            func_name = self.to_snake(step_name),
        )
    }

    fn assemble_prompt(
        &self,
        project_name: &str,
        all_functions: &str,
        pipeline_desc: &str,
    ) -> String {
        format!(
            r#"你是一个 Python 数据处理工程师。请将以下函数组装成一个完整的可执行 Python 脚本。

项目: {project_name}
管道: {pipeline_desc}

函数列表:
{all_functions}

要求:
1. 添加 import 语句（放在文件开头）
2. 添加 if __name__ == "__main__" 入口
3. 按管道顺序调用函数
4. 每个函数的输出传递给下一个函数
5. 添加 argparse 支持输入文件路径参数
6. 只输出 Python 代码，不要解释

完整脚本:
"#
        )
    }

    fn extract(&self, response: &str) -> String {
        // Strip markdown code blocks
        for marker in &["```python", "```py", "```"] {
            if let Some(start) = response.find(marker) {
                let s = start + marker.len();
                let e = response[s..]
                    .find("```")
                    .map(|i| s + i)
                    .unwrap_or(response.len());
                return response[s..e].trim().to_string();
            }
        }
        response.to_string()
    }

    fn extract_signature(&self, code: &str, step_name: &str) -> String {
        let snake = self.to_snake(step_name);
        // Find "def func_name" line
        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("def ") {
                return trimmed.strip_suffix(':').unwrap_or(trimmed).to_string();
            }
        }
        format!("def {snake}(data):  # {step_name}")
    }

    fn to_snake(&self, s: &str) -> String {
        s.to_lowercase()
            .replace([' ', '-', '.'], "_")
            .replace("__", "_")
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
            .map_err(|err| format!("执行 Python 脚本失败: {err}"))?;
        if !status.success() {
            return Err("Python 脚本执行失败".to_string());
        }
        Ok(String::new())
    }
}

/// 名称转 snake_case 函数名（兼容旧路径 `stage::implement::to_snake`）。
///
/// # 示例
///
/// ```
/// assert_eq!(qtcloud_data_cli::runtime::python::to_snake("Normalize Data"), "normalize_data");
/// assert_eq!(qtcloud_data_cli::runtime::python::to_snake("load-csv"), "load_csv");
/// ```
pub fn to_snake(s: &str) -> String {
    PythonRuntime.to_snake(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_strips_markdown_code_blocks() {
        let rt = PythonRuntime;
        assert_eq!(
            rt.extract("prefix\n```python\ndef f():\n    pass\n```\nsuffix"),
            "def f():\n    pass"
        );
        assert_eq!(rt.extract("```\nraw code\n```"), "raw code");
        assert_eq!(rt.extract("def g(): pass"), "def g(): pass");
    }

    #[test]
    fn extract_signature_finds_def_line() {
        let rt = PythonRuntime;
        assert_eq!(
            rt.extract_signature("def step1(data):\n    return data", "step1"),
            "def step1(data)"
        );
        // 无 def 时回退到 snake_case 签名
        assert_eq!(
            rt.extract_signature("x = 1", "Normalize Data"),
            "def normalize_data(data):  # Normalize Data"
        );
    }

    #[test]
    fn step_prompt_mentions_snake_case_name() {
        let rt = PythonRuntime;
        let prompt = rt.step_prompt("Normalize Data", "csv", "out", "清理", "");
        assert!(prompt.contains("normalize_data"));
        assert!(prompt.contains("无（这是第一步）"));
    }
}
