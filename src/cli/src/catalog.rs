use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

use crate::store;

#[derive(Args)]
pub struct CatalogArgs {
    #[command(subcommand)]
    pub action: CatalogAction,
}

#[derive(Subcommand)]
pub enum CatalogAction {
    /// 列出 catalog 中的所有 volume
    List,
    /// 查看 volume 详情
    Show {
        /// volume 名称
        name: String,
    },
    /// 注册一个文件到 catalog
    Add {
        /// 文件路径
        path: String,
        /// volume 名称（不指定则用文件名）
        #[arg(long)]
        name: Option<String>,
        /// 来源 provider
        #[arg(long)]
        provider: Option<String>,
        /// 来源 URL
        #[arg(long)]
        source: Option<String>,
    },
    /// 删除 volume
    Rm {
        /// volume 名称
        name: String,
    },
}

/// Volume 状态，序列化保持既有字符串（`registry.json` 落盘格式不变）。
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeStatus {
    Received,
    Processing,
    Processed,
    Delivered,
    /// 未知状态（兼容旧数据或未来新增状态）。
    #[serde(other)]
    Unknown,
}

impl Default for VolumeStatus {
    fn default() -> Self {
        VolumeStatus::Received
    }
}

impl fmt::Display for VolumeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            VolumeStatus::Received => "received",
            VolumeStatus::Processing => "processing",
            VolumeStatus::Processed => "processed",
            VolumeStatus::Delivered => "delivered",
            VolumeStatus::Unknown => "unknown",
        };
        f.write_str(text)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Volume {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub received_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default)]
    pub status: VolumeStatus,
}

pub struct RegisterVolume<'a> {
    pub path: &'a str,
    pub name: Option<&'a str>,
    pub provider: Option<&'a str>,
    pub source: Option<&'a str>,
    pub status: VolumeStatus,
}

fn registry_path() -> PathBuf {
    store::catalog_dir().join("registry.json")
}

fn open_registry() -> store::Registry<Volume> {
    store::Registry::open(&registry_path()).unwrap_or_default()
}

pub fn register_volume(input: RegisterVolume<'_>) -> Result<Volume, String> {
    let path = PathBuf::from(input.path);
    if !path.exists() {
        return Err(format!("文件不存在: {}", input.path));
    }

    let meta = std::fs::metadata(&path).map_err(|err| format!("读取文件元数据失败: {err}"))?;

    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let volume_name = input
        .name
        .map(|name| name.to_string())
        .unwrap_or_else(|| file_name);

    let volume = Volume {
        name: volume_name.clone(),
        path: path
            .canonicalize()
            .unwrap_or(path)
            .to_string_lossy()
            .to_string(),
        size: meta.len(),
        received_at: store::now_utc(),
        provider: input.provider.map(|provider| provider.to_string()),
        source: input.source.map(|source| source.to_string()),
        status: input.status,
    };

    let mut registry = open_registry();
    registry
        .insert(volume_name, volume.clone())
        .map_err(|err| format!("写入 registry 失败: {err}"))?;

    Ok(volume)
}

pub fn run(args: &CatalogArgs) {
    match &args.action {
        CatalogAction::List => list(),
        CatalogAction::Show { name } => show(name),
        CatalogAction::Add {
            path,
            name,
            provider,
            source,
        } => add(
            path,
            name.as_deref(),
            provider.as_deref(),
            source.as_deref(),
        ),
        CatalogAction::Rm { name } => rm(name),
    }
}

fn list() {
    let registry = open_registry();
    if registry.is_empty() {
        println!("catalog 为空");
        return;
    }
    println!("Volume:");
    for v in registry.entries().values() {
        let status_icon = match v.status {
            VolumeStatus::Received => "📥",
            VolumeStatus::Processing => "⏳",
            VolumeStatus::Processed => "✅",
            VolumeStatus::Delivered => "📤",
            VolumeStatus::Unknown => "📄",
        };
        println!("  {status_icon} {}  ({})", v.name, v.path);
    }
}

fn show(name: &str) {
    let registry = open_registry();
    match registry.get(name) {
        Some(v) => {
            println!("名称:       {}", v.name);
            println!("路径:       {}", v.path);
            println!("大小:       {}", format_size(v.size));
            println!("接收时间:   {}", v.received_at);
            println!("状态:       {}", v.status);
            if let Some(p) = &v.provider {
                println!("Provider:   {p}");
            }
            if let Some(s) = &v.source {
                println!("来源:       {s}");
            }
        }
        None => {
            eprintln!("未找到 volume: {name}");
            std::process::exit(1);
        }
    }
}

fn add(path_str: &str, name: Option<&str>, provider: Option<&str>, source: Option<&str>) {
    let volume = register_volume(RegisterVolume {
        path: path_str,
        name,
        provider,
        source,
        status: VolumeStatus::Received,
    })
    .unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });

    println!("✓ 已注册 volume: {}", volume.name);
}

fn rm(name: &str) {
    let mut registry = open_registry();
    match registry.remove(name) {
        Ok(Some(_)) => println!("✓ 已删除 volume: {name}"),
        Ok(None) => {
            eprintln!("未找到 volume: {name}");
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("删除 volume 失败: {err}");
            std::process::exit(1);
        }
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        dir
    }

    #[test]
    fn register_volume_writes_registry_entry() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-register");
        let catalog_dir = root.join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();
        let file = root.join("final.csv");
        std::fs::write(&file, "a,b\n1,2\n").unwrap();

        unsafe {
            std::env::set_var("CATALOG_DIR", &catalog_dir);
        }
        let volume = register_volume(RegisterVolume {
            path: file.to_str().unwrap(),
            name: Some("ABC-001-final"),
            provider: Some("process"),
            source: Some("process:ABC-001-123"),
            status: VolumeStatus::Delivered,
        })
        .unwrap();
        unsafe {
            std::env::remove_var("CATALOG_DIR");
        }

        assert_eq!(volume.name, "ABC-001-final");
        assert_eq!(volume.provider.as_deref(), Some("process"));
        assert_eq!(volume.source.as_deref(), Some("process:ABC-001-123"));
        assert_eq!(volume.status, VolumeStatus::Delivered);

        let registry = std::fs::read_to_string(catalog_dir.join("registry.json")).unwrap();
        assert!(registry.contains("ABC-001-final"));

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn volume_status_serializes_to_legacy_strings() {
        assert_eq!(
            serde_json::to_string(&VolumeStatus::Delivered).unwrap(),
            "\"delivered\""
        );
        assert_eq!(
            serde_json::to_string(&VolumeStatus::Received).unwrap(),
            "\"received\""
        );
    }

    #[test]
    fn volume_status_deserializes_unknown_strings_as_unknown() {
        // 旧数据/未来新增状态不应导致整表加载失败
        assert_eq!(
            serde_json::from_str::<VolumeStatus>("\"future-status\"").unwrap(),
            VolumeStatus::Unknown
        );
        assert_eq!(
            serde_json::from_str::<VolumeStatus>("\"delivered\"").unwrap(),
            VolumeStatus::Delivered
        );
    }

    #[test]
    fn registry_path_uses_data_root_when_catalog_dir_missing() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-data-root");

        unsafe {
            std::env::remove_var("CATALOG_DIR");
            std::env::set_var("DATA_ROOT", &root);
        }
        let path = registry_path();
        unsafe {
            std::env::remove_var("DATA_ROOT");
        }

        assert_eq!(path, root.join("catalog").join("registry.json"));

        std::fs::remove_dir_all(&root).ok();
    }
}
