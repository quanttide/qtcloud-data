use clap::Args;
use std::path::{Path, PathBuf};

use crate::error::CliError;
use crate::spec;

#[derive(Args)]
pub struct ImplementArgs {
    /// Blueprint YAML 文件路径
    pub input: String,

    /// 目标语言（默认 python）
    #[arg(short, long, default_value = "python")]
    pub lang: String,

    /// 输出文件路径（可选）
    #[arg(short, long)]
    pub output: Option<String>,
}

pub struct ImplementHandler {
    llm: quanttide_agent::LLM,
}

impl ImplementHandler {
    pub fn new(llm: quanttide_agent::LLM) -> Self {
        Self { llm }
    }

    pub fn run(&self, args: &ImplementArgs) -> Result<(), CliError> {
        match args.lang.as_str() {
            "python" => self.cmd_implement_python(&args.input, &args.output),
            other => Err(CliError::new(format!(
                "不支持的语言: {other}（目前只支持 python）"
            ))),
        }
    }

    fn cmd_implement_python(&self, input: &str, output: &Option<String>) -> Result<(), CliError> {
        let yaml_path = Path::new(input);
        let yaml_content = std::fs::read_to_string(yaml_path)
            .map_err(|e| CliError::new(format!("无法读取 YAML: {e}")))?;

        let bp = spec::load_blueprint_from_yaml(&yaml_content)?;

        let output_path = match output {
            Some(o) => PathBuf::from(o),
            None => {
                let stem = yaml_path.file_stem().unwrap_or_default();
                PathBuf::from(stem).with_extension("py")
            }
        };

        let mut generated_functions = String::new();
        let mut prev_signatures = String::new();

        println!(
            "正在生成 {} 的 Python 实现 ({} 个步骤)...",
            bp.name,
            bp.pipeline.steps.len()
        );

        for (i, step) in bp.pipeline.steps.iter().enumerate() {
            let prompt = implement_step_prompt(
                &step.name,
                &step.from,
                &step.to,
                &step.desc,
                &prev_signatures,
            );

            println!(
                "  [{}/{}] 正在生成: {} ...",
                i + 1,
                bp.pipeline.steps.len(),
                step.name
            );

            let messages = vec![quanttide_agent::Message::new("user", &prompt)];
            match self
                .llm
                .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
            {
                Ok(resp) => {
                    let code = extract_python_fn(&resp.content);
                    generated_functions.push_str(&code);
                    generated_functions.push('\n');
                    // Extract function signature for context
                    let sig = extract_signature(&code, &step.name);
                    prev_signatures.push_str(&format!("{}\n", sig));
                    println!("    已生成: {}", sig.trim());
                }
                Err(e) => {
                    return Err(CliError::new(format!("LLM 调用失败 [{}]: {e}", step.name)));
                }
            }
        }

        // Assemble final script
        let assemble_prompt = implement_assemble_prompt(
            &bp.name,
            &generated_functions,
            &format!("{} 个步骤的数据处理管道", bp.pipeline.steps.len()),
        );

        println!("  正在组装完整脚本...");
        let messages = vec![quanttide_agent::Message::new("user", &assemble_prompt)];
        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                let script = extract_python_fn(&resp.content);
                std::fs::write(&output_path, &script)
                    .map_err(|e| CliError::new(format!("写入脚本失败: {e}")))?;
                println!("已生成: {}", output_path.display());
                Ok(())
            }
            Err(e) => {
                // Fallback: write raw functions
                eprintln!("组装失败 ({e})，写入原始函数...");
                std::fs::write(&output_path, &generated_functions)
                    .map_err(|err| CliError::new(format!("写入脚本失败: {err}")))?;
                println!("已生成（未组装）: {}", output_path.display());
                Ok(())
            }
        }
    }
}

fn extract_python_fn(response: &str) -> String {
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

fn extract_signature(code: &str, step_name: &str) -> String {
    let snake = to_snake(step_name);
    // Find "def func_name" line
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("def ") {
            return trimmed.strip_suffix(':').unwrap_or(trimmed).to_string();
        }
    }
    format!("def {snake}(data):  # {step_name}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fake_llm;
    use crate::test_support::temp_dir;

    const BLUEPRINT_YAML: &str = "name: demo\nstatus: draft\ndescription: 示例\ncreated_at: \"2026-01-01\"\nupdated_at: \"2026-01-01\"\ncontract:\n  input:\n    schema: a\n    format: CSV\n  output:\n    schema: b\n    format: CSV\npipeline:\n  name: demo-pipeline\n  steps:\n    - name: step1\n      from: \"[]\"\n      to: \"[]\"\n      desc: 第一步\n";

    #[test]
    fn implement_python_generates_script_from_blueprint() {
        let root = temp_dir("qtcloud-implement-python");
        let yaml_in = root.join("bp.yaml");
        std::fs::write(&yaml_in, BLUEPRINT_YAML).unwrap();
        let output = root.join("bp.py");

        let handler = ImplementHandler::new(fake_llm(
            "```python\ndef step1(data):\n    return data\n```\n",
        ));
        handler
            .run(&ImplementArgs {
                input: yaml_in.to_string_lossy().to_string(),
                lang: "python".to_string(),
                output: Some(output.to_string_lossy().to_string()),
            })
            .unwrap();

        let script = std::fs::read_to_string(&output).unwrap();
        assert!(script.contains("def "), "{script}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn extract_python_fn_strips_markdown_code_blocks() {
        assert_eq!(
            extract_python_fn("prefix\n```python\ndef f():\n    pass\n```\nsuffix"),
            "def f():\n    pass"
        );
        assert_eq!(extract_python_fn("```\nraw code\n```"), "raw code");
        // 无代码块时原样返回
        assert_eq!(extract_python_fn("def g(): pass"), "def g(): pass");
    }

    #[test]
    fn extract_signature_finds_first_def_line() {
        assert_eq!(
            extract_signature("def step1(data):\n    return data", "step1"),
            "def step1(data)"
        );
        // 无 def 时回退到 snake_case 签名
        assert_eq!(
            extract_signature("x = 1", "Normalize Data"),
            "def normalize_data(data):  # Normalize Data"
        );
    }
}

/// Build the implement prompt for a single pipeline step.
/// Generates a Python function for that step.
// ── prompt 与命名工具 ──
pub fn implement_step_prompt(
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
        func_name = to_snake(step_name),
    )
}

/// Build the assemble prompt: combine all step functions into a complete script.
pub fn implement_assemble_prompt(
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

/// Convert a step name to snake_case function name.
pub fn to_snake(s: &str) -> String {
    s.to_lowercase()
        .replace([' ', '-', '.'], "_")
        .replace("__", "_")
}
