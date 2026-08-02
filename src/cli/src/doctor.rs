//! 环境检查命令：外部工具 / 数据目录 / 传输凭证（检查三态）。

use clap::Args;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::error::CliError;

#[derive(Args)]
pub struct DoctorArgs {
    /// 只输出诊断报告，即使缺少必需项也返回成功
    #[arg(long)]
    pub no_fail: bool,

    /// 输出 JSON，便于 CI、Studio 或其他脚本读取
    #[arg(long)]
    pub json: bool,

    /// 自动创建常用 .quanttide/data 目录
    #[arg(long)]
    pub fix_dirs: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
// ── 数据模型（CheckStatus / Check / DoctorArgs） ──
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
}

impl Check {
    fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Pass,
            message: message.into(),
        }
    }

    fn warn(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Warn,
            message: message.into(),
        }
    }

    fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Fail,
            message: message.into(),
        }
    }
}

// ── 命令 ──
/// 环境检查命令入口；存在失败项且未 `--no-fail` 时返回错误。
pub fn run(args: &DoctorArgs) -> Result<(), CliError> {
    let dirs = data_dirs();
    let mut checks = Vec::new();

    if args.fix_dirs {
        checks.extend(create_data_dirs(&dirs));
    }
    checks.extend(checks_with_dirs(&dirs));

    if args.json {
        print_output(&render_json_report(&checks));
    } else {
        print_output(&render_report(&checks));
    }

    if has_failures(&checks) && !args.no_fail {
        return Err(CliError::new("doctor 检查存在失败项"));
    }
    Ok(())
}

// ── 检查项构造 ──
/// 构造默认检查项集合（工具 + 目录 + 凭证）。
pub fn default_checks() -> Vec<Check> {
    checks_with_dirs(&data_dirs())
}

fn checks_with_dirs(dirs: &[DataDir]) -> Vec<Check> {
    let mut checks = vec![
        check_command("git", true, "版本记录和协作事实源需要 git"),
        check_command("cargo", true, "CLI 开发和发布需要 cargo"),
        check_command("rustc", true, "CLI 编译需要 rustc"),
        check_command(
            "python3",
            false,
            "process 执行 Python pipeline 时会用到 python3",
        ),
        check_command("bash", false, "process 执行 shell pipeline 时会用到 bash"),
        check_command("cue", true, "pipeline/blueprint/contract 查看命令需要 cue"),
    ];

    for dir in dirs {
        checks.push(check_directory(&dir.path, &dir.name));
    }

    checks.push(check_env_any(
        "DROPBOX_ACCESS_TOKEN",
        &["DROPBOX_ACCESS_TOKEN"],
        false,
        "Dropbox 传输",
    ));
    checks.push(check_env_any(
        "BAIDU_ACCESS_TOKEN",
        &["BAIDU_ACCESS_TOKEN", "BAIDUDRIVE_ACCESS_TOKEN"],
        false,
        "百度网盘传输",
    ));
    checks.push(check_env_any(
        "GOOGLE_DRIVE_ACCESS_TOKEN",
        &["GOOGLE_DRIVE_ACCESS_TOKEN", "GDRIVE_ACCESS_TOKEN"],
        false,
        "Google Drive 传输",
    ));
    checks.push(check_env_any(
        "ONEDRIVE_ACCESS_TOKEN",
        &["ONEDRIVE_ACCESS_TOKEN", "OD_ACCESS_TOKEN"],
        false,
        "OneDrive 传输",
    ));
    checks.push(check_env_any(
        "SFTP_HOST",
        &["SFTP_HOST"],
        false,
        "SFTP 传输",
    ));
    checks.push(check_env_any(
        "AWS",
        &["AWS_PROFILE", "AWS_ACCESS_KEY_ID"],
        false,
        "S3 传输",
    ));

    checks
}

// ── 报告渲染 ──
fn print_output(output: &str) {
    let mut stdout = io::stdout();
    let _ = write!(stdout, "{output}");
    let _ = stdout.flush();
}

/// 渲染人类可读检查报告（不含凭证值）。
pub fn render_report(checks: &[Check]) -> String {
    let mut report = String::from("qtcloud-data doctor\n检查本机 DataOps 环境\n\n");

    for check in checks {
        let status = match check.status {
            CheckStatus::Pass => "PASS",
            CheckStatus::Warn => "WARN",
            CheckStatus::Fail => "FAIL",
        };
        report.push_str(&format!(
            "[{status}] {:<24} {}\n",
            check.name, check.message
        ));
    }

    let failures = checks
        .iter()
        .filter(|check| check.status == CheckStatus::Fail)
        .count();
    let warnings = checks
        .iter()
        .filter(|check| check.status == CheckStatus::Warn)
        .count();

    report.push_str(&format!(
        "\nSummary: {} failed, {} warning(s)\n",
        failures, warnings
    ));

    if failures > 0 {
        report.push_str("Fix FAIL items before running build, test, or data pipeline commands.\n");
    }

    report
}

/// 渲染机器可读 JSON 检查报告（CI / Studio 用）。
pub fn render_json_report(checks: &[Check]) -> String {
    let (failures, warnings) = summary_counts(checks);
    let checks_json: Vec<_> = checks
        .iter()
        .map(|check| {
            serde_json::json!({
                "name": check.name,
                "status": status_json(&check.status),
                "message": check.message,
            })
        })
        .collect();

    let report = serde_json::json!({
        "command": "doctor",
        "summary": {
            "failed": failures,
            "warnings": warnings,
        },
        "checks": checks_json,
    });

    format!(
        "{}\n",
        // Value 序列化为 String 实际不会失败，回退到空报告
        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
    )
}

/// 判断检查结果中是否存在失败项（warning 不计）。
pub fn has_failures(checks: &[Check]) -> bool {
    checks.iter().any(|check| check.status == CheckStatus::Fail)
}

fn summary_counts(checks: &[Check]) -> (usize, usize) {
    let failures = checks
        .iter()
        .filter(|check| check.status == CheckStatus::Fail)
        .count();
    let warnings = checks
        .iter()
        .filter(|check| check.status == CheckStatus::Warn)
        .count();
    (failures, warnings)
}

fn status_json(status: &CheckStatus) -> &'static str {
    match status {
        CheckStatus::Pass => "pass",
        CheckStatus::Warn => "warn",
        CheckStatus::Fail => "fail",
    }
}

struct DataDir {
    name: String,
    path: PathBuf,
}

// ── 数据目录 ──
fn data_dirs() -> Vec<DataDir> {
    data_dirs_with(|name| env::var(name).ok())
}

fn data_dirs_with(lookup: impl Fn(&str) -> Option<String>) -> Vec<DataDir> {
    let data_root = lookup("DATA_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".quanttide/data"));

    vec![
        DataDir {
            name: "DATA_ROOT".to_string(),
            path: data_root.clone(),
        },
        data_dir_with("DRD_DIR", data_root.join("drd"), &lookup),
        data_dir_with("SPEC_DIR", data_root.join("spec"), &lookup),
        data_dir_with("BLUEPRINT_DIR", data_root.join("blueprint"), &lookup),
        data_dir_with("CONTRACT_DIR", data_root.join("contract"), &lookup),
        data_dir_with("PIPELINE_DIR", data_root.join("pipeline"), &lookup),
        data_dir_with("CATALOG_DIR", data_root.join("catalog"), &lookup),
    ]
}

fn data_dir_with(
    env_name: &str,
    default_path: impl Into<PathBuf>,
    lookup: impl Fn(&str) -> Option<String>,
) -> DataDir {
    let path = lookup(env_name)
        .map(PathBuf::from)
        .unwrap_or_else(|| default_path.into());
    DataDir {
        name: env_name.to_string(),
        path,
    }
}

fn create_data_dirs(dirs: &[DataDir]) -> Vec<Check> {
    dirs.iter()
        .map(|dir| match fs::create_dir_all(&dir.path) {
            Ok(_) => Check::pass(&dir.name, format!("{} ready", dir.path.display())),
            Err(err) => Check::fail(
                &dir.name,
                format!("{} could not be created: {err}", dir.path.display()),
            ),
        })
        .collect()
}

// ── 检查函数 ──
fn check_command(command: &str, required: bool, purpose: &str) -> Check {
    if command_exists(command) {
        Check::pass(command, format!("{purpose}: found"))
    } else if required {
        Check::fail(command, format!("{purpose}: not found in PATH"))
    } else {
        Check::warn(command, format!("{purpose}: not found in PATH"))
    }
}

fn check_directory(path: &Path, name: &str) -> Check {
    if path.is_dir() {
        Check::pass(name, format!("{} exists", path.display()))
    } else {
        Check::warn(
            name,
            format!(
                "{} missing; create it when this workflow is used",
                path.display()
            ),
        )
    }
}

fn check_env_any(display_name: &str, env_names: &[&str], required: bool, purpose: &str) -> Check {
    check_env_any_with(display_name, env_names, required, purpose, |name| {
        env::var(name).ok()
    })
}

fn check_env_any_with(
    display_name: &str,
    env_names: &[&str],
    required: bool,
    purpose: &str,
    lookup: impl Fn(&str) -> Option<String>,
) -> Check {
    let configured_name = env_names.iter().find(|name| {
        lookup(name)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    });

    if let Some(name) = configured_name {
        Check::pass(display_name, format!("{purpose}: configured via {name}"))
    } else if required {
        Check::fail(
            display_name,
            format!("{purpose}: missing {}", env_names.join(" / ")),
        )
    } else {
        Check::warn(
            display_name,
            format!("{purpose}: missing {}", env_names.join(" / ")),
        )
    }
}

fn command_exists(command: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };

    let path_exts = executable_extensions();

    env::split_paths(&path).any(|dir| {
        let direct = dir.join(command);
        if direct.is_file() {
            return true;
        }

        path_exts
            .iter()
            .any(|ext| dir.join(format!("{command}{ext}")).is_file())
    })
}

fn executable_extensions() -> Vec<String> {
    if cfg!(windows) {
        env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
            .split(';')
            .filter(|ext| !ext.trim().is_empty())
            .map(|ext| ext.to_ascii_lowercase())
            .collect()
    } else {
        vec![String::new()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;

    #[test]
    fn render_report_shows_summary_without_secret_values() {
        let checks = vec![
            Check::pass("DROPBOX_ACCESS_TOKEN", "Dropbox 传输: configured"),
            Check::warn(
                "cue",
                "旧 pipeline/contract 查看命令仍可能依赖 cue: not found in PATH",
            ),
        ];

        let report = render_report(&checks);

        assert!(report.contains("qtcloud-data doctor"));
        assert!(report.contains("检查本机 DataOps 环境"));
        assert!(report.contains("[PASS] DROPBOX_ACCESS_TOKEN"));
        assert!(report.contains("Summary: 0 failed, 1 warning(s)"));
        assert!(!report.contains("secret"));
    }

    #[test]
    fn data_dir_uses_env_override_when_present() {
        let dir = data_dir_with("BLUEPRINT_DIR", ".quanttide/data/blueprint", |name| {
            if name == "BLUEPRINT_DIR" {
                Some("custom/blueprints".to_string())
            } else {
                None
            }
        });

        assert_eq!(dir.name, "BLUEPRINT_DIR");
        assert_eq!(dir.path, PathBuf::from("custom/blueprints"));
    }

    #[test]
    fn data_dir_uses_default_when_env_missing() {
        let dir = data_dir_with("PIPELINE_DIR", ".quanttide/data/pipeline", |_| None);

        assert_eq!(dir.name, "PIPELINE_DIR");
        assert_eq!(dir.path, PathBuf::from(".quanttide/data/pipeline"));
    }

    #[test]
    fn data_dirs_use_data_root_for_default_children() {
        let data_root = PathBuf::from("custom/root");
        let dirs = data_dirs_with(|name| {
            if name == "DATA_ROOT" {
                Some("custom/root".to_string())
            } else {
                None
            }
        });

        let drd_path = data_root.join("drd");
        let spec_path = data_root.join("spec");

        assert!(
            dirs.iter()
                .any(|dir| dir.name == "DRD_DIR" && dir.path == drd_path)
        );
        assert!(
            dirs.iter()
                .any(|dir| dir.name == "SPEC_DIR" && dir.path == spec_path)
        );
    }

    #[test]
    fn check_env_any_accepts_alias_without_printing_value() {
        let check = check_env_any_with(
            "GOOGLE_DRIVE_ACCESS_TOKEN",
            &["GOOGLE_DRIVE_ACCESS_TOKEN", "GDRIVE_ACCESS_TOKEN"],
            false,
            "Google Drive 传输",
            |name| {
                if name == "GDRIVE_ACCESS_TOKEN" {
                    Some("top-secret-token".to_string())
                } else {
                    None
                }
            },
        );

        assert_eq!(check.status, CheckStatus::Pass);
        assert!(check.message.contains("GDRIVE_ACCESS_TOKEN"));
        assert!(!check.message.contains("top-secret-token"));
    }

    #[test]
    fn check_env_any_warns_when_optional_env_missing() {
        let check = check_env_any_with("SFTP_HOST", &["SFTP_HOST"], false, "SFTP 传输", |_| None);

        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.message.contains("SFTP_HOST"));
    }

    #[test]
    fn check_env_any_fails_when_required_env_missing() {
        let check = check_env_any_with("REQUIRED", &["REQUIRED"], true, "必需配置", |_| None);

        assert_eq!(check.status, CheckStatus::Fail);
    }

    #[test]
    fn has_failures_detects_failed_checks() {
        let checks = vec![
            Check::pass("git", "found"),
            Check::fail("cargo", "not found in PATH"),
        ];

        assert!(has_failures(&checks));
    }

    fn fake_path_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn command_exists_respects_path_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = fake_path_dir("qtcloud-doctor-path");
        let fake = dir.join("fakecmd");
        std::fs::write(&fake, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        unsafe {
            std::env::set_var("PATH", &dir);
        }
        assert!(command_exists("fakecmd"));
        assert!(!command_exists("ghostcmd"));
        unsafe {
            std::env::remove_var("PATH");
        }

        // PATH 未设置时返回 false（不 panic）
        assert!(!command_exists("fakecmd"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_command_status_reflects_required_and_presence() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = fake_path_dir("qtcloud-doctor-check-cmd");
        let fake = dir.join("toolx");
        std::fs::write(&fake, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        unsafe {
            std::env::set_var("PATH", &dir);
        }
        // 存在：无论 required 与否都 pass
        assert_eq!(
            check_command("toolx", true, "测试工具").status,
            CheckStatus::Pass
        );
        // 缺失 + required → fail
        assert_eq!(
            check_command("ghostcmd", true, "测试工具").status,
            CheckStatus::Fail
        );
        // 缺失 + optional → warn
        assert_eq!(
            check_command("ghostcmd", false, "测试工具").status,
            CheckStatus::Warn
        );
        unsafe {
            std::env::remove_var("PATH");
        }

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_directory_exists_passes_and_missing_warns() {
        let root = crate::test_support::temp_dir("qtcloud-doctor-check-dir");
        let exists = root.join("exists");
        std::fs::create_dir_all(&exists).unwrap();
        let missing = root.join("missing");

        assert_eq!(check_directory(&exists, "EXISTS").status, CheckStatus::Pass);
        assert_eq!(
            check_directory(&missing, "MISSING").status,
            CheckStatus::Warn
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn checks_with_dirs_builds_full_check_list() {
        let root = crate::test_support::temp_dir("qtcloud-doctor-checks");
        let dirs = vec![
            DataDir {
                name: "DRD".to_string(),
                path: root.join("drd"),
            },
            DataDir {
                name: "SPEC".to_string(),
                path: root.join("spec"),
            },
        ];

        let checks = checks_with_dirs(&dirs);
        let names: Vec<&str> = checks.iter().map(|c| c.name.as_str()).collect();

        // 6 个工具检查
        for tool in ["git", "cargo", "rustc", "python3", "bash", "cue"] {
            assert!(names.contains(&tool), "缺工具检查: {tool}");
        }
        // 2 个目录检查
        assert!(names.contains(&"DRD"));
        assert!(names.contains(&"SPEC"));
        // 6 个 env 检查
        for env in [
            "DROPBOX_ACCESS_TOKEN",
            "BAIDU_ACCESS_TOKEN",
            "GOOGLE_DRIVE_ACCESS_TOKEN",
            "ONEDRIVE_ACCESS_TOKEN",
            "SFTP_HOST",
            "AWS",
        ] {
            assert!(names.contains(&env), "缺 env 检查: {env}");
        }
        assert_eq!(checks.len(), 6 + 2 + 6);

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn render_json_report_produces_machine_readable_json() {
        let checks = vec![Check::pass("git", "found"), Check::fail("cue", "not found")];
        let json = render_json_report(&checks);
        assert!(json.contains("\"status\""), "{json}");
        assert!(json.contains("\"pass\""), "{json}");
        assert!(json.contains("\"fail\""), "{json}");
        // 不泄露 secret 值（这里没有 secret）
        assert!(!json.contains("SECRET"), "{json}");
    }

    #[test]
    fn create_data_dirs_creates_directories_and_reports() {
        let root = crate::test_support::temp_dir("qtcloud-doctor-create-dirs");
        let target = root.join("sub");
        let dirs = vec![DataDir {
            name: "SUB".to_string(),
            path: target.clone(),
        }];

        let checks = create_data_dirs(&dirs);
        assert!(target.is_dir(), "目录应被创建");
        assert_eq!(checks[0].status, CheckStatus::Pass);
        assert_eq!(checks[0].name, "SUB");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn has_failures_allows_warnings() {
        let checks = vec![
            Check::pass("git", "found"),
            Check::warn("DROPBOX_ACCESS_TOKEN", "missing"),
        ];

        assert!(!has_failures(&checks));
    }
}
