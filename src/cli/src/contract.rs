use clap::{Args, Subcommand};
use std::process::Command;

#[derive(Args)]
pub struct ContractArgs {
    #[command(subcommand)]
    pub action: ContractAction,
}

#[derive(Subcommand)]
pub enum ContractAction {
    /// 列出所有可用契约
    List,
    /// 查看契约定义详情
    Show {
        /// 契约名称（不含扩展名）
        name: String,
    },
}

pub fn run(args: &ContractArgs) {
    let dir = std::env::var("CONTRACTS_DIR").unwrap_or_else(|_| ".quanttide/data/contracts".to_string());

    match &args.action {
        ContractAction::List => {
            let output = Command::new("cue")
                .args(["export", "--out", "yaml", &dir])
                .output()
                .expect("需要 cue");
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // 如果 cue 失败（可能不是 cue 格式），尝试直接列文件
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    println!("可用的 Contract:");
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".yaml")
                                || name.ends_with(".cue")
                                || name.ends_with(".json")
                            {
                                println!(
                                    "  - {}",
                                    name.trim_end_matches(".yaml")
                                        .trim_end_matches(".cue")
                                        .trim_end_matches(".json")
                                );
                            }
                        }
                    }
                } else {
                    eprintln!("{stderr}");
                    std::process::exit(1);
                }
                return;
            }
            let yaml = String::from_utf8_lossy(&output.stdout);
            println!("可用的 Contract:");
            for line in yaml.lines() {
                if line.starts_with("  name: ") {
                    let name = line.trim_start_matches("  name: ").trim_matches('"');
                    println!("  - {name}");
                }
            }
        }
        ContractAction::Show { name } => {
            // 先尝试 cue 解析
            let key = super::process::to_camel(name);
            let output = Command::new("cue")
                .args(["export", "--out", "yaml", "--expression", &key, &dir])
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    println!("{}", String::from_utf8_lossy(&out.stdout));
                    return;
                }
            }
            // 回退：直接读取文件
            for ext in &["yaml", "yml", "cue", "json"] {
                let path = format!("{dir}/{name}.{ext}");
                if let Ok(content) = std::fs::read_to_string(&path) {
                    println!("{content}");
                    return;
                }
            }
            eprintln!("找不到 Contract: {name}");
            std::process::exit(1);
        }
    }
}
