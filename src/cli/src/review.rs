use clap::Args;

use crate::blueprint_core;

#[derive(Args)]
pub struct ReviewArgs {
    /// blueprint .cue 文件路径或 Blueprint 名称
    pub input: String,
}

pub fn run(args: &ReviewArgs) {
    let dir = blueprint_core::spec_dir();
    let cue_path = blueprint_core::resolve_cue_path(&args.input, &dir)
        .unwrap_or_else(|| {
            // Fallback: try old blueprint_dir
            let old_dir = blueprint_core::blueprint_dir();
            blueprint_core::resolve_cue_path(&args.input, &old_dir)
                .unwrap_or_else(|| {
                    eprintln!("找不到 Specification: {}", args.input);
                    std::process::exit(1);
                })
        });

    let cue_content = std::fs::read_to_string(&cue_path).unwrap_or_else(|e| {
        eprintln!("无法读取文件 {}: {e}", cue_path.display());
        std::process::exit(1);
    });

    let blueprint = quanttide_data_core::serde::cue::from_cue::parse_cue_str(&cue_content)
        .unwrap_or_else(|e| {
            eprintln!("解析 CUE 失败: {e}");
            std::process::exit(1);
        });

    let validation_issues = match quanttide_data_core::validate(&blueprint) {
        Ok(()) => String::new(),
        Err(errors) => errors.iter().map(|e| format!("  - {e}")).collect::<Vec<_>>().join("\n"),
    };

    let llm = quanttide_agent::LLM::default();
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
    match llm.complete(&messages, quanttide_agent::llm::CompleteOptions::default()) {
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
