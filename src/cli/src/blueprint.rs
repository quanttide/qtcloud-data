use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::blueprint_core;

#[derive(Args)]
pub struct BlueprintArgs {
    #[command(subcommand)]
    pub action: BlueprintAction,
}

#[derive(Subcommand)]
pub enum BlueprintAction {
    /// 列出所有可用 blueprint
    List,
    /// 查看 blueprint 定义详情
    Show { name: String },
    /// 审计已有 Blueprint，输出问题清单（老项目入口）
    Review { input: String },
    /// 创建/编辑人类可读的 Blueprint 文档 (.md)
    Design {
        #[command(subcommand)]
        action: DesignAction,
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
    /// 维护版本历史与变更记录
    Version {
        #[command(subcommand)]
        action: VersionAction,
    },
}

#[derive(Subcommand)]
pub enum DesignAction {
    New { name: String },
    Edit { name: String },
}

#[derive(Subcommand)]
pub enum VersionAction {
    List { name: String },
    Show { name: String, version: String },
    Diff { name: String, v1: String, v2: String },
}

pub fn run(args: &BlueprintArgs) {
    let dir = blueprint_core::blueprint_dir();

    match &args.action {
        BlueprintAction::List => cmd_list(&dir),
        BlueprintAction::Show { name } => cmd_show(&dir, name),
        BlueprintAction::Review { input } => cmd_review(input, &dir),
        BlueprintAction::Design { action } => cmd_design(action, &dir),
        BlueprintAction::Formalize { input, output } => cmd_formalize(input, output, &dir),
        BlueprintAction::Preview { input, output } => cmd_preview(input, output),
        BlueprintAction::Version { action } => cmd_version(action, &dir),
    }
}

// ── List / Show ──

fn cmd_list(dir: &str) {
    let output = Command::new("cue")
        .args(["export", "--out", "yaml", dir])
        .output()
        .expect("需要 cue CLI。安装: https://cuelang.org/docs/install/");
    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    let yaml = String::from_utf8_lossy(&output.stdout);
    println!("可用的 Blueprint:");
    for line in yaml.lines() {
        if line.starts_with("  name: ") {
            let name = line.trim_start_matches("  name: ").trim_matches('"');
            println!("  - {name}");
        }
    }
}

fn cmd_show(dir: &str, name: &str) {
    let key = blueprint_core::to_camel(name);
    let output = Command::new("cue")
        .args(["export", "--out", "yaml", "--expression", &key, dir])
        .output()
        .expect("需要 cue CLI");
    if !output.status.success() {
        eprintln!("找不到 Blueprint: {name}");
        std::process::exit(1);
    }
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

// ── Review ──

fn cmd_review(input: &str, dir: &str) {
    let cue_path = blueprint_core::resolve_cue_path(input, dir)
        .unwrap_or_else(|| {
            eprintln!("找不到 Blueprint: {input}");
            std::process::exit(1);
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

    println!("正在审计 Blueprint: {} ...\n", blueprint.name);
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

// ── Design ──

fn cmd_design(action: &DesignAction, dir: &str) {
    match action {
        DesignAction::New { name } => {
            let path = Path::new(dir).join(format!("{name}.md"));
            if path.exists() {
                eprintln!("Blueprint .md 已存在: {}", path.display());
                std::process::exit(1);
            }
            let template = blueprint_core::design_template(name);
            std::fs::write(&path, &template).unwrap_or_else(|e| {
                eprintln!("写入失败: {e}");
                std::process::exit(1);
            });
            println!("已创建: {}", path.display());
        }
        DesignAction::Edit { name } => {
            let path = blueprint_core::resolve_md_path(name, dir);
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let status = Command::new(&editor).arg(&path).status().unwrap_or_else(|e| {
                eprintln!("无法启动编辑器 {editor}: {e}");
                std::process::exit(1);
            });
            if !status.success() {
                std::process::exit(1);
            }
        }
    }
}

// ── Formalize ──

fn cmd_formalize(input: &str, output: &Option<String>, dir: &str) {
    let md_path = Path::new(input);
    let md_content = std::fs::read_to_string(md_path).unwrap_or_else(|e| {
        eprintln!("无法读取 .md 文件: {e}");
        std::process::exit(1);
    });

    let output_path = match output {
        Some(o) => PathBuf::from(o),
        None => {
            let stem = md_path.file_stem().unwrap_or_default().to_string_lossy();
            Path::new(dir).join(format!("{stem}.cue"))
        }
    };

    let llm = quanttide_agent::LLM::default();
    let prompt = blueprint_core::formalize_prompt(&md_content);
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

            let vet = Command::new("cue")
                .args(["vet", output_path.to_str().unwrap()])
                .output();
            match vet {
                Ok(o) if o.status.success() => println!("cue vet 通过"),
                Ok(o) => eprintln!("cue vet 警告:\n{}", String::from_utf8_lossy(&o.stderr)),
                Err(_) => eprintln!("cue CLI 不可用，跳过 vet 验证"),
            }
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

    let blueprint = quanttide_data_core::serde::cue::from_cue::parse_cue_str(&cue_content)
        .unwrap_or_else(|e| {
            eprintln!("解析 .cue 失败: {e}");
            std::process::exit(1);
        });

    let step_refs: Vec<(&str, &str, &str, &str)> = blueprint
        .pipeline
        .steps
        .iter()
        .map(|s| (s.name.as_str(), s.from.as_str(), s.to.as_str(), s.desc.as_str()))
        .collect();

    let html = blueprint_core::render_html(
        &blueprint.name,
        blueprint.description.as_deref(),
        blueprint.status.as_str(),
        &blueprint.created_at,
        &blueprint.updated_at,
        &blueprint.contract.input.schema,
        &blueprint.contract.output.schema,
        &step_refs,
    );

    std::fs::write(&output_path, &html).unwrap_or_else(|e| {
        eprintln!("写入 .html 失败: {e}");
        std::process::exit(1);
    });
    println!("已生成: {}", output_path.display());
}

// ── Version ──

fn cmd_version(action: &VersionAction, dir: &str) {
    match action {
        VersionAction::List { name } => {
            let output = Command::new("git")
                .args(["log", "--oneline", "--follow", &format!("{name}.cue")])
                .current_dir(dir)
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    println!("{} 版本历史:\n{}", name, String::from_utf8_lossy(&o.stdout));
                }
                _ => println!("{name}: 无版本历史"),
            }
        }
        VersionAction::Show { name, version } => {
            let output = Command::new("git")
                .args(["show", &format!("{version}:{name}.cue")])
                .current_dir(dir)
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    println!("{}", String::from_utf8_lossy(&o.stdout));
                }
                _ => {
                    eprintln!("找不到版本 {version} 的 {name}");
                    std::process::exit(1);
                }
            }
        }
        VersionAction::Diff { name, v1, v2 } => {
            let output = Command::new("git")
                .args(["diff", &format!("{v1}:{name}.cue"), &format!("{v2}:{name}.cue")])
                .current_dir(dir)
                .output();
            match output {
                Ok(o) => println!("{}", String::from_utf8_lossy(&o.stdout)),
                Err(e) => {
                    eprintln!("git diff 失败: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
