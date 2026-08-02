//! JSON 文件注册表（key -> record），读时整表加载，写时原子落盘。
//!
//! 供 `registry.json` / `jobs.json` / `delivery-links.json` 使用（原 store.rs 拆出）。

use serde::{Serialize, de::DeserializeOwned};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::temp_dir;

    #[test]
    fn registry_roundtrip_insert_save_load() {
        let root = temp_dir("qtcloud-registry-roundtrip");
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
        let root = temp_dir("qtcloud-registry-remove");
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
}
