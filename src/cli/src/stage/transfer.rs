//! 传输命令与服务函数：send / receive（6 平台，进程内 + QTDATA_CLI 委派）。

use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

use crate::error::CliError;
use crate::registry;
use crate::storage;
use crate::util;

#[derive(Args)]
pub struct TransferArgs {
    /// 网盘提供商: dropbox（默认）| baidu | google | onedrive | s3 | sftp
    #[arg(long, value_enum, default_value_t = TransferProvider::Dropbox)]
    pub provider: TransferProvider,

    #[command(subcommand)]
    pub action: TransferAction,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum TransferProvider {
    #[default]
    Dropbox,
    Baidu,
    Google,
    Onedrive,
    S3,
    Sftp,
}

impl TransferProvider {
    pub fn parse(value: &str) -> Result<Self, CliError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "dropbox" => Ok(Self::Dropbox),
            "baidu" | "baidudrive" => Ok(Self::Baidu),
            "google" | "googledrive" => Ok(Self::Google),
            "onedrive" => Ok(Self::Onedrive),
            "s3" => Ok(Self::S3),
            "sftp" => Ok(Self::Sftp),
            other => Err(CliError::new(format!(
                "不支持的提供商: {other}，可选: dropbox / baidu / google / onedrive / s3 / sftp"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dropbox => "dropbox",
            Self::Baidu => "baidu",
            Self::Google => "google",
            Self::Onedrive => "onedrive",
            Self::S3 => "s3",
            Self::Sftp => "sftp",
        }
    }

    pub const fn storage_name(self) -> &'static str {
        self.as_str()
    }

    pub fn storage(self) -> Option<Box<dyn storage::Storage>> {
        storage::from_name(self.storage_name())
    }
}

#[derive(Subcommand)]
pub enum TransferAction {
    /// 发送文件：上传到网盘并生成分享链接，把链接给对方
    Send {
        /// 本地文件路径
        file: String,
        /// 远程路径，不指定则使用文件名
        remote: Option<String>,
        /// 将链接写入文件（不指定则直接打印到终端）
        #[arg(long)]
        output: Option<String>,
    },
    /// 接收文件：从共享链接下载（手动）或直接拉取（自动）
    ///
    /// 手动模式：传入分享链接（http/https），自动识别提供商
    /// 自动模式：传入远程路径，配合 --provider 使用
    Receive {
        /// 分享链接（http/https）或远程路径
        source: String,
        /// 本地保存路径，不指定则自动取名
        #[arg(long)]
        output: Option<String>,
    },
}

// ── 命令层 ──
/// 传输命令入口（send / receive）。
pub fn run(args: &TransferArgs) -> Result<(), CliError> {
    match &args.action {
        TransferAction::Send {
            file,
            remote,
            output,
        } => {
            let output_path = output.as_deref().map(Path::new);
            send_with_provider(file, remote.as_deref(), output_path, args.provider)
                .map_err(|err| CliError::new(format!("发送失败: {err}")))?;
            Ok(())
        }
        TransferAction::Receive { source, output } => {
            let output_path = output
                .as_deref()
                .map(Path::new)
                .unwrap_or_else(|| Path::new(source.rsplit('/').next().unwrap_or("received")));
            receive_with_provider(source, output_path, args.provider)?;
            Ok(())
        }
    }
}

/// 进程内传输服务：接收数据到本地文件。
///
/// `QTDATA_CLI` 环境变量设置时委派给外部 CLI（测试与部署逃生舱），
/// 否则走进程内 provider（替代 process 自我 re-exec）。
// ── 服务函数（receive / send / 委派） ──
/// 进程内接收服务：从 URL 或远程路径下载到本地文件。
pub fn receive(source: &str, output: &Path, provider: &str) -> Result<(), CliError> {
    let provider = TransferProvider::parse(provider)?;
    receive_with_provider(source, output, provider)
}

pub fn receive_with_provider(
    source: &str,
    output: &Path,
    provider: TransferProvider,
) -> Result<(), CliError> {
    let output_str = output.to_string_lossy().to_string();

    if let Ok(bin) = std::env::var("QTDATA_CLI") {
        return run_delegated(
            &bin,
            &["transfer", "receive", source, "--output", &output_str],
        );
    }

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::new(format!("创建运行时失败: {e}")))?;
    let is_url = source.starts_with("http://") || source.starts_with("https://");
    if is_url {
        // 手动模式：从 URL 自动识别提供商
        let p = storage::detect(source)
            .or_else(|| provider.storage())
            .ok_or_else(|| CliError::new(format!("不支持的提供商: {}", provider.as_str())))?;
        return rt
            .block_on(p.receive(source, &output_str))
            .map_err(|e| CliError::new(format!("接收失败: {e}")));
    }
    // 自动模式：使用指定提供商直接拉取
    let p = provider
        .storage()
        .ok_or_else(|| CliError::new(format!("不支持的提供商: {}", provider.as_str())))?;
    rt.block_on(p.receive_path(source, &output_str))
        .map_err(|e| CliError::new(format!("自动接收失败: {e}")))
}

/// 进程内传输服务：发送文件并返回交付链接。
///
/// `QTDATA_CLI` 环境变量设置时委派给外部 CLI，否则走进程内 provider。
pub fn send(
    file: &str,
    remote: Option<&str>,
    output: Option<&Path>,
    provider: &str,
) -> Result<String, CliError> {
    let provider = TransferProvider::parse(provider)?;
    send_with_provider(file, remote, output, provider)
}

pub fn send_with_provider(
    file: &str,
    remote: Option<&str>,
    output: Option<&Path>,
    provider: TransferProvider,
) -> Result<String, CliError> {
    let remote_path = remote
        .map(str::to_string)
        .unwrap_or_else(|| format!("/send/{}", file.rsplit('/').next().unwrap_or("result")));

    if let Ok(bin) = std::env::var("QTDATA_CLI") {
        return send_delegated(&bin, file, output);
    }

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::new(format!("创建运行时失败: {e}")))?;
    let p = provider
        .storage()
        .ok_or_else(|| format!("不支持的提供商: {}", provider.as_str()))?;
    let link = rt
        .block_on(p.send(file, &remote_path))
        .map_err(|e| format!("发送失败: {e}"))?;
    handle_sent_link(SentLinkInput {
        provider: provider.as_str(),
        file,
        remote_path: &remote_path,
        link: &link,
        output: output.map(|p| p.to_string_lossy().to_string()).as_deref(),
    })?;
    Ok(link)
}

fn run_delegated(bin: &str, args: &[&str]) -> Result<(), CliError> {
    let status = std::process::Command::new(bin)
        .args(args)
        .status()
        .map_err(|e| CliError::new(format!("执行 {bin} 失败: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(CliError::new(format!("{bin} 执行失败: {status}")))
    }
}

fn send_delegated(bin: &str, file: &str, output: Option<&Path>) -> Result<String, CliError> {
    match output {
        Some(out) => {
            run_delegated(
                bin,
                &["transfer", "send", file, "--output", &out.to_string_lossy()],
            )?;
            std::fs::read_to_string(out)
                .map_err(|e| CliError::new(format!("读取交付链接失败: {e}")))
        }
        None => {
            let output = std::process::Command::new(bin)
                .args(["transfer", "send", file])
                .output()
                .map_err(|e| CliError::new(format!("执行 {bin} 失败: {e}")))?;
            if !output.status.success() {
                return Err(CliError::new(format!("{bin} 执行失败: {}", output.status)));
            }
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
// ── 记录模型（DeliveryLinkRecord / SentLinkInput） ──
struct DeliveryLinkRecord {
    id: String,
    provider: String,
    file_path: String,
    remote_path: String,
    link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_path: Option<String>,
    status: String,
    sent_at: String,
}

struct SentLinkInput<'a> {
    provider: &'a str,
    file: &'a str,
    remote_path: &'a str,
    link: &'a str,
    output: Option<&'a str>,
}

fn handle_sent_link(input: SentLinkInput<'_>) -> Result<(), String> {
    let links_path = delivery_links_path();
    handle_sent_link_in(input, &links_path)
}

fn handle_sent_link_in(input: SentLinkInput<'_>, links_path: &Path) -> Result<(), String> {
    let link_path = if let Some(out) = input.output {
        write_link_file(out, input.link)?;
        println!("✓ 链接已写入: {out}");
        Some(path_for_record(out))
    } else {
        println!("{}", input.link);
        None
    };

    let record = DeliveryLinkRecord {
        id: new_delivery_link_id(input.file),
        provider: input.provider.to_string(),
        file_path: path_for_record(input.file),
        remote_path: input.remote_path.to_string(),
        link: input.link.to_string(),
        link_path,
        status: "sent".to_string(),
        sent_at: util::now_utc(),
    };

    if let Err(err) = save_delivery_link_record_at(links_path, &record) {
        eprintln!("写入交付链接记录失败: {err}");
    }

    Ok(())
}

// ── 链接文件与工具 ──
fn write_link_file(path: &str, link: &str) -> Result<(), String> {
    let path = Path::new(path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|err| format!("创建链接文件目录失败: {err}"))?;
    }

    std::fs::write(path, link).map_err(|err| format!("写入链接文件失败: {err}"))
}

fn save_delivery_link_record_at(path: &Path, record: &DeliveryLinkRecord) -> io::Result<()> {
    let mut registry = registry::Registry::open(path)?;
    registry.insert(record.id.clone(), record.clone())
}

fn delivery_links_path() -> PathBuf {
    util::catalog_dir().join("delivery-links.json")
}

fn new_delivery_link_id(file: &str) -> String {
    let stem = Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(sanitize_id)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "delivery".to_string());
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{stem}-{millis}")
}

// ── id 与路径工具 ──
fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn path_for_record(path: &str) -> String {
    let path = PathBuf::from(path);
    path.canonicalize()
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::test_support::temp_dir;
    use crate::test_support::write_script;

    fn fake_qtdata_path(root: &Path) -> PathBuf {
        if cfg!(windows) {
            root.join("fake-qtdata.cmd")
        } else {
            root.join("fake-qtdata.sh")
        }
    }

    #[test]
    fn handle_sent_link_writes_output_file_and_delivery_record() {
        let root = temp_dir("qtcloud-transfer-link-record");
        let catalog_dir = root.join("catalog");
        let data_file = root.join("report.csv");
        let link_file = root.join("share-link.txt");
        std::fs::create_dir_all(&catalog_dir).unwrap();
        std::fs::write(&data_file, "a,b\n1,2\n").unwrap();

        handle_sent_link_in(
            SentLinkInput {
                provider: "dropbox",
                file: data_file.to_str().unwrap(),
                remote_path: "/send/report.csv",
                link: "https://delivery.example/report.csv",
                output: Some(link_file.to_str().unwrap()),
            },
            &catalog_dir.join("delivery-links.json"),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(&link_file).unwrap(),
            "https://delivery.example/report.csv"
        );

        let content = std::fs::read_to_string(catalog_dir.join("delivery-links.json")).unwrap();
        let registry: serde_json::Value = serde_json::from_str(&content).unwrap();
        let records = registry.as_object().unwrap();
        assert_eq!(records.len(), 1);

        let record = records.values().next().unwrap();
        assert_eq!(record["provider"], "dropbox");
        assert_eq!(record["remote_path"], "/send/report.csv");
        assert_eq!(record["link"], "https://delivery.example/report.csv");
        assert_eq!(record["status"], "sent");
        assert!(
            record["file_path"]
                .as_str()
                .unwrap()
                .ends_with("report.csv")
        );
        assert!(
            record["link_path"]
                .as_str()
                .unwrap()
                .ends_with("share-link.txt")
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn delivery_links_path_uses_data_root_when_catalog_dir_missing() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-transfer-data-root");

        unsafe {
            std::env::remove_var("CATALOG_DIR");
            std::env::set_var("DATA_ROOT", &root);
        }
        let path = delivery_links_path();
        unsafe {
            std::env::remove_var("DATA_ROOT");
        }

        assert_eq!(path, root.join("catalog").join("delivery-links.json"));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn receive_delegates_to_external_cli_when_qtdata_cli_set() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-transfer-receive-delegated");
        let script = fake_qtdata_path(&root);
        let script_content = if cfg!(windows) {
            "@echo off\r\necho downloaded>%5\r\nexit /b 0\r\n"
        } else {
            "#!/bin/sh\nout=''\nwhile [ $# -gt 0 ]; do\n  case \"$1\" in\n    --output) out=\"$2\"; shift 2 ;;\n    *) shift ;;\n  esac\ndone\n[ -n \"$out\" ] && echo downloaded > \"$out\"\nexit 0\n"
        };
        write_script(&script, script_content);
        let out = root.join("received.csv");

        unsafe {
            std::env::set_var("QTDATA_CLI", &script);
        }
        let result = receive("https://share.example/file.csv", &out, "dropbox");
        unsafe {
            std::env::remove_var("QTDATA_CLI");
        }

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(
            std::fs::read_to_string(&out).unwrap().trim_end(),
            "downloaded"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn receive_reports_delegated_cli_failure() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-transfer-receive-fail");
        let script = fake_qtdata_path(&root);
        let script_content = if cfg!(windows) {
            "@echo off\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nexit 1\n"
        };
        write_script(&script, script_content);
        let out = root.join("received.csv");

        unsafe {
            std::env::set_var("QTDATA_CLI", &script);
        }
        let result = receive("https://share.example/file.csv", &out, "dropbox");
        unsafe {
            std::env::remove_var("QTDATA_CLI");
        }

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("执行失败"));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn send_delegates_and_reads_link_from_output_file() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-transfer-send-delegated");
        let script = fake_qtdata_path(&root);
        let script_content = if cfg!(windows) {
            "@echo off\r\n<nul set /p=\"https://delivery.example/link\">%5\r\nexit /b 0\r\n"
        } else {
            "#!/bin/sh\nout=''\nwhile [ $# -gt 0 ]; do\n  case \"$1\" in\n    --output) out=\"$2\"; shift 2 ;;\n    *) shift ;;\n  esac\ndone\nprintf '%s' 'https://delivery.example/link' > \"$out\"\nexit 0\n"
        };
        write_script(&script, script_content);
        let file = root.join("data.csv");
        std::fs::write(&file, "a,b\n").unwrap();
        let link_out = root.join("link.txt");

        unsafe {
            std::env::set_var("QTDATA_CLI", &script);
        }
        let result = send(
            file.to_str().unwrap(),
            Some("/send/data.csv"),
            Some(&link_out),
            "dropbox",
        );
        unsafe {
            std::env::remove_var("QTDATA_CLI");
        }

        assert_eq!(result.unwrap(), "https://delivery.example/link");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn send_without_output_reads_link_from_stdout() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-transfer-send-stdout");
        let script = fake_qtdata_path(&root);
        let script_content = if cfg!(windows) {
            "@echo off\r\necho https://delivery.example/from-stdout\r\nexit /b 0\r\n"
        } else {
            "#!/bin/sh\necho 'https://delivery.example/from-stdout'\nexit 0\n"
        };
        write_script(&script, script_content);
        let file = root.join("data.csv");
        std::fs::write(&file, "a,b\n").unwrap();

        unsafe {
            std::env::set_var("QTDATA_CLI", &script);
        }
        let result = send(
            file.to_str().unwrap(),
            Some("/send/data.csv"),
            None,
            "dropbox",
        );
        unsafe {
            std::env::remove_var("QTDATA_CLI");
        }

        assert_eq!(result.unwrap(), "https://delivery.example/from-stdout");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn receive_rejects_unknown_provider() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-transfer-unknown-provider");
        let out = root.join("x.csv");
        let err = receive("https://example.invalid/share/file.csv", &out, "unknown").unwrap_err();
        assert!(err.to_string().contains("不支持的提供商"), "{err}");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn send_rejects_unknown_provider() {
        let _guard = ENV_LOCK.lock().unwrap();
        let err = send("/tmp/qtcloud-nonexistent.csv", None, None, "unknown").unwrap_err();
        assert!(err.to_string().contains("不支持的提供商"), "{err}");
    }

    #[test]
    fn sanitize_id_replaces_invalid_chars_and_trims_dashes() {
        assert_eq!(sanitize_id("report-2024.csv"), "report-2024-csv");
        assert_eq!(sanitize_id("annual report v2"), "annual-report-v2");
        assert_eq!(sanitize_id("--data--"), "data");
        assert_eq!(sanitize_id("数据表"), "");
        assert_eq!(sanitize_id("a_b.c"), "a_b-c");
    }

    #[test]
    fn new_delivery_link_id_builds_sanitized_stem_with_timestamp() {
        let id = new_delivery_link_id("annual report.csv");
        assert!(id.starts_with("annual-report-"), "{id}");
        let suffix = id.trim_start_matches("annual-report-");
        assert!(suffix.chars().all(|c| c.is_ascii_digit()), "{id}");

        let fallback = new_delivery_link_id("无扩展名路径");
        assert!(fallback.starts_with("delivery-"), "{fallback}");
    }

    #[test]
    fn path_for_record_canonicalizes_existing_file() {
        let root = temp_dir("qtcloud-transfer-path-record");
        let file = root.join("report.csv");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&file, "a,b\n").unwrap();

        let canonical = path_for_record(file.to_str().unwrap());
        assert!(std::path::Path::new(&canonical).is_absolute());
        assert!(canonical.ends_with("report.csv"), "{canonical}");

        // 不存在的文件回退到原始路径
        let missing = root.join("missing.csv");
        let fallback = path_for_record(missing.to_str().unwrap());
        assert!(fallback.ends_with("missing.csv"), "{fallback}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn provider_enum_maps_cli_values_and_storage_aliases() {
        assert_eq!(
            TransferProvider::parse("dropbox").unwrap(),
            TransferProvider::Dropbox
        );
        assert_eq!(
            TransferProvider::parse("google").unwrap(),
            TransferProvider::Google
        );
        assert_eq!(
            TransferProvider::parse("sftp").unwrap(),
            TransferProvider::Sftp
        );
        assert_eq!(TransferProvider::Google.as_str(), "google");
        assert_eq!(TransferProvider::Google.storage_name(), "google");
        assert!(TransferProvider::parse("unknown").is_err());
    }

    #[test]
    fn provider_enum_resolves_registered_storage() {
        for provider in [
            TransferProvider::Dropbox,
            TransferProvider::Baidu,
            TransferProvider::Google,
            TransferProvider::Onedrive,
            TransferProvider::S3,
            TransferProvider::Sftp,
        ] {
            assert!(provider.storage().is_some(), "{} 未注册", provider.as_str());
        }
    }
}
