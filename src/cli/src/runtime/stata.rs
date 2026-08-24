//! Stata 运行时：do-file codegen prompt 与脚本执行。

use std::path::Path;

use super::Runtime;

pub struct StataRuntime;

impl Runtime for StataRuntime {
    fn name(&self) -> &'static str {
        "stata"
    }

    fn extension(&self) -> &'static str {
        "do"
    }

    fn command(&self) -> &'static str {
        "stata"
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
            r#"你是一个 Stata 数据处理工程师。请根据以下步骤描述生成一个 Stata do-file 片段。

步骤名: {step_name}
输入: {from_desc}
输出: {to_desc}
处理逻辑: {step_desc}

前置步骤:
{prev}

要求:
1. 使用 Stata 合法命令完成数据处理
2. 使用 `args input output` 接收输入和输出路径
3. 使用清晰的注释说明处理逻辑
4. 只输出 Stata 代码，不要解释

代码:
"#,
            prev = if prev_functions.is_empty() {
                "无（这是第一步）"
            } else {
                prev_functions
            },
        )
    }

    fn assemble_prompt(
        &self,
        project_name: &str,
        all_functions: &str,
        pipeline_desc: &str,
    ) -> String {
        format!(
            r#"你是一个 Stata 数据处理工程师。请将以下 Stata 代码组装成完整可执行的 do-file。

项目: {project_name}
管道: {pipeline_desc}

代码:
{all_functions}

要求:
1. 文件开头使用 `args input output`
2. 按管道顺序执行每个步骤
3. 最后保存输出数据
4. 只输出 Stata 代码，不要解释

完整 do-file:
"#,
        )
    }

    fn extract(&self, response: &str) -> String {
        extract_code(response, &["```stata", "```do", "```"])
    }

    fn extract_signature(&self, code: &str, step_name: &str) -> String {
        code.lines()
            .map(str::trim)
            .find(|line| line.starts_with("program define"))
            .map(str::to_string)
            .unwrap_or_else(|| format!("* {step_name}"))
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
        run_script(cmd, script, input, output, work_dir)
    }
}

fn extract_code(response: &str, markers: &[&str]) -> String {
    for marker in markers {
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

fn run_script(
    cmd: &str,
    script: &Path,
    input: &str,
    output: &str,
    work_dir: &str,
) -> Result<String, String> {
    let status = std::process::Command::new(cmd)
        .args(["-b", "do"])
        .arg(script)
        .arg(input)
        .arg(output)
        .current_dir(work_dir)
        .status()
        .map_err(|err| format!("执行 Stata 脚本失败: {err}"))?;
    if !status.success() {
        return Err("Stata 脚本执行失败".to_string());
    }
    Ok(String::new())
}
