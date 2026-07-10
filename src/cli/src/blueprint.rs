use clap::{Args, Subcommand};
use std::process::Command;

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
    let dir = std::env::var("BLUEPRINTS_DIR").unwrap_or_else(|_| ".quanttide/data/blueprints".to_string());

    match &args.action {
        BlueprintAction::List => {
            let output = Command::new("cue")
                .args(["export", "--out", "yaml", &dir])
                .output()
                .expect("需要 cue");
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
        BlueprintAction::Show { name } => {
            let key = super::process::to_camel(name);
            let output = Command::new("cue")
                .args(["export", "--out", "yaml", "--expression", &key, &dir])
                .output()
                .expect("需要 cue");
            if !output.status.success() {
                eprintln!("找不到 Blueprint: {name}");
                std::process::exit(1);
            }
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
}
