use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::providers;

#[derive(Args)]
pub struct TransferArgs {
    /// 网盘提供商: dropbox（默认）| baidu | google | onedrive | s3 | sftp
    #[arg(long, default_value = "dropbox")]
    pub provider: String,

    #[command(subcommand)]
    pub action: TransferAction,
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

pub fn run(args: &TransferArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    match &args.action {
        TransferAction::Send {
            file,
            remote,
            output,
        } => {
            let provider = providers::from_name(&args.provider)
                .unwrap_or_else(|| panic!("不支持的提供商: {}", args.provider));

            let remote_path = remote.clone().unwrap_or_else(|| {
                format!("/send/{}", file.rsplit('/').next().unwrap_or("result"))
            });

            match rt.block_on(provider.send(file, &remote_path)) {
                Ok(link) => {
                    if let Err(err) = handle_sent_link(SentLinkInput {
                        provider: &args.provider,
                        file,
                        remote_path: &remote_path,
                        link: &link,
                        output: output.as_deref(),
                    }) {
                        eprintln!("{err}");
                        std::process::exit(1);
                    }
                }
                Err(e) => eprintln!("发送失败: {e}"),
            }
        }
        TransferAction::Receive { source, output } => {
            let local_path = output
                .clone()
                .unwrap_or_else(|| source.rsplit('/').next().unwrap_or("received").to_string());

            let is_url = source.starts_with("http://") || source.starts_with("https://");

            if is_url {
                // 手动模式：从 URL 自动识别提供商
                let provider = providers::detect(source).unwrap_or_else(|| {
                    providers::from_name(&args.provider)
                        .unwrap_or_else(|| panic!("不支持的提供商: {}", args.provider))
                });
                if let Err(e) = rt.block_on(provider.receive(source, &local_path)) {
                    eprintln!("接收失败: {e}");
                }
            } else {
                // 自动模式：使用 --provider 指定的提供商直接拉取
                let provider = providers::from_name(&args.provider)
                    .unwrap_or_else(|| panic!("不支持的提供商: {}", args.provider));
                if let Err(e) = rt.block_on(provider.receive_path(source, &local_path)) {
                    eprintln!("自动接收失败: {e}");
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
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
        sent_at: chrono_now(),
    };

    if let Err(err) = save_delivery_link_record_at(links_path, &record) {
        eprintln!("写入交付链接记录失败: {err}");
    }

    Ok(())
}

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
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut registry = load_delivery_link_registry(path)?;
    registry.insert(record.id.clone(), record.clone());
    let json = serde_json::to_string_pretty(&registry)?;
    std::fs::write(path, json)
}

fn load_delivery_link_registry(path: &Path) -> io::Result<BTreeMap<String, DeliveryLinkRecord>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = std::fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(BTreeMap::new());
    }

    serde_json::from_str(&content).map_err(io::Error::other)
}

fn delivery_links_path() -> PathBuf {
    let catalog_dir = std::env::var("CATALOG_DIR").ok();
    let data_root = std::env::var("DATA_ROOT").ok();
    let path = delivery_links_path_from(catalog_dir.as_deref(), data_root.as_deref());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    path
}

fn delivery_links_path_from(catalog_dir: Option<&str>, data_root: Option<&str>) -> PathBuf {
    let dir = catalog_dir
        .map(PathBuf::from)
        .or_else(|| data_root.map(|root| PathBuf::from(root).join("catalog")))
        .unwrap_or_else(|| PathBuf::from(".quanttide/data/catalog"));
    dir.join("delivery-links.json")
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

fn chrono_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let (y, m, d) = days_to_date(days as i64);
    format!("{y:04}-{m:02}-{d:02} {hours:02}:{minutes:02}:{seconds:02}")
}

fn days_to_date(mut days: i64) -> (i64, u32, u32) {
    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 };
    let era = era / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        dir
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
        let root = temp_dir("qtcloud-transfer-data-root");

        let path = delivery_links_path_from(None, Some(root.to_str().unwrap()));

        assert_eq!(path, root.join("catalog").join("delivery-links.json"));

        std::fs::remove_dir_all(&root).ok();
    }
}
