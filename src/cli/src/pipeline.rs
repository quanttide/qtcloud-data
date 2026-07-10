use clap::{Args, Subcommand};
use std::process::Command;

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
    let dir = std::env::var("PIPELINES_DIR").unwrap_or_else(|_| ".quanttide/data/pipelines".to_string());

    match &args.action {
        PipelineAction::List => {
            let output = Command::new("cue")
                .args(["export", "--out", "yaml", &dir])
                .output()
                .expect("需要 cue");
            if !output.status.success() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }
            let yaml = String::from_utf8_lossy(&output.stdout);
            println!("可用的 Pipeline:");
            for line in yaml.lines() {
                if line.starts_with("  name: ") {
                    let name = line.trim_start_matches("  name: ").trim_matches('"');
                    println!("  - {name}");
                }
            }
        }
        PipelineAction::Show { name } => {
            let key = super::process::to_camel(name);
            let output = Command::new("cue")
                .args(["export", "--out", "yaml", "--expression", &key, &dir])
                .output()
                .expect("需要 cue");
            if !output.status.success() {
                eprintln!("找不到 Pipeline: {name}");
                std::process::exit(1);
            }
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
}
