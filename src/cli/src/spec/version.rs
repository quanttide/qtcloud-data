//! Specification 版本管理：基于 git 历史（list / show / diff）。
//! 注：顶层 `version` 命令已废弃（v0.3 移除），使用 `spec version`。

use clap::{Args, Subcommand};
use std::process::Command;

use crate::error::CliError;
use crate::util;

#[derive(Args)]
pub struct SpecVersionArgs {
    #[command(subcommand)]
    pub action: SpecVersionAction,
}

#[derive(Subcommand)]
pub enum SpecVersionAction {
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

/// 规格版本管理命令入口（list / show / diff）。
pub fn run(args: &SpecVersionArgs) -> Result<(), CliError> {
    let dir = util::spec_dir();

    match &args.action {
        SpecVersionAction::List { name } => {
            // Try spec/ first, then old blueprint/
            let output = Command::new("git")
                .args([
                    "log",
                    "--oneline",
                    "--follow",
                    &format!("{name}-blueprint.cue"),
                ])
                .current_dir(&dir)
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    println!("{name} 版本历史:\n{}", String::from_utf8_lossy(&o.stdout));
                }
                _ => {
                    // Fallback to old blueprint directory
                    let old_dir = util::blueprint_dir();
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
            Ok(())
        }
        SpecVersionAction::Show { name, version } => {
            let output = Command::new("git")
                .args(["show", &format!("{version}:{name}-blueprint.cue")])
                .current_dir(&dir)
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    println!("{}", String::from_utf8_lossy(&o.stdout));
                    Ok(())
                }
                _ => Err(CliError::new(format!("找不到版本 {version} 的 {name}"))),
            }
        }
        SpecVersionAction::Diff { name, v1, v2 } => {
            let output = Command::new("git")
                .args([
                    "diff",
                    &format!("{v1}:{name}-blueprint.cue"),
                    &format!("{v2}:{name}-blueprint.cue"),
                ])
                .current_dir(&dir)
                .output();
            match output {
                Ok(o) => {
                    println!("{}", String::from_utf8_lossy(&o.stdout));
                    Ok(())
                }
                Err(e) => Err(CliError::new(format!("git diff 失败: {e}"))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;

    /// 在临时目录初始化一个含两个提交的 git 仓库，返回 spec 目录路径。
    fn init_git_repo(name: &str) -> (std::path::PathBuf, String, String) {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();

        let git = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(&dir)
                .status()
                .expect("需要 git");
            assert!(status.success(), "git {:?} 失败", args);
        };

        git(&["init", "-q"]);
        git(&["config", "user.name", "test"]);
        git(&["config", "user.email", "test@example.com"]);

        std::fs::write(dir.join("abc-blueprint.cue"), "v1 content\n").unwrap();
        git(&["add", "abc-blueprint.cue"]);
        git(&["commit", "-q", "-m", "v1"]);
        let v1 = String::from_utf8_lossy(
            &Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&dir)
                .output()
                .unwrap()
                .stdout,
        )
        .trim()
        .to_string();

        std::fs::write(dir.join("abc-blueprint.cue"), "v2 content\n").unwrap();
        git(&["add", "abc-blueprint.cue"]);
        git(&["commit", "-q", "-m", "v2"]);
        let v2 = String::from_utf8_lossy(
            &Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&dir)
                .output()
                .unwrap()
                .stdout,
        )
        .trim()
        .to_string();

        (dir, v1, v2)
    }

    #[test]
    fn version_list_prints_commit_history_from_spec_dir() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (dir, _, _) = init_git_repo("qtcloud-version-list");

        unsafe {
            std::env::set_var("SPEC_DIR", &dir);
        }
        run(&SpecVersionArgs {
            action: SpecVersionAction::List {
                name: "abc".to_string(),
            },
        })
        .unwrap();
        unsafe {
            std::env::remove_var("SPEC_DIR");
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn version_show_prints_blueprint_at_given_version() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (dir, v1, _) = init_git_repo("qtcloud-version-show");

        unsafe {
            std::env::set_var("SPEC_DIR", &dir);
        }
        run(&SpecVersionArgs {
            action: SpecVersionAction::Show {
                name: "abc".to_string(),
                version: v1,
            },
        })
        .unwrap();
        unsafe {
            std::env::remove_var("SPEC_DIR");
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn version_diff_compares_two_versions() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (dir, v1, v2) = init_git_repo("qtcloud-version-diff");

        unsafe {
            std::env::set_var("SPEC_DIR", &dir);
        }
        run(&SpecVersionArgs {
            action: SpecVersionAction::Diff {
                name: "abc".to_string(),
                v1,
                v2,
            },
        })
        .unwrap();
        unsafe {
            std::env::remove_var("SPEC_DIR");
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn version_list_falls_back_to_blueprint_dir_on_missing_history() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir =
            std::env::temp_dir().join(format!("qtcloud-version-fallback-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        // 空目录：spec/ 无版本历史 → 回退 blueprint/ 也无 → 打印“无版本历史”
        std::fs::create_dir_all(&dir).unwrap();

        unsafe {
            std::env::set_var("SPEC_DIR", &dir);
            std::env::set_var("BLUEPRINT_DIR", &dir);
        }
        run(&SpecVersionArgs {
            action: SpecVersionAction::List {
                name: "missing".to_string(),
            },
        })
        .unwrap();
        unsafe {
            std::env::remove_var("SPEC_DIR");
            std::env::remove_var("BLUEPRINT_DIR");
        }

        std::fs::remove_dir_all(&dir).ok();
    }
}
