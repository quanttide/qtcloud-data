use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

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

const CONTRACT_EXTS: [&str; 4] = [".yaml", ".yml", ".cue", ".json"];

fn contract_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("CONTRACT_DIR").unwrap_or_else(|_| ".quanttide/data/contract".to_string()),
    )
}

/// 以文件直读为主路径列出契约（cue 为可选增强，当前不依赖 cue）。
fn contract_names(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let Some(file_name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            for ext in CONTRACT_EXTS {
                if let Some(stem) = file_name.strip_suffix(ext) {
                    names.push(stem.to_string());
                    break;
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn find_contract(dir: &Path, name: &str) -> Option<PathBuf> {
    for ext in CONTRACT_EXTS {
        let candidate = dir.join(format!("{name}{ext}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn run(args: &ContractArgs) {
    let dir = contract_dir();
    match &args.action {
        ContractAction::List => cmd_list(&dir),
        ContractAction::Show { name } => cmd_show(&dir, name),
    }
}

fn cmd_list(dir: &Path) {
    if !dir.is_dir() {
        eprintln!("契约目录不存在: {}", dir.display());
        std::process::exit(1);
    }
    let names = contract_names(dir);
    println!("可用的 Contract:");
    for name in names {
        println!("  - {name}");
    }
}

fn cmd_show(dir: &Path, name: &str) {
    let path = find_contract(dir, name).unwrap_or_else(|| {
        eprintln!("未找到 Contract: {name}");
        std::process::exit(1);
    });
    let content = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!("读取契约失败: {err}");
        std::process::exit(1);
    });
    println!("{content}");
}
