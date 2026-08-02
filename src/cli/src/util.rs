//! 通用工具：数据目录解析与 UTC 时间格式化（原 store.rs 拆出）。

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

    #[test]
    fn to_camel_basic() {
        assert_eq!(to_camel("csv-standard"), "csvStandard");
        assert_eq!(to_camel("simple"), "simple");
        assert_eq!(to_camel("abc-def-ghi"), "abcDefGhi");
        assert_eq!(to_camel(""), "");
    }

    #[test]
    fn test_resolve_cue_path_nonexistent() {
        let result = resolve_cue_path("nonexistent-blueprint-12345", "/tmp");
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_cue_path_by_name() {
        let tmp = std::env::temp_dir().join("bp-test-resolve");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("my-bp.cue"),
            "package blueprints\n{name: \"test\"}",
        )
        .unwrap();

        let result = resolve_cue_path("my-bp", tmp.to_str().unwrap());
        assert!(result.is_some());

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn test_blueprint_dir_default() {
        let dir = blueprint_dir();
        assert_eq!(dir, ".quanttide/data/blueprint");
    }

    #[test]
    fn test_drd_dir_default() {
        assert_eq!(drd_dir(), ".quanttide/data/drd");
    }

    #[test]
    fn test_spec_dir_default() {
        assert_eq!(spec_dir(), ".quanttide/data/spec");
    }

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
        let root = temp_dir("qtcloud-util-data-root");
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

/// Get the blueprint directory from env or default.
pub fn blueprint_dir() -> String {
    std::env::var("BLUEPRINT_DIR").unwrap_or_else(|_| ".quanttide/data/blueprint".to_string())
}

/// Get the DRD directory (Data Requirements Document).
pub fn drd_dir() -> String {
    std::env::var("DRD_DIR").unwrap_or_else(|_| ".quanttide/data/drd".to_string())
}

/// Get the Specification directory.
pub fn spec_dir() -> String {
    std::env::var("SPEC_DIR").unwrap_or_else(|_| ".quanttide/data/spec".to_string())
}

/// Resolve a user input to a .cue or .yaml file path.
pub fn resolve_cue_path(input: &str, dir: &str) -> Option<PathBuf> {
    let p = Path::new(input);
    if p.exists() {
        Some(p.to_path_buf())
    } else {
        for ext in &["yaml", "cue"] {
            let with_ext = Path::new(dir).join(format!("{input}.{ext}"));
            if with_ext.exists() {
                return Some(with_ext);
            }
        }
        None
    }
}

// ── 命名与 JSON 工具 ──

/// 从 cue 导出的 JSON 顶层收集各定义的 `name` 字段（结构化解析，替代文本 grep）。
pub fn collect_defined_names(value: &serde_json::Value) -> Vec<String> {
    let mut names = Vec::new();
    if let serde_json::Value::Object(map) = value {
        for nested in map.values() {
            if let Some(name) = nested.get("name").and_then(|n| n.as_str()) {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

/// 转换 kebab-case 为 camelCase（用于 cue expression 键）。
///
/// # 示例
///
/// ```
/// assert_eq!(qtcloud_data_cli::util::to_camel("csv-standard"), "csvStandard");
/// assert_eq!(qtcloud_data_cli::util::to_camel("abc-def"), "abcDef");
/// ```
pub fn to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '-' {
            upper = true;
        } else if upper {
            result.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            result.push(c);
        }
    }
    result
}
