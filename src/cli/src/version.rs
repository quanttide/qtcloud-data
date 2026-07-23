use clap::{Args, Subcommand};
use std::process::Command;

use crate::blueprint_core;

#[derive(Args)]
pub struct VersionArgs {
    #[command(subcommand)]
    pub action: VersionAction,
}

#[derive(Subcommand)]
pub enum VersionAction {
    /// 列出版本历史
    List {
        /// blueprint 名称
        name: String,
    },
    /// 查看指定版本详情
    Show {
        /// blueprint 名称
        name: String,
        /// 版本号
        version: String,
    },
    /// 比较两个版本的差异
    Diff {
        /// blueprint 名称
        name: String,
        /// 版本1
        v1: String,
        /// 版本2
        v2: String,
    },
}

pub fn run(args: &VersionArgs) {
    let dir = blueprint_core::spec_dir();

    match &args.action {
        VersionAction::List { name } => {
            // Try spec/ first, then old blueprint/
            let output = Command::new("git")
                .args(["log", "--oneline", "--follow", &format!("{name}-blueprint.cue")])
                .current_dir(&dir)
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    println!("{name} 版本历史:\n{}", String::from_utf8_lossy(&o.stdout));
                }
                _ => {
                    // Fallback to old blueprint directory
                    let old_dir = blueprint_core::blueprint_dir();
                    let output2 = Command::new("git")
                        .args(["log", "--oneline", "--follow", &format!("{name}.cue")])
                        .current_dir(&old_dir)
                        .output();
                    match output2 {
                        Ok(o) if o.status.success() => {
                            println!("{name} 版本历史:\n{}", String::from_utf8_lossy(&o.stdout));
                        }
                        _ => println!("{name}: 无版本历史"),
                    }
                }
            }
        }
        VersionAction::Show { name, version } => {
            let output = Command::new("git")
                .args(["show", &format!("{version}:{name}-blueprint.cue")])
                .current_dir(&dir)
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
                .args(["diff", &format!("{v1}:{name}-blueprint.cue"), &format!("{v2}:{name}-blueprint.cue")])
                .current_dir(&dir)
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
