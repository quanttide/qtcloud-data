use clap::Args;

use crate::{blueprint_core, spec};

#[derive(Args)]
pub struct ReviewArgs {
    /// blueprint .cue 文件路径或 Blueprint 名称
    pub input: String,
}

pub struct ReviewHandler {
    llm: quanttide_agent::LLM,
}

impl ReviewHandler {
    pub fn new(llm: quanttide_agent::LLM) -> Self {
        Self { llm }
    }

    pub fn run(&self, args: &ReviewArgs) {
        let dir = blueprint_core::spec_dir();
        let cue_path = blueprint_core::resolve_cue_path(&args.input, &dir).unwrap_or_else(|| {
            // Fallback: try old blueprint_dir
            let old_dir = blueprint_core::blueprint_dir();
            blueprint_core::resolve_cue_path(&args.input, &old_dir).unwrap_or_else(|| {
                eprintln!("找不到 Specification: {}", args.input);
                std::process::exit(1);
            })
        });

        let cue_content = std::fs::read_to_string(&cue_path).unwrap_or_else(|e| {
            eprintln!("无法读取文件 {}: {e}", cue_path.display());
            std::process::exit(1);
        });

        let blueprint = spec::load_blueprint_from_yaml(&cue_content).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });

        let validation_issues = match quanttide_data::validate(&blueprint) {
            Ok(()) => String::new(),
            Err(errors) => errors
                .iter()
                .map(|e| format!("  - {e}"))
                .collect::<Vec<_>>()
                .join("\n"),
        };

        let prompt = blueprint_core::review_prompt(
            &blueprint.name,
            blueprint.status.as_str(),
            blueprint.pipeline.steps.len(),
            &blueprint.contract.input.schema,
            &blueprint.contract.output.schema,
            &validation_issues,
        );
        let messages = vec![quanttide_agent::Message::new("user", &prompt)];

        println!("正在审计 Specification: {} ...\n", blueprint.name);
        match self
            .llm
            .complete(&messages, quanttide_agent::llm::CompleteOptions::default())
        {
            Ok(resp) => {
                println!("=== Review Report ===");
                println!("{}", resp.content);
                println!("\n=== 结构校验 ===");
                if validation_issues.is_empty() {
                    println!("  结构校验通过。");
                } else {
                    println!("{}", validation_issues);
                }
            }
            Err(e) => {
                eprintln!("LLM 调用失败: {e}");
                if !validation_issues.is_empty() {
                    println!("\n结构校验问题:\n{validation_issues}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::test_support::fake_llm;

    #[test]
    fn review_audits_blueprint_and_prints_report() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = std::env::temp_dir().join(format!("qtcloud-review-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(&root).unwrap();
        let spec_path = root.join("demo-blueprint.yaml");
        std::fs::write(
            &spec_path,
            "name: demo\nstatus: draft\ndescription: 示例\ncreated_at: \"2026-01-01\"\nupdated_at: \"2026-01-01\"\ncontract:\n  input:\n    schema: a\n    format: CSV\n  output:\n    schema: b\n    format: CSV\npipeline:\n  name: demo-pipeline\n  steps:\n    - name: step1\n      from: \"[]\"\n      to: \"[]\"\n      desc: 第一步\n",
        )
        .unwrap();

        unsafe {
            std::env::set_var("SPEC_DIR", &root);
        }
        let handler = ReviewHandler::new(fake_llm("## 审计结论\n结构合理。"));
        handler.run(&ReviewArgs {
            input: spec_path.to_string_lossy().to_string(),
        });
        unsafe {
            std::env::remove_var("SPEC_DIR");
        }

        std::fs::remove_dir_all(&root).ok();
    }
}
