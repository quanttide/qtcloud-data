//! R 运行时：R codegen prompt 与脚本执行。

use std::path::Path;

use super::Runtime;

pub struct RRuntime;

impl Runtime for RRuntime {
    fn name(&self) -> &'static str {
        "r"
    }

    fn extension(&self) -> &'static str {
        "r"
    }

    fn command(&self) -> &'static str {
        "Rscript"
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
            r#"你是一个 R 数据处理工程师。请根据以下步骤描述，生成一个 R 函数。

函数名: {step_name}
输入: {from_desc}
输出: {to_desc}
处理逻辑: {step_desc}

已生成的前置函数:
{prev}

要求:
1. 函数名使用 snake_case: {func_name}
2. 函数接收上一步的数据并返回处理后的数据
3. 使用清晰的 R 语法，可在 Rscript 中执行
4. 添加注释说明输入输出
5. 只输出 R 代码，不要解释

生成的函数:
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
            r#"你是一个 R 数据处理工程师。请将以下函数组装成一个完整的可执行 R 脚本。

项目: {project_name}
管道: {pipeline_desc}

函数列表:
{all_functions}

要求:
1. 添加必要的 library/import 语句
2. 使用 commandArgs(trailingOnly = TRUE) 接收输入和输出路径
3. 按管道顺序调用函数
4. 只输出 R 代码，不要解释

完整脚本:
"#,
        )
    }

    fn extract(&self, response: &str) -> String {
        extract_code(response, &["```r", "```R", "```"])
    }

    fn extract_signature(&self, code: &str, step_name: &str) -> String {
        let snake = self.to_snake(step_name);
        code.lines()
            .map(str::trim)
            .find(|line| line.starts_with("function ") || line.contains(" <- function"))
            .map(str::to_string)
            .unwrap_or_else(|| format!("{snake} <- function(data) {{  # {step_name}"))
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
        run_script(cmd, script, input, output, work_dir, "R")
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
    language: &str,
) -> Result<String, String> {
    let status = std::process::Command::new(cmd)
        .arg(script)
        .arg(input)
        .arg(output)
        .current_dir(work_dir)
        .status()
        .map_err(|err| format!("执行 {language} 脚本失败: {err}"))?;
    if !status.success() {
        return Err(format!("{language} 脚本执行失败"));
    }
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r_prompt_and_extract_are_language_specific() {
        let runtime = RRuntime;
        assert!(
            runtime
                .step_prompt("清洗 数据", "raw", "clean", "去空格", "")
                .contains("R")
        );
        assert!(
            runtime
                .step_prompt("清洗 数据", "raw", "clean", "去空格", "")
                .contains("qing_")
                || runtime
                    .step_prompt("clean data", "raw", "clean", "trim", "")
                    .contains("clean_data")
        );
        assert_eq!(
            runtime.extract("```r\nclean <- function(x) x\n```").trim(),
            "clean <- function(x) x"
        );
        assert!(
            runtime
                .extract_signature("x <- 1", "Clean Data")
                .contains("clean_data")
        );
    }
}
