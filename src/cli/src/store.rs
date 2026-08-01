//! 统一数据存取：catalog/process/transfer 共用的路径解析、时间工具与 JSON 注册表读写。
//!
//! 收敛三份拷贝：
//! - catalog 目录解析（catalog.rs / process.rs / transfer.rs 各一份）
//! - UTC 时间格式化（三份 `chrono_now` + `days_to_date`）
//! - JSON 文件注册表读写（`registry.json` / `jobs.json` / `delivery-links.json`）
//! - 写盘原子化（临时文件 + rename，避免半写文件被并发读者读到）

use serde::{Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 解析 catalog 根目录：`CATALOG_DIR` > `DATA_ROOT/catalog` > `.quanttide/data/catalog`。
pub fn catalog_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CATALOG_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(root) = std::env::var("DATA_ROOT") {
        return PathBuf::from(root).join("catalog");
    }
    PathBuf::from(".quanttide/data/catalog")
}

/// JSON 文件注册表（key -> record），读时整表加载，写时原子落盘。
pub struct Registry<T> {
    path: PathBuf,
    entries: BTreeMap<String, T>,
}

impl<T> Default for Registry<T> {
    /// 空注册表（路径为空，仅用于只读场景的兜底）。
    fn default() -> Self {
        Self {
            path: PathBuf::default(),
            entries: BTreeMap::new(),
        }
    }
}

impl<T: Serialize + DeserializeOwned> Registry<T> {
    /// 打开注册表；文件不存在或内容为空时返回空表（不创建目录）。
    pub fn open(path: &Path) -> io::Result<Self> {
        let entries = if !path.exists() {
            BTreeMap::new()
        } else {
            let content = fs::read_to_string(path)?;
            if content.trim().is_empty() {
                BTreeMap::new()
            } else {
                serde_json::from_str(&content).map_err(io::Error::other)?
            }
        };
        Ok(Self {
            path: path.to_path_buf(),
            entries,
        })
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.entries.get(key)
    }

    pub fn entries(&self) -> &BTreeMap<String, T> {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 插入记录并原子落盘。
    pub fn insert(&mut self, key: String, value: T) -> io::Result<()> {
        self.entries.insert(key, value);
        self.save()
    }

    /// 删除记录并原子落盘；记录不存在时不写盘。
    pub fn remove(&mut self, key: &str) -> io::Result<Option<T>> {
        let removed = self.entries.remove(key);
        if removed.is_some() {
            self.save()?;
        }
        Ok(removed)
    }

    /// 原子写盘：先写同目录临时文件，再 rename 覆盖。
    pub fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.entries)?;
        atomic_write(&self.path, json.as_bytes())
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = temp_sibling(path);
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn temp_sibling(path: &Path) -> PathBuf {
    let file_name = path.file_name().unwrap_or_default();
    let tmp_name = format!("{}.tmp", file_name.to_string_lossy());
    path.with_file_name(tmp_name)
}

/// UTC 时间 `YYYY-MM-DD HH:MM:SS`，替换三份 chrono_now 拷贝。
pub fn now_utc() -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ENV_LOCK;
    use crate::test_support::temp_dir;

    #[test]
    fn catalog_dir_resolution_prefers_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("CATALOG_DIR", "/tmp/custom-catalog");
            std::env::set_var("DATA_ROOT", "/tmp/custom-root");
        }
        assert_eq!(catalog_dir(), PathBuf::from("/tmp/custom-catalog"));
        unsafe {
            std::env::remove_var("CATALOG_DIR");
            std::env::remove_var("DATA_ROOT");
        }
    }

    #[test]
    fn catalog_dir_falls_back_to_data_root_then_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        let root = temp_dir("qtcloud-store-data-root");
        unsafe {
            std::env::remove_var("CATALOG_DIR");
            std::env::set_var("DATA_ROOT", &root);
        }
        assert_eq!(catalog_dir(), root.join("catalog"));
        unsafe {
            std::env::remove_var("DATA_ROOT");
        }
        assert_eq!(catalog_dir(), PathBuf::from(".quanttide/data/catalog"));
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn registry_roundtrip_insert_save_load() {
        let root = temp_dir("qtcloud-store-registry");
        let path = root.join("registry.json");
        let mut registry = Registry::open(&path).unwrap();
        assert!(registry.is_empty());
        registry.insert("a".to_string(), 1u32).unwrap();
        registry.insert("b".to_string(), 2u32).unwrap();

        let loaded = Registry::<u32>::open(&path).unwrap();
        assert_eq!(loaded.get("a"), Some(&1));
        assert_eq!(loaded.get("b"), Some(&2));
        assert_eq!(loaded.len(), 2);
        // 原子写盘后不残留临时文件
        assert!(!path.with_file_name("registry.json.tmp").exists());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn registry_remove_persists_and_missing_key_skips_write() {
        let root = temp_dir("qtcloud-store-remove");
        let path = root.join("jobs.json");
        let mut registry = Registry::open(&path).unwrap();
        registry
            .insert("job-1".to_string(), "x".to_string())
            .unwrap();
        assert_eq!(registry.remove("nope").unwrap(), None);
        // 未命中 key 不写盘，记录仍在
        let loaded = Registry::<String>::open(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(registry.remove("job-1").unwrap(), Some("x".to_string()));
        let loaded = Registry::<String>::open(&path).unwrap();
        assert!(loaded.is_empty());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn now_utc_formats_yyyy_mm_dd_hh_mm_ss() {
        let s = now_utc();
        assert_eq!(s.len(), 19, "unexpected format: {s}");
        let bytes = s.as_bytes();
        assert_eq!(&bytes[4..5], b"-");
        assert_eq!(&bytes[7..8], b"-");
        assert_eq!(&bytes[10..11], b" ");
        assert_eq!(&bytes[13..14], b":");
        assert_eq!(&bytes[16..17], b":");
    }
}
