use clap::Args;
use std::path::{Path, PathBuf};

use crate::{blueprint_core, spec};

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

    pub fn run(&self, args: &ImplementArgs) {
        match args.lang.as_str() {
            "python" => self.cmd_implement_python(&args.input, &args.output),
            other => {
                eprintln!("不支持的语言: {other}（目前只支持 python）");
                std::process::exit(1);
            }
        }
    }

    fn cmd_implement_python(&self, input: &str, output: &Option<String>) {
        let yaml_path = Path::new(input);
        let yaml_content = std::fs::read_to_string(yaml_path).unwrap_or_else(|e| {
            eprintln!("无法读取 YAML: {e}");
            std::process::exit(1);
        });

        let bp = spec::load_blueprint_from_yaml(&yaml_content).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });

        let output_path = match output {
            Some(o) => PathBuf::from(o),
            None => {
                let stem = yaml_path.file_stem().unwrap_or_default().to_string_lossy();
                PathBuf::from(format!("{stem}.py"))
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
            let prompt = blueprint_core::implement_step_prompt(
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
                    eprintln!("  LLM 调用失败 [{}]: {e}", step.name);
                    std::process::exit(1);
                }
            }
        }

        // Assemble final script
        let assemble_prompt = blueprint_core::implement_assemble_prompt(
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
                std::fs::write(&output_path, &script).unwrap_or_else(|e| {
                    eprintln!("写入脚本失败: {e}");
                    std::process::exit(1);
                });
                println!("已生成: {}", output_path.display());
            }
            Err(e) => {
                // Fallback: write raw functions
                eprintln!("组装失败 ({e})，写入原始函数...");
                std::fs::write(&output_path, &generated_functions).unwrap_or_else(|e| {
                    eprintln!("写入脚本失败: {e}");
                    std::process::exit(1);
                });
                println!("已生成（未组装）: {}", output_path.display());
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
    let snake = blueprint_core::to_snake(step_name);
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

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    const BLUEPRINT_YAML: &str = "name: demo\nstatus: draft\ndescription: 示例\ncreated_at: \"2026-01-01\"\nupdated_at: \"2026-01-01\"\ncontract:\n  input:\n    schema: a\n    format: CSV\n  output:\n    schema: b\n    format: CSV\npipeline:\n  name: demo-pipeline\n  steps:\n    - name: step1\n      from: \"[]\"\n      to: \"[]\"\n      desc: 第一步\n";

    #[test]
    fn implement_python_generates_script_from_blueprint() {
        let root = temp_root("qtcloud-implement-python");
        let yaml_in = root.join("bp.yaml");
        std::fs::write(&yaml_in, BLUEPRINT_YAML).unwrap();
        let output = root.join("bp.py");

        let handler = ImplementHandler::new(fake_llm(
            "```python\ndef step1(data):\n    return data\n```\n",
        ));
        handler.run(&ImplementArgs {
            input: yaml_in.to_string_lossy().to_string(),
            lang: "python".to_string(),
            output: Some(output.to_string_lossy().to_string()),
        });

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
