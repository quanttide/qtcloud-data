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

pub fn run(args: &ImplementArgs) {
    match args.lang.as_str() {
        "python" => cmd_implement_python(&args.input, &args.output),
        other => {
            eprintln!("不支持的语言: {other}（目前只支持 python）");
            std::process::exit(1);
        }
    }
}

fn cmd_implement_python(input: &str, output: &Option<String>) {
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

    let llm = quanttide_agent::LLM::default();
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
        match llm.complete(&messages, quanttide_agent::llm::CompleteOptions::default()) {
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
    match llm.complete(&messages, quanttide_agent::llm::CompleteOptions::default()) {
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
