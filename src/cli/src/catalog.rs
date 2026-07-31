use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

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
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "received".to_string()
}

fn registry_path() -> PathBuf {
    let dir =
        std::env::var("CATALOG_DIR").unwrap_or_else(|_| ".quanttide/data/catalog".to_string());
    let p = PathBuf::from(&dir);
    std::fs::create_dir_all(&p).ok();
    p.join("registry.json")
}

fn load_registry() -> BTreeMap<String, Volume> {
    let path = registry_path();
    if !path.exists() {
        return BTreeMap::new();
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    if content.trim().is_empty() {
        return BTreeMap::new();
    }
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_registry(registry: &BTreeMap<String, Volume>) {
    let path = registry_path();
    let json = serde_json::to_string_pretty(registry).expect("序列化失败");
    std::fs::write(&path, json).expect("写入 registry 失败");
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
    let registry = load_registry();
    if registry.is_empty() {
        println!("catalog 为空");
        return;
    }
    println!("Volume:");
    for v in registry.values() {
        let status_icon = match v.status.as_str() {
            "received" => "📥",
            "processing" => "⏳",
            "processed" => "✅",
            "delivered" => "📤",
            _ => "📄",
        };
        println!("  {status_icon} {}  ({})", v.name, v.path);
    }
}

fn show(name: &str) {
    let registry = load_registry();
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
    let path = PathBuf::from(path_str);
    if !path.exists() {
        eprintln!("文件不存在: {path_str}");
        std::process::exit(1);
    }

    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("读取文件元数据失败: {e}");
            std::process::exit(1);
        }
    };

    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let volume_name = name.map(|s| s.to_string()).unwrap_or_else(|| file_name);

    let now = chrono_now();

    let volume = Volume {
        name: volume_name.clone(),
        path: path
            .canonicalize()
            .unwrap_or(path)
            .to_string_lossy()
            .to_string(),
        size: meta.len(),
        received_at: now,
        provider: provider.map(|s| s.to_string()),
        source: source.map(|s| s.to_string()),
        status: "received".to_string(),
    };

    let mut registry = load_registry();
    registry.insert(volume_name.clone(), volume);
    save_registry(&registry);

    println!("✓ 已注册 volume: {volume_name}");
}

fn rm(name: &str) {
    let mut registry = load_registry();
    if registry.remove(name).is_some() {
        save_registry(&registry);
        println!("✓ 已删除 volume: {name}");
    } else {
        eprintln!("未找到 volume: {name}");
        std::process::exit(1);
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

fn chrono_now() -> String {
    // 不引入 chrono 依赖，用 UTC 时间戳格式化
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // 简单格式化为 YYYY-MM-DD HH:MM:SS
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // 从 1970-01-01 计算年月日
    let (y, m, d) = days_to_date(days as i64);
    format!("{y:04}-{m:02}-{d:02} {hours:02}:{minutes:02}:{seconds:02}")
}

fn days_to_date(mut days: i64) -> (i64, u32, u32) {
    // 从 Unix 纪元 (1970-01-01) 计算日期
    days += 719468; // 从公元 0 年开始
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
