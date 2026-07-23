use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

use crate::blueprint_core;

#[derive(Args)]
pub struct DesignArgs {
    #[command(subcommand)]
    pub action: DesignAction,
}

#[derive(Subcommand)]
pub enum DesignAction {
    /// 从 DRD 生成数据契约（Contract: .cue + .md）
    Contract {
        /// DRD .md 文件路径
        input: String,
    },
    /// 从 DRD 生成处理蓝图（Blueprint: .cue + .md + .html）
    Blueprint {
        /// DRD .md 文件路径
        input: String,
    },
    /// 将 Markdown 形式化为 CUE 结构化定义
    Formalize {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 从 CUE 生成可视化 HTML 页面
    Preview {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}

pub fn run(args: &DesignArgs) {
    match &args.action {
        DesignAction::Contract { input } => cmd_contract(input),
        DesignAction::Blueprint { input } => cmd_blueprint(input),
        DesignAction::Formalize { input, output } => cmd_formalize(input, output),
        DesignAction::Preview { input, output } => cmd_preview(input, output),
    }
}

// ── Contract: LLM outputs Markdown table, code generates CUE ──

fn cmd_contract(input: &str) {
    let drd = read_drd(input);
    let prompt = blueprint_core::design_contract_prompt(&drd);
    let llm = quanttide_agent::LLM::default();
    let messages = vec![quanttide_agent::Message::new("user", &prompt)];

    let stem = Path::new(input).file_stem().unwrap_or_default().to_string_lossy();
    println!("正在从 DRD 生成 Contract: {stem} ...");

    match llm.complete(&messages, quanttide_agent::llm::CompleteOptions::default()) {
        Ok(resp) => {
            let (cue_content, md_content) = blueprint_core::contract_tables_to_cue(&resp.content);
            write_spec_files(&stem, "contract", &cue_content, &md_content);
        }
        Err(e) => {
            eprintln!("LLM 调用失败: {e}");
            std::process::exit(1);
        }
    }
}

// ── Blueprint: LLM outputs Markdown table, code generates CUE ──

fn cmd_blueprint(input: &str) {
    let drd = read_drd(input);
    let prompt = blueprint_core::design_blueprint_prompt(&drd);
    let llm = quanttide_agent::LLM::default();
    let messages = vec![quanttide_agent::Message::new("user", &prompt)];

    let stem = Path::new(input).file_stem().unwrap_or_default().to_string_lossy();
    println!("正在从 DRD 生成 Blueprint: {stem} ...");

    match llm.complete(&messages, quanttide_agent::llm::CompleteOptions::default()) {
        Ok(resp) => {
            let (cue_content, md_content) = blueprint_core::blueprint_table_to_cue(&resp.content, &stem);
            write_spec_files(&stem, "blueprint", &cue_content, &md_content);

            // Generate HTML preview
            let bp = quanttide_data_core::serde::cue::from_cue::parse_cue_str(&cue_content)
                .unwrap_or_else(|e| {
                    eprintln!("解析 CUE 失败: {e}");
                    std::process::exit(1);
                });
            let step_refs: Vec<(&str, &str, &str, &str)> = bp.pipeline.steps.iter()
                .map(|s| (s.name.as_str(), s.from.as_str(), s.to.as_str(), s.desc.as_str()))
                .collect();
            let html = blueprint_core::render_html(
                &bp.name, bp.description.as_deref(), bp.status.as_str(),
                &bp.created_at, &bp.updated_at,
                &bp.contract.input.schema, &bp.contract.output.schema, &step_refs,
            );
            let spec_dir = blueprint_core::spec_dir();
            let html_path = Path::new(&spec_dir).join(format!("{stem}-blueprint.html"));
            std::fs::write(&html_path, &html).unwrap_or_else(|e| {
                eprintln!("写入 .html 失败: {e}");
                std::process::exit(1);
            });
            println!("已生成: {}", html_path.display());
        }
        Err(e) => {
            eprintln!("LLM 调用失败: {e}");
            std::process::exit(1);
        }
    }
}

// ── Formalize ──

fn cmd_formalize(input: &str, output: &Option<String>) {
    let md_path = Path::new(input);
    let md_content = std::fs::read_to_string(md_path).unwrap_or_else(|e| {
        eprintln!("无法读取 .md 文件: {e}");
        std::process::exit(1);
    });

    let output_path = match output {
        Some(o) => PathBuf::from(o),
        None => {
            let stem = md_path.file_stem().unwrap_or_default().to_string_lossy();
            Path::new(&blueprint_core::spec_dir()).join(format!("{stem}.cue"))
        }
    };

    let prompt = blueprint_core::design_formalize_prompt(&md_content);
    let llm = quanttide_agent::LLM::default();
    let messages = vec![quanttide_agent::Message::new("user", &prompt)];

    println!("正在形式化 {} ...", md_path.display());
    match llm.complete(&messages, quanttide_agent::llm::CompleteOptions::default()) {
        Ok(resp) => {
            let cue_code = blueprint_core::extract_cue(&resp.content);
            std::fs::write(&output_path, &cue_code).unwrap_or_else(|e| {
                eprintln!("写入 .cue 失败: {e}");
                std::process::exit(1);
            });
            println!("已生成: {}", output_path.display());
        }
        Err(e) => {
            eprintln!("LLM 调用失败: {e}");
            std::process::exit(1);
        }
    }
}

// ── Preview ──

fn cmd_preview(input: &str, output: &Option<String>) {
    let cue_path = Path::new(input);
    let output_path = match output {
        Some(o) => PathBuf::from(o),
        None => {
            let stem = cue_path.file_stem().unwrap_or_default().to_string_lossy();
            PathBuf::from(format!("{stem}.html"))
        }
    };

    let cue_content = std::fs::read_to_string(cue_path).unwrap_or_else(|e| {
        eprintln!("无法读取 .cue: {e}");
        std::process::exit(1);
    });

    let bp = quanttide_data_core::serde::cue::from_cue::parse_cue_str(&cue_content)
        .unwrap_or_else(|e| {
            eprintln!("解析 .cue 失败: {e}");
            std::process::exit(1);
        });

    let step_refs: Vec<(&str, &str, &str, &str)> = bp.pipeline.steps.iter()
        .map(|s| (s.name.as_str(), s.from.as_str(), s.to.as_str(), s.desc.as_str()))
        .collect();
    let html = blueprint_core::render_html(
        &bp.name, bp.description.as_deref(), bp.status.as_str(),
        &bp.created_at, &bp.updated_at,
        &bp.contract.input.schema, &bp.contract.output.schema, &step_refs,
    );
    std::fs::write(&output_path, &html).unwrap_or_else(|e| {
        eprintln!("写入 .html 失败: {e}");
        std::process::exit(1);
    });
    println!("已生成: {}", output_path.display());
}

// ── Helpers ──

fn read_drd(input: &str) -> String {
    std::fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("无法读取 DRD 文件 {input}: {e}");
        std::process::exit(1);
    })
}

fn write_spec_files(stem: &str, kind: &str, cue: &str, md: &str) {
    let spec_dir = blueprint_core::spec_dir();
    std::fs::create_dir_all(&spec_dir).unwrap_or_else(|e| {
        eprintln!("无法创建目录 {spec_dir}: {e}");
        std::process::exit(1);
    });
    let cue_path = Path::new(&spec_dir).join(format!("{stem}-{kind}.cue"));
    let md_path = Path::new(&spec_dir).join(format!("{stem}-{kind}.md"));
    std::fs::write(&cue_path, cue).unwrap_or_else(|e| {
        eprintln!("写入 .cue 失败: {e}");
        std::process::exit(1);
    });
    std::fs::write(&md_path, md).unwrap_or_else(|e| {
        eprintln!("写入 .md 失败: {e}");
        std::process::exit(1);
    });
    println!("已生成: {}", cue_path.display());
    println!("已生成: {}", md_path.display());
}
