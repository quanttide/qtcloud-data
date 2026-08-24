//! 管道定义查看命令（list / show）。

use clap::{Args, Subcommand};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::OutputMode;
use crate::error::CliError;
use crate::util::collect_defined_names;

const PIPELINE_EXTS: [&str; 4] = [".yaml", ".yml", ".cue", ".json"];

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

/// 管道查看命令入口（list / show）。
pub fn run(args: &PipelineArgs) -> Result<(), CliError> {
    run_with_mode(args, OutputMode::Text)
}

pub fn run_with_mode(args: &PipelineArgs, mode: OutputMode) -> Result<(), CliError> {
    let dir =
        std::env::var("PIPELINE_DIR").unwrap_or_else(|_| ".quanttide/data/pipeline".to_string());

    match &args.action {
        PipelineAction::List => cmd_list(&dir, mode),
        PipelineAction::Show { name } => cmd_show(&dir, name, mode),
    }
}

fn cmd_list(dir: &str, mode: OutputMode) -> Result<(), CliError> {
    let dir_path = Path::new(dir);
    if dir_path.is_dir() {
        let names = definition_names(dir_path);
        render_list(&names, mode);
        return Ok(());
    }

    let output = cue_export(&["export", "--out", "json", dir])
        .map_err(|_| CliError::new("需要 cue CLI".to_string()))?;
    if !output.status.success() {
        return Err(CliError::new(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| CliError::new(format!("cue 输出不是合法 JSON: {e}")))?;
    let names = collect_defined_names(&value);
    render_list(&names, mode);
    Ok(())
}

fn cmd_show(dir: &str, name: &str, mode: OutputMode) -> Result<(), CliError> {
    let dir_path = Path::new(dir);
    if let Some(path) = find_definition(dir_path, name) {
        let content = std::fs::read_to_string(&path)
            .map_err(|err| CliError::new(format!("读取 Pipeline 失败: {err}")))?;
        render_show(name, &content, mode)?;
        return Ok(());
    }

    let key = crate::util::to_camel(name);
    let output = cue_export(&["export", "--out", "json", "--expression", &key, dir])
        .map_err(|_| CliError::new(format!("找不到 Pipeline: {name}")))?;
    if !output.status.success() {
        return Err(CliError::new(format!("找不到 Pipeline: {name}")));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| CliError::new(format!("cue 输出不是合法 JSON: {e}")))?;
    let content = serde_json::to_string_pretty(&value)
        .map_err(|e| CliError::new(format!("序列化失败: {e}")))?;
    render_show(name, &content, mode)?;
    Ok(())
}

fn render_list(names: &[String], mode: OutputMode) {
    match mode {
        OutputMode::Text => {
            println!("可用的 Pipeline:");
            for name in names {
                println!("  - {name}");
            }
        }
        OutputMode::Json => println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "command": "pipeline list",
                "items": names,
            })
        ),
    }
}

fn render_show(name: &str, content: &str, mode: OutputMode) -> Result<(), CliError> {
    match mode {
        OutputMode::Text => println!("{content}"),
        OutputMode::Json => {
            let definition: serde_json::Value = serde_yaml::from_str(content)
                .or_else(|_| serde_json::from_str(content))
                .map_err(|err| CliError::new(format!("Pipeline 不是合法结构化数据: {err}")))?;
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "command": "pipeline show",
                    "name": name,
                    "pipeline": definition,
                })
            );
        }
    }
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
            for ext in PIPELINE_EXTS {
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
    for ext in PIPELINE_EXTS {
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

    fn fake_cue_env() -> (crate::test_support::TempDir, Option<std::ffi::OsString>) {
        let root = temp_dir("qtcloud-pipeline-fake-cue");
        let bin = root.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let cue_bin = if cfg!(windows) { "cue.cmd" } else { "cue" };
        let cue_script = if cfg!(windows) {
            "@echo off\r\nif \"%4\"==\"--expression\" goto show\r\necho {\"pipe1\":{\"name\":\"pipe1\"}}\r\nexit /b 0\r\n:show\r\necho {\"name\":\"pipe1\"}\r\nexit /b 0\r\n"
        } else {
            "#!/bin/sh\ncase \"$*\" in\n  *--expression*) echo '{\"name\": \"pipe1\"}' ;;\n  *) echo '{\"pipe1\": {\"name\": \"pipe1\"}}' ;;\nesac\n"
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

        let result = cmd_list(root.to_str().unwrap(), OutputMode::Text);
        restore_path(old_path);

        assert!(result.is_ok(), "{result:?}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cmd_show_prints_expression_json() {
        let _guard = ENV_LOCK.lock().unwrap();
        let (root, old_path) = fake_cue_env();

        let result = cmd_show(root.to_str().unwrap(), "pipe1", OutputMode::Text);
        restore_path(old_path);

        assert!(result.is_ok(), "{result:?}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cmd_list_reads_yaml_files_without_cue() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-pipeline-file-list");
        std::fs::write(root.join("normalize.yaml"), "name: normalize\n").unwrap();
        std::fs::write(root.join("review.json"), "{\"name\":\"review\"}\n").unwrap();
        let empty_bin = root.join("empty-bin");
        std::fs::create_dir_all(&empty_bin).unwrap();

        let old_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", &empty_bin);
        }
        let result = cmd_list(root.to_str().unwrap(), OutputMode::Text);
        restore_path(old_path);

        assert!(result.is_ok(), "{result:?}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn cmd_show_reads_yaml_file_without_cue() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-pipeline-file-show");
        std::fs::write(root.join("normalize.yaml"), "name: normalize\n").unwrap();
        let empty_bin = root.join("empty-bin");
        std::fs::create_dir_all(&empty_bin).unwrap();

        let old_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", &empty_bin);
        }
        let result = cmd_show(root.to_str().unwrap(), "normalize", OutputMode::Text);
        restore_path(old_path);

        assert!(result.is_ok(), "{result:?}");
        std::fs::remove_dir_all(&root).ok();
    }
}
