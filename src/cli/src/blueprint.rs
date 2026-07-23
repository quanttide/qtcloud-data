use clap::{Args, Subcommand};
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
    Show {
        /// blueprint 名称
        name: String,
    },
}

pub fn run(args: &BlueprintArgs) {
    let dir = blueprint_core::blueprint_dir();

    match &args.action {
        BlueprintAction::List => cmd_list(&dir),
        BlueprintAction::Show { name } => cmd_show(&dir, name),
    }
}

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
