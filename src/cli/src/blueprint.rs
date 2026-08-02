use clap::{Args, Subcommand};
use serde_json::Value;
use std::process::Command;

use crate::error::CliError;
use crate::util::collect_defined_names;

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

pub fn run(args: &BlueprintArgs) -> Result<(), CliError> {
    let dir = crate::util::blueprint_dir();

    match &args.action {
        BlueprintAction::List => cmd_list(&dir),
        BlueprintAction::Show { name } => cmd_show(&dir, name),
    }
}

fn cmd_list(dir: &str) -> Result<(), CliError> {
    let output = Command::new("cue")
        .args(["export", "--out", "json", dir])
        .output()
        .map_err(|_| {
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
    let key = crate::util::to_camel(name);
    let output = Command::new("cue")
        .args(["export", "--out", "json", "--expression", &key, dir])
        .output()
        .map_err(|_| CliError::new("需要 cue CLI".to_string()))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::test_support::{temp_dir, write_script};

    /// 安装 fake cue 到临时 PATH 并返回旧 PATH 守卫。
    fn fake_cue_env() -> (std::path::PathBuf, Option<std::ffi::OsString>) {
        let root = temp_dir("qtcloud-blueprint-fake-cue");
        let bin = root.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        write_script(
            &bin.join("cue"),
            "#!/bin/sh\ncase \"$*\" in\n  *--expression*) echo '{\"name\": \"demo\", \"desc\": \"x\"}' ;;\n  *) echo '{\"alpha\": {\"name\": \"alpha\"}, \"beta\": {\"name\": \"beta\"}}' ;;\nesac\n",
        );
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
    fn cmd_list_reports_cue_missing_as_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-blueprint-no-cue");
        let empty_bin = root.join("empty-bin");
        std::fs::create_dir_all(&empty_bin).unwrap();

        let old_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", &empty_bin);
        }
        let err = cmd_list(root.to_str().unwrap()).unwrap_err();
        restore_path(old_path);

        assert!(err.to_string().contains("需要 cue"), "{err}");
        std::fs::remove_dir_all(&root).ok();
    }
}
