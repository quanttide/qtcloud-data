use clap::{Args, Subcommand};
use serde_json::Value;
use std::process::Command;

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

pub fn run(args: &PipelineArgs) {
    let dir =
        std::env::var("PIPELINE_DIR").unwrap_or_else(|_| ".quanttide/data/pipeline".to_string());

    match &args.action {
        PipelineAction::List => cmd_list(&dir),
        PipelineAction::Show { name } => cmd_show(&dir, name),
    }
}

fn cmd_list(dir: &str) {
    let output = Command::new("cue")
        .args(["export", "--out", "json", dir])
        .output()
        .expect("需要 cue");
    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    let value: Value = serde_json::from_slice(&output.stdout).expect("cue 输出不是合法 JSON");
    let names = collect_defined_names(&value);
    println!("可用的 Pipeline:");
    for name in names {
        println!("  - {name}");
    }
}

fn cmd_show(dir: &str, name: &str) {
    let key = crate::process::to_camel(name);
    let output = Command::new("cue")
        .args(["export", "--out", "json", "--expression", &key, dir])
        .output()
        .expect("需要 cue");
    if !output.status.success() {
        eprintln!("找不到 Pipeline: {name}");
        std::process::exit(1);
    }
    let value: Value = serde_json::from_slice(&output.stdout).expect("cue 输出不是合法 JSON");
    println!(
        "{}",
        serde_json::to_string_pretty(&value).expect("序列化失败")
    );
}
