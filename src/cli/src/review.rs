use clap::Args;

use crate::error::CliError;
use crate::spec;
use crate::util;

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

    pub fn run(&self, args: &ReviewArgs) -> Result<(), CliError> {
        let dir = util::spec_dir();
        let cue_path = util::resolve_cue_path(&args.input, &dir)
            .or_else(|| util::resolve_cue_path(&args.input, &util::blueprint_dir()))
            .ok_or_else(|| CliError::new(format!("找不到 Specification: {}", args.input)))?;

        let cue_content = std::fs::read_to_string(&cue_path)
            .map_err(|e| CliError::new(format!("无法读取文件 {}: {e}", cue_path.display())))?;

        let blueprint = spec::load_blueprint_from_yaml(&cue_content)?;

        let validation_issues = match quanttide_data::validate(&blueprint) {
            Ok(()) => String::new(),
            Err(errors) => errors
                .iter()
                .map(|e| format!("  - {e}"))
                .collect::<Vec<_>>()
                .join("\n"),
        };

        let prompt = review_prompt(
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
                Ok(())
            }
            Err(e) => {
                eprintln!("LLM 调用失败: {e}");
                if !validation_issues.is_empty() {
                    println!("\n结构校验问题:\n{validation_issues}");
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_prompt_contains_key_info() {
        let prompt = review_prompt("test-bp", "draft", 5, "input-schema", "output-schema", "");
        assert!(prompt.contains("test-bp"));
        assert!(prompt.contains("draft"));
        assert!(prompt.contains("管道步骤数: 5"));
        assert!(prompt.contains("input-schema"));
        assert!(prompt.contains("output-schema"));
        assert!(prompt.contains("【严重】"));
        assert!(prompt.contains("【警告】"));
        assert!(prompt.contains("【建议】"));
    }

    #[test]
    fn test_review_prompt_with_issues() {
        let prompt = review_prompt("bp", "submitted", 0, "in", "out", "step1: missing desc");
        assert!(prompt.contains("step1: missing desc"));
    }

    #[test]
    fn test_review_prompt_empty_issues_shows_none() {
        let prompt = review_prompt("bp", "draft", 0, "in", "out", "");
        assert!(prompt.contains("无"));
    }

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
        handler
            .run(&ReviewArgs {
                input: spec_path.to_string_lossy().to_string(),
            })
            .unwrap();
        unsafe {
            std::env::remove_var("SPEC_DIR");
        }

        std::fs::remove_dir_all(&root).ok();
    }
}

// ── 自 blueprint_core 回迁 ──

/// Build the review prompt for LLM.
pub fn review_prompt(
    name: &str,
    status: &str,
    step_count: usize,
    input_schema: &str,
    output_schema: &str,
    issues: &str,
) -> String {
    format!(
        r#"你是数据工程 Blueprint 审计专家。请审查以下 Blueprint 并输出结构化问题清单。

Blueprint:
- 名称: {name}
- 状态: {status}
- 管道步骤数: {step_count}
- 输入 schema: {input_schema}
- 输出 schema: {output_schema}

结构校验问题:
{issues_section}

请按以下格式输出问题清单：
1. 【严重】阻断性问题（缺失必填字段、契约不完整）
2. 【警告】可能导致交付偏差的问题（口径不明确、步骤描述过于简略）
3. 【建议】可以优化的地方（命名规范、文档完整性）

每个问题标注：严重程度、位置（字段/步骤名）、具体问题、建议修复。"#,
        name = name,
        status = status,
        step_count = step_count,
        input_schema = input_schema,
        output_schema = output_schema,
        issues_section = if issues.is_empty() { "无" } else { issues },
    )
}
