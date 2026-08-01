use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

use crate::error::CliError;

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

pub fn run(args: &ContractArgs) -> Result<(), CliError> {
    let dir = contract_dir();
    match &args.action {
        ContractAction::List => cmd_list(&dir),
        ContractAction::Show { name } => cmd_show(&dir, name),
    }
}

fn cmd_list(dir: &Path) -> Result<(), CliError> {
    if !dir.is_dir() {
        return Err(CliError::new(format!("契约目录不存在: {}", dir.display())));
    }
    let names = contract_names(dir);
    println!("可用的 Contract:");
    for name in names {
        println!("  - {name}");
    }
    Ok(())
}

fn cmd_show(dir: &Path, name: &str) -> Result<(), CliError> {
    let path = find_contract(dir, name)
        .ok_or_else(|| CliError::new(format!("未找到 Contract: {name}")))?;
    let content = std::fs::read_to_string(&path)
        .map_err(|err| CliError::new(format!("读取契约失败: {err}")))?;
    println!("{content}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::temp_dir;

    #[test]
    fn contract_names_lists_supported_extensions_sorted_and_deduped() {
        let root = temp_dir("qtcloud-contract-names");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("alpha.yaml"), "a").unwrap();
        std::fs::write(root.join("beta.yml"), "b").unwrap();
        std::fs::write(root.join("gamma.cue"), "c").unwrap();
        std::fs::write(root.join("delta.json"), "d").unwrap();
        // 同 stem 不同扩展名 → 去重；不支持/伪装扩展名 → 忽略
        std::fs::write(root.join("alpha.cue"), "a2").unwrap();
        std::fs::write(root.join("alpha.yaml.bak"), "x").unwrap();
        std::fs::write(root.join("README.txt"), "r").unwrap();
        std::fs::write(root.join("notes"), "no ext").unwrap();
        // 子目录不应被计入
        std::fs::create_dir_all(root.join("nested")).unwrap();
        std::fs::write(root.join("nested").join("inner.yaml"), "i").unwrap();

        let names = contract_names(&root);
        assert_eq!(names, vec!["alpha", "beta", "delta", "gamma"]);

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn contract_names_handles_missing_dir_gracefully() {
        let root = temp_dir("qtcloud-contract-missing");
        assert!(contract_names(&root).is_empty());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn find_contract_prefers_yaml_over_other_extensions() {
        let root = temp_dir("qtcloud-contract-find");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("doc.yaml"), "y").unwrap();
        std::fs::write(root.join("doc.cue"), "c").unwrap();
        std::fs::write(root.join("doc.json"), "j").unwrap();

        assert_eq!(find_contract(&root, "doc"), Some(root.join("doc.yaml")));
        assert_eq!(find_contract(&root, "missing"), None);

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn find_contract_requires_exact_stem_match() {
        let root = temp_dir("qtcloud-contract-stem");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("contract-2024.yaml"), "y").unwrap();

        assert_eq!(find_contract(&root, "contract"), None);
        assert_eq!(
            find_contract(&root, "contract-2024"),
            Some(root.join("contract-2024.yaml"))
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cmd_list_reports_missing_dir_without_exiting() {
        let root = temp_dir("qtcloud-contract-list-missing");
        let missing = root.join("nope");
        let err = cmd_list(&missing).unwrap_err();
        assert!(err.to_string().contains("契约目录不存在"), "{}", err);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cmd_show_reports_missing_contract_without_exiting() {
        let root = temp_dir("qtcloud-contract-show-missing");
        let err = cmd_show(&root, "ghost").unwrap_err();
        assert_eq!(err.to_string(), "未找到 Contract: ghost");
        std::fs::remove_dir_all(&root).ok();
    }
}
