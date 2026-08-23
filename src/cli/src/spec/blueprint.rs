//! 蓝图定义查看命令（list / show）。

use clap::{Args, Subcommand};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::CliError;
use crate::util::collect_defined_names;

const BLUEPRINT_EXTS: [&str; 4] = [".yaml", ".yml", ".cue", ".json"];

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

/// 蓝图查看命令入口（list / show），返回 `Result<(), CliError>`。
pub fn run(args: &BlueprintArgs) -> Result<(), CliError> {
    let dir = crate::util::blueprint_dir();

    match &args.action {
        BlueprintAction::List => cmd_list(&dir),
        BlueprintAction::Show { name } => cmd_show(&dir, name),
    }
}

fn cmd_list(dir: &str) -> Result<(), CliError> {
    let dir_path = Path::new(dir);
    if dir_path.is_dir() {
        let names = definition_names(dir_path);
        println!("可用的 Blueprint:");
        for name in names {
            println!("  - {name}");
        }
        return Ok(());
    }

    let output = cue_export(&["export", "--out", "json", dir]).map_err(|_| {
        CliError::new("需要 cue CLI。安装: https://cuelang.org/docs/install/".to_string())
    })?;
    if !output.status.success() {
        return Err(CliError::new(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| CliError::new(format!("cue 输出不是合法 JSON: {e}")))?;
    let names = collect_defined_names(&value);
    println!("可用的 Blueprint:");
    for name in names {
        println!("  - {name}");
    }
    Ok(())
}

fn cmd_show(dir: &str, name: &str) -> Result<(), CliError> {
    let dir_path = Path::new(dir);
    if let Some(path) = find_definition(dir_path, name) {
        let content = std::fs::read_to_string(&path)
            .map_err(|err| CliError::new(format!("读取 Blueprint 失败: {err}")))?;
        println!("{content}");
        return Ok(());
    }

    let key = crate::util::to_camel(name);
    let output = cue_export(&["export", "--out", "json", "--expression", &key, dir])
        .map_err(|_| CliError::new(format!("找不到 Blueprint: {name}")))?;
    if !output.status.success() {
        return Err(CliError::new(format!("找不到 Blueprint: {name}")));
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

fn definition_names(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if !entry.path().is_file() {
                continue;
            }
            let Some(file_name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            for ext in BLUEPRINT_EXTS {
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

fn find_definition(dir: &Path, name: &str) -> Option<PathBuf> {
    for ext in BLUEPRINT_EXTS {
        let candidate = dir.join(format!("{name}{ext}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn cue_export(args: &[&str]) -> std::io::Result<std::process::Output> {
    #[cfg(windows)]
    {
        let mut command = Command::new("cmd");
        command.arg("/C").arg("cue").args(args).output()
    }
    #[cfg(not(windows))]
    {
        Command::new("cue").args(args).output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::test_support::{temp_dir, write_script};

    /// 安装 fake cue 到临时 PATH 并返回旧 PATH 守卫。
    fn fake_cue_env() -> (crate::test_support::TempDir, Option<std::ffi::OsString>) {
        let root = temp_dir("qtcloud-blueprint-fake-cue");
        let bin = root.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let cue_bin = if cfg!(windows) { "cue.cmd" } else { "cue" };
        let cue_script = if cfg!(windows) {
            "@echo off\r\nif \"%4\"==\"--expression\" goto show\r\necho {\"alpha\":{\"name\":\"alpha\"},\"beta\":{\"name\":\"beta\"}}\r\nexit /b 0\r\n:show\r\necho {\"name\":\"demo\",\"desc\":\"x\"}\r\nexit /b 0\r\n"
        } else {
            "#!/bin/sh\ncase \"$*\" in\n  *--expression*) echo '{\"name\": \"demo\", \"desc\": \"x\"}' ;;\n  *) echo '{\"alpha\": {\"name\": \"alpha\"}, \"beta\": {\"name\": \"beta\"}}' ;;\nesac\n"
        };
        write_script(&bin.join(cue_bin), cue_script);
        let old_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", &bin);
        }
        (root, old_path)
    }

    fn restore_path(old_path: Option<std::ffi::OsString>) {
        unsafe {
            match old_path {
                Some(p) => std::env::set_var("PATH", p),
                None => std::env::remove_var("PATH"),
            }
        }
    }

    #[test]
    fn cmd_list_parses_cue_json_names() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (root, old_path) = fake_cue_env();

        let result = cmd_list(root.to_str().unwrap());
        restore_path(old_path);

        assert!(result.is_ok(), "{result:?}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cmd_show_prints_expression_json() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (root, old_path) = fake_cue_env();

        let result = cmd_show(root.to_str().unwrap(), "demo");
        restore_path(old_path);

        assert!(result.is_ok(), "{result:?}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cmd_list_reads_yaml_files_without_cue() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-blueprint-file-list");
        std::fs::write(root.join("alpha.yaml"), "name: alpha\n").unwrap();
        std::fs::write(root.join("beta.json"), "{\"name\":\"beta\"}\n").unwrap();
        let empty_bin = root.join("empty-bin");
        std::fs::create_dir_all(&empty_bin).unwrap();

        let old_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", &empty_bin);
        }
        let result = cmd_list(root.to_str().unwrap());
        restore_path(old_path);

        assert!(result.is_ok(), "{result:?}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cmd_show_reads_yaml_file_without_cue() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-blueprint-file-show");
        std::fs::write(root.join("demo.yaml"), "name: demo\n").unwrap();
        let empty_bin = root.join("empty-bin");
        std::fs::create_dir_all(&empty_bin).unwrap();

        let old_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", &empty_bin);
        }
        let result = cmd_show(root.to_str().unwrap(), "demo");
        restore_path(old_path);

        assert!(result.is_ok(), "{result:?}");
        std::fs::remove_dir_all(&root).ok();
    }
}
