//! Matlab 运行时：函数 codegen prompt 与脚本执行。

use std::path::Path;

use super::Runtime;

pub struct MatlabRuntime;

impl Runtime for MatlabRuntime {
    fn name(&self) -> &'static str {
        "matlab"
    }

    fn extension(&self) -> &'static str {
        "m"
    }

    fn command(&self) -> &'static str {
        "matlab"
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
            r#"你是一个 Matlab 数据处理工程师。请根据以下步骤生成一个 Matlab 函数。

函数名: {step_name}
输入: {from_desc}
输出: {to_desc}
处理逻辑: {step_desc}

前置函数:
{prev}

要求:
1. 函数名使用 snake_case: {func_name}
2. 函数签名应接收输入数据并返回输出数据
3. 代码可以由 Matlab 执行
4. 只输出 Matlab 代码，不要解释

代码:
"#,
            prev = if prev_functions.is_empty() {
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
            r#"你是一个 Matlab 数据处理工程师。请将以下函数组装成完整可执行的 Matlab 脚本。

项目: {project_name}
管道: {pipeline_desc}

函数:
{all_functions}

要求:
1. 按管道顺序调用函数
2. 通过命令行参数接收输入和输出路径
3. 保存最终输出
4. 只输出 Matlab 代码，不要解释

完整脚本:
"#,
        )
    }

    fn extract(&self, response: &str) -> String {
        extract_code(response)
    }

    fn extract_signature(&self, code: &str, step_name: &str) -> String {
        let snake = self.to_snake(step_name);
        code.lines()
            .map(str::trim)
            .find(|line| line.starts_with("function"))
            .map(str::to_string)
            .unwrap_or_else(|| format!("function data = {snake}(data)"))
    }

    fn to_snake(&self, value: &str) -> String {
        value
            .to_lowercase()
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
            .args(["-batch"])
            .arg(format!("run('{}','{}')", input, output))
            .arg(script)
            .current_dir(work_dir)
            .status()
            .map_err(|err| format!("执行 Matlab 脚本失败: {err}"))?;
        if !status.success() {
            return Err("Matlab 脚本执行失败".to_string());
        }
        Ok(String::new())
    }
}

fn extract_code(response: &str) -> String {
    for marker in ["```matlab", "```m", "```"] {
        if let Some(start) = response.find(marker) {
            let code_start = start + marker.len();
            let code_end = response[code_start..]
                .find("```")
                .map(|offset| code_start + offset)
                .unwrap_or(response.len());
            return response[code_start..code_end].trim().to_string();
        }
    }
    response.to_string()
}
