use clap::{Args, Subcommand};
use serde_json::Value;
use std::process::Command;

use crate::error::CliError;
use crate::process::collect_defined_names;

#[derive(Args)]
pub struct PipelineArgs {
    #[command(subcommand)]
    pub action: PipelineAction,
}

#[derive(Subcommand)]
pub enum PipelineAction {
    /// 列出所有可用 pipeline
    List,
    /// 查看 pipeline 定义详情
    Show {
        /// pipeline 名称
        name: String,
    },
}

pub fn run(args: &PipelineArgs) -> Result<(), CliError> {
    let dir =
        std::env::var("PIPELINE_DIR").unwrap_or_else(|_| ".quanttide/data/pipeline".to_string());

    match &args.action {
        PipelineAction::List => cmd_list(&dir),
        PipelineAction::Show { name } => cmd_show(&dir, name),
    }
}

fn cmd_list(dir: &str) -> Result<(), CliError> {
    let output = Command::new("cue")
        .args(["export", "--out", "json", dir])
        .output()
        .map_err(|_| CliError::new("需要 cue CLI".to_string()))?;
    if !output.status.success() {
        return Err(CliError::new(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| CliError::new(format!("cue 输出不是合法 JSON: {e}")))?;
    let names = collect_defined_names(&value);
    println!("可用的 Pipeline:");
    for name in names {
        println!("  - {name}");
    }
    Ok(())
}

fn cmd_show(dir: &str, name: &str) -> Result<(), CliError> {
    let key = crate::process::to_camel(name);
    let output = Command::new("cue")
        .args(["export", "--out", "json", "--expression", &key, dir])
        .output()
        .map_err(|_| CliError::new("需要 cue CLI".to_string()))?;
    if !output.status.success() {
        return Err(CliError::new(format!("找不到 Pipeline: {name}")));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| CliError::new(format!("cue 输出不是合法 JSON: {e}")))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&value)
            .map_err(|e| CliError::new(format!("序列化失败: {e}")))?
    );
    Ok(())
}
