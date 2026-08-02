//! 代码实现命令：Blueprint → 可执行脚本（LLM codegen）。
//!
//! 语言逻辑（prompt / 提取 / 执行）在 `crate::runtime`（`Runtime` trait + 注册表），
//! 本模块只保留命令骨架：查注册表 → 逐 step 调 LLM → 组装 → 写文件。

use clap::Args;
use std::path::{Path, PathBuf};

use crate::error::CliError;
use crate::runtime::{self, Runtime};
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
        let rt = runtime::from_name(&args.lang).ok_or_else(|| {
            CliError::new(format!("不支持的语言: {}（目前只支持 python）", args.lang))
        })?;
        self.cmd_implement(rt.as_ref(), &args.input, &args.output, &args.lang)
    }

    fn cmd_implement(
        &self,
        rt: &dyn Runtime,
        input: &str,
        output: &Option<String>,
        lang: &str,
    ) -> Result<(), CliError> {
        let yaml_path = Path::new(input);
        let yaml_content = std::fs::read_to_string(yaml_path)
            .map_err(|e| CliError::new(format!("无法读取 YAML: {e}")))?;

        let bp = spec::load_blueprint_from_yaml(&yaml_content)?;

        let output_path = match output {
            Some(o) => PathBuf::from(o),
            None => {
                let stem = yaml_path.file_stem().unwrap_or_default();
                PathBuf::from(stem).with_extension(rt.extension())
            }
        };

        let mut generated_functions = String::new();
        let mut prev_signatures = String::new();

        println!(
            "正在生成 {} 的 {} 实现 ({} 个步骤)...",
            bp.name,
            lang,
            bp.pipeline.steps.len()
        );

        for (i, step) in bp.pipeline.steps.iter().enumerate() {
            let prompt = rt.step_prompt(
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
                    let code = rt.extract(&resp.content);
                    generated_functions.push_str(&code);
                    generated_functions.push('\n');
                    // Extract function signature for context
                    let sig = rt.extract_signature(&code, &step.name);
                    prev_signatures.push_str(&format!("{}\n", sig));
                    println!("    已生成: {}", sig.trim());
                }
                Err(e) => {
                    return Err(CliError::new(format!("LLM 调用失败 [{}]: {e}", step.name)));
                }
            }
        }

        // Assemble final script
        let assemble_prompt = rt.assemble_prompt(
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
                let script = rt.extract(&resp.content);
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

// ── 兼容层（v0.2.x 公开路径，随 v0.3 移除）──
// 语言逻辑已迁移到 `runtime::python::PythonRuntime`，此处保留旧 pub 函数转发，
// 避免破坏依赖 `stage::implement::*` 的外部代码。

#[deprecated(note = "迁移到 runtime::python::PythonRuntime.step_prompt")]
pub fn implement_step_prompt(
    step_name: &str,
    from_desc: &str,
    to_desc: &str,
    step_desc: &str,
    prev_functions: &str,
) -> String {
    runtime::python::PythonRuntime.step_prompt(
        step_name,
        from_desc,
        to_desc,
        step_desc,
        prev_functions,
    )
}

#[deprecated(note = "迁移到 runtime::python::PythonRuntime.assemble_prompt")]
pub fn implement_assemble_prompt(
    project_name: &str,
    all_functions: &str,
    pipeline_desc: &str,
) -> String {
    runtime::python::PythonRuntime.assemble_prompt(project_name, all_functions, pipeline_desc)
}

#[deprecated(note = "迁移到 runtime::python::PythonRuntime.to_snake")]
pub fn to_snake(s: &str) -> String {
    runtime::python::PythonRuntime.to_snake(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::test_support::fake_llm;
    use std::io::Write;

    fn write_blueprint(path: &std::path::Path) {
        let yaml = r#"name: demo
pipeline:
  name: main
  steps:
    - name: normalize
      from: csv
      to: out
      desc: 清理
status: draft
created_at: "2026-08-02T00:00:00Z"
updated_at: "2026-08-02T00:00:00Z"
"#;
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(yaml.as_bytes()).unwrap();
    }

    #[test]
    fn implement_python_generates_script_from_blueprint() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = crate::test_support::temp_dir("qtcloud-implement-python");
        let bp = root.join("demo.yaml");
        write_blueprint(&bp);

        let handler = ImplementHandler::new(fake_llm(
            r#"```python
def step1(data):
    return data
```"#,
        ));
        let args = ImplementArgs {
            input: bp.to_string_lossy().into_owned(),
            lang: "python".to_string(),
            output: Some(root.join("out.py").to_string_lossy().into_owned()),
        };
        handler.run(&args).unwrap();
        let script = std::fs::read_to_string(root.join("out.py")).unwrap();
        assert!(script.contains("def step1(data)"));
    }

    #[test]
    fn implement_rejects_unsupported_lang() {
        let handler = ImplementHandler::new(fake_llm(""));
        let args = ImplementArgs {
            input: "x.yaml".to_string(),
            lang: "r".to_string(),
            output: None,
        };
        let err = handler.run(&args).unwrap_err();
        assert!(err.to_string().contains("不支持的语言"));
    }
}
