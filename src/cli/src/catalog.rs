use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

use crate::error::CliError;
use crate::registry;
use crate::util;

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
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeStatus {
    #[default]
    Received,
    Processing,
    Processed,
    Delivered,
    /// 未知状态（兼容旧数据或未来新增状态）。
    #[serde(other)]
    Unknown,
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
    util::catalog_dir().join("registry.json")
}

fn open_registry() -> registry::Registry<Volume> {
    registry::Registry::open(&registry_path()).unwrap_or_default()
}

// ── 数据模型（VolumeStatus / Volume / RegisterVolume） ──
pub fn register_volume(input: RegisterVolume<'_>) -> Result<Volume, CliError> {
    let path = PathBuf::from(input.path);
    if !path.exists() {
        return Err(CliError::new(format!("文件不存在: {}", input.path)));
    }

    let meta = std::fs::metadata(&path).map_err(CliError::from)?;

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
        received_at: util::now_utc(),
        provider: input.provider.map(|provider| provider.to_string()),
        source: input.source.map(|source| source.to_string()),
        status: input.status,
    };

    let mut registry = open_registry();
    registry
        .insert(volume_name, volume.clone())
        .map_err(|err| CliError::new(format!("写入 registry 失败: {err}")))?;

    Ok(volume)
}

// ── 命令（run / list / show / add / rm） ──
pub fn run(args: &CatalogArgs) -> Result<(), CliError> {
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

fn list() -> Result<(), CliError> {
    let registry = open_registry();
    if registry.is_empty() {
        println!("catalog 为空");
        return Ok(());
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
    Ok(())
}

fn show(name: &str) -> Result<(), CliError> {
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
            Ok(())
        }
        None => Err(CliError::new(format!("未找到 volume: {name}"))),
    }
}

fn add(
    path_str: &str,
    name: Option<&str>,
    provider: Option<&str>,
    source: Option<&str>,
) -> Result<(), CliError> {
    let volume = register_volume(RegisterVolume {
        path: path_str,
        name,
        provider,
        source,
        status: VolumeStatus::Received,
    })?;

    println!("✓ 已注册 volume: {}", volume.name);
    Ok(())
}

fn rm(name: &str) -> Result<(), CliError> {
    let mut registry = open_registry();
    match registry.remove(name) {
        Ok(Some(_)) => {
            println!("✓ 已删除 volume: {name}");
            Ok(())
        }
        Ok(None) => Err(CliError::new(format!("未找到 volume: {name}"))),
        Err(err) => Err(CliError::new(format!("删除 volume 失败: {err}"))),
    }
}

// ── 工具 ──
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
    use crate::test_support::temp_dir;

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

    #[test]
    fn format_size_rounds_to_appropriate_unit() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 5 / 2), "2.5 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 3), "3.0 GB");
    }

    #[test]
    fn show_reports_missing_volume_without_exiting() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-show-missing");
        let catalog_dir = root.join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        unsafe {
            std::env::set_var("CATALOG_DIR", &catalog_dir);
        }
        let err = show("ghost").unwrap_err();
        unsafe {
            std::env::remove_var("CATALOG_DIR");
        }

        assert_eq!(err.to_string(), "未找到 volume: ghost");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn add_reports_missing_file_without_exiting() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-add-missing");
        let catalog_dir = root.join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        unsafe {
            std::env::set_var("CATALOG_DIR", &catalog_dir);
        }
        let err = add("/nonexistent/file.csv", None, None, None).unwrap_err();
        unsafe {
            std::env::remove_var("CATALOG_DIR");
        }

        assert!(err.to_string().contains("文件不存在"), "{}", err);

        std::fs::remove_dir_all(&root).ok();
    }

    /// 在临时 CATALOG_DIR 中注册一个 volume 作为测试数据。
    fn seed_volume(catalog_dir: &std::path::Path) -> String {
        let file = catalog_dir.parent().unwrap().join("final.csv");
        std::fs::write(&file, "a,b\n1,2\n").unwrap();
        let volume = register_volume(RegisterVolume {
            path: file.to_str().unwrap(),
            name: Some("ABC-001"),
            provider: Some("process"),
            source: None,
            status: VolumeStatus::Delivered,
        })
        .unwrap();
        volume.name
    }

    #[test]
    fn list_prints_non_empty_catalog_without_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-list");
        let catalog_dir = root.join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        unsafe {
            std::env::set_var("CATALOG_DIR", &catalog_dir);
        }
        seed_volume(&catalog_dir);
        let result = list();
        unsafe {
            std::env::remove_var("CATALOG_DIR");
        }
        assert!(result.is_ok(), "{result:?}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn list_prints_empty_message_for_empty_catalog() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-list-empty");
        let catalog_dir = root.join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        unsafe {
            std::env::set_var("CATALOG_DIR", &catalog_dir);
        }
        let result = list();
        unsafe {
            std::env::remove_var("CATALOG_DIR");
        }
        assert!(result.is_ok(), "{result:?}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn show_prints_volume_details_for_existing_volume() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-show");
        let catalog_dir = root.join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        unsafe {
            std::env::set_var("CATALOG_DIR", &catalog_dir);
        }
        let name = seed_volume(&catalog_dir);
        let result = show(&name);
        unsafe {
            std::env::remove_var("CATALOG_DIR");
        }
        assert!(result.is_ok(), "{result:?}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn add_registers_volume_and_returns_ok() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-add-ok");
        let catalog_dir = root.join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();
        let file = root.join("data.csv");
        std::fs::write(&file, "a,b\n").unwrap();

        unsafe {
            std::env::set_var("CATALOG_DIR", &catalog_dir);
        }
        let result = add(
            file.to_str().unwrap(),
            Some("DATA-1"),
            Some("process"),
            None,
        );
        unsafe {
            std::env::remove_var("CATALOG_DIR");
        }
        assert!(result.is_ok(), "{result:?}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn rm_removes_existing_volume_and_returns_ok() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-rm-ok");
        let catalog_dir = root.join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        unsafe {
            std::env::set_var("CATALOG_DIR", &catalog_dir);
        }
        let name = seed_volume(&catalog_dir);
        let result = rm(&name);
        unsafe {
            std::env::remove_var("CATALOG_DIR");
        }
        assert!(result.is_ok(), "{result:?}");

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn volume_status_display_covers_all_variants() {
        assert_eq!(format!("{}", VolumeStatus::Received), "received");
        assert_eq!(format!("{}", VolumeStatus::Processing), "processing");
        assert_eq!(format!("{}", VolumeStatus::Processed), "processed");
        assert_eq!(format!("{}", VolumeStatus::Delivered), "delivered");
        assert_eq!(format!("{}", VolumeStatus::Unknown), "unknown");
    }

    #[test]
    fn rm_reports_missing_volume_without_exiting() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-catalog-rm-missing");
        let catalog_dir = root.join("catalog");
        std::fs::create_dir_all(&catalog_dir).unwrap();

        unsafe {
            std::env::set_var("CATALOG_DIR", &catalog_dir);
        }
        let err = rm("ghost").unwrap_err();
        unsafe {
            std::env::remove_var("CATALOG_DIR");
        }

        assert_eq!(err.to_string(), "未找到 volume: ghost");

        std::fs::remove_dir_all(&root).ok();
    }
}
