//! v0.2.1 基线 e2e：对现有 `process` 命令建立全链路回归保护。
//!
//! fixture 位于 `tests/fixtures/github-activity/`（对标
//! `examples/AI范例-DRD-数据需求文档.md` 的 GitHub 用户活动面板）。
//!
//! 覆盖点（相对既有单元测试新增的内容级断言）：
//! - 真实业务数据从 receive 流入 pipeline，产物内容与 `expected-final.csv` 逐字节一致
//! - process 全链路（receive → pipeline → send）落盘 jobs.json / registry.json / 日志
//! - 敏感信息（URL query token）不出现在输出和落盘记录中

use std::path::{Path, PathBuf};
use std::process::Command;

fn cli() -> Command {
    Command::new("./target/debug/qtcloud-data")
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixture(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

fn script_path(dir: &Path, stem: &str) -> PathBuf {
    let ext = if cfg!(windows) { "cmd" } else { "sh" };
    dir.join(format!("{stem}.{ext}"))
}

fn write_script(path: &Path, content: &str) {
    std::fs::write(path, content).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}

/// fixture 驱动的假 qtdata：receive 从 `FIXTURE_RAW` 复制真实 raw 数据，
/// send 写入交付链接。业务数据流由 fixture 决定，保证内容级断言有效。
fn fixture_driven_qtdata_script() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\nif \"%1\"==\"transfer\" if \"%2\"==\"receive\" (\r\n  copy /Y \"%FIXTURE_RAW%\" \"%5\" >NUL\r\n  exit /b 0\r\n)\r\nif \"%1\"==\"transfer\" if \"%2\"==\"send\" (\r\n  echo https://delivery.example/github-activity>\"%5\"\r\n  exit /b 0\r\n)\r\nexit /b 1\r\n"
    } else {
        "#!/bin/sh\nif [ \"$1\" = \"transfer\" ] && [ \"$2\" = \"receive\" ]; then\n  cp \"$FIXTURE_RAW\" \"$5\"\n  exit 0\nfi\nif [ \"$1\" = \"transfer\" ] && [ \"$2\" = \"send\" ]; then\n  printf 'https://delivery.example/github-activity\\n' > \"$5\"\n  exit 0\nfi\nexit 1\n"
    }
}

#[test]
fn e2e_process_full_chain_delivers_normalized_activity() {
    let root = std::env::temp_dir().join(format!("qtcloud-e2e-baseline-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    // 1) 测试脚手架：fixture 驱动的假 qtdata
    let fake_qtdata = script_path(&root, "fake-qtdata");
    write_script(&fake_qtdata, fixture_driven_qtdata_script());

    // 2) 真实流水线脚本：unix 直接使用 fixture，windows 退化为 copy（仅回归链路机制）
    let pipeline = script_path(&root, "normalize");
    let pipeline_content = if cfg!(windows) {
        "@echo off\r\ncopy /Y \"%1\" \"%2\" >NUL\r\nexit /b %ERRORLEVEL%\r\n".to_string()
    } else {
        std::fs::read_to_string(fixture("github-activity/normalize.sh")).unwrap()
    };
    write_script(&pipeline, &pipeline_content);

    let catalog_dir = root.join("catalog");
    let work_dir = root.join("work");
    let raw_fixture = fixture("github-activity/raw.csv");

    // 3) 运行 process 全链路
    let output = cli()
        .env("QTDATA_CLI", &fake_qtdata)
        .env("FIXTURE_RAW", &raw_fixture)
        .env("CATALOG_DIR", &catalog_dir)
        .env("WORKDIR", &work_dir)
        .arg("process")
        .arg("github-activity")
        .arg("https://example.com/raw.csv?access_token=secret")
        .arg("--pipeline")
        .arg(&pipeline)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "process failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // 4) 产物内容 == 期望（内容级断言，逐字节）
    let final_path = work_dir.join("github-activity").join("final.csv");
    let expected = std::fs::read_to_string(fixture("github-activity/expected-final.csv")).unwrap();
    let actual = std::fs::read_to_string(&final_path).unwrap();
    assert_eq!(
        actual, expected,
        "final.csv 与 expected-final.csv 不一致:\n--- actual ---\n{actual}\n--- expected ---\n{expected}"
    );

    // 5) jobs.json：单条 delivered 记录，URL token 已脱敏
    let jobs_content = std::fs::read_to_string(catalog_dir.join("jobs.json")).unwrap();
    assert!(
        !jobs_content.contains("access_token=secret"),
        "jobs.json 不应包含 URL token"
    );
    let jobs: serde_json::Value = serde_json::from_str(&jobs_content).unwrap();
    let jobs = jobs.as_object().unwrap();
    assert_eq!(jobs.len(), 1);
    let record = jobs.values().next().unwrap();
    assert_eq!(record["customer_id"], "github-activity");
    assert_eq!(record["source_url"], "https://example.com/raw.csv");
    assert_eq!(record["status"], "delivered");
    assert!(record["raw_path"].as_str().unwrap().ends_with("raw.csv"));
    assert!(
        record["output_path"]
            .as_str()
            .unwrap()
            .ends_with("final.csv")
    );
    assert!(std::path::Path::new(record["log_path"].as_str().unwrap()).is_file());

    // 6) registry.json：最终产物已登记为 delivered
    let registry_content = std::fs::read_to_string(catalog_dir.join("registry.json")).unwrap();
    let registry: serde_json::Value = serde_json::from_str(&registry_content).unwrap();
    let volumes = registry.as_object().unwrap();
    assert_eq!(volumes.len(), 1);
    let volume = volumes.values().next().unwrap();
    assert_eq!(volume["provider"], "process");
    assert_eq!(volume["status"], "delivered");
    assert!(volume["path"].as_str().unwrap().ends_with("final.csv"));

    // 7) 日志含流水线完成记录
    let log = std::fs::read_to_string(record["log_path"].as_str().unwrap()).unwrap();
    assert!(
        log.contains("pipeline completed"),
        "日志缺少 pipeline completed: {log}"
    );

    // 8) 敏感信息不出现在 stdout/stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("access_token=secret"));
    assert!(!stderr.contains("access_token=secret"));

    std::fs::remove_dir_all(&root).ok();
}

// ── process 命令级测试（自 cli_test.rs 归位）──

fn fake_qtdata_script() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\nif \"%1\"==\"transfer\" if \"%2\"==\"receive\" (\r\n  echo raw,data>\"%5\"\r\n  exit /b 0\r\n)\r\nif \"%1\"==\"transfer\" if \"%2\"==\"send\" (\r\n  echo https://delivery.example/link>\"%5\"\r\n  exit /b 0\r\n)\r\nexit /b 1\r\n"
    } else {
        "#!/bin/sh\nif [ \"$1\" = \"transfer\" ] && [ \"$2\" = \"receive\" ]; then\n  printf 'raw,data\\n' > \"$5\"\n  exit 0\nfi\nif [ \"$1\" = \"transfer\" ] && [ \"$2\" = \"send\" ]; then\n  printf 'https://delivery.example/link\\n' > \"$5\"\n  exit 0\nfi\nexit 1\n"
    }
}

fn copy_pipeline_script() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\ncopy /Y \"%1\" \"%2\" >NUL\r\nexit /b %ERRORLEVEL%\r\n"
    } else {
        "#!/bin/sh\ncp \"$1\" \"$2\"\n"
    }
}

fn failing_pipeline_script() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\nexit /b 7\r\n"
    } else {
        "#!/bin/sh\nexit 7\n"
    }
}

#[test]
fn test_process_writes_job_record_after_success() {
    let root = std::env::temp_dir().join(format!("qtcloud-process-record-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let fake_qtdata = script_path(&root, "fake-qtdata");
    let pipeline = script_path(&root, "copy-pipeline");
    write_script(&fake_qtdata, fake_qtdata_script());
    write_script(&pipeline, copy_pipeline_script());

    let catalog_dir = root.join("catalog");
    let work_dir = root.join("work");

    let output = cli()
        .env("QTDATA_CLI", &fake_qtdata)
        .env("CATALOG_DIR", &catalog_dir)
        .env("WORKDIR", &work_dir)
        .arg("process")
        .arg("ABC-001")
        .arg("https://example.com/raw.csv?access_token=secret")
        .arg("--pipeline")
        .arg(&pipeline)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "process failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("access_token=secret"));
    assert!(!stderr.contains("access_token=secret"));

    let jobs_path = catalog_dir.join("jobs.json");
    let content = std::fs::read_to_string(&jobs_path).unwrap();
    let jobs: serde_json::Value = serde_json::from_str(&content).unwrap();
    let jobs = jobs.as_object().unwrap();
    assert_eq!(jobs.len(), 1);

    let record = jobs.values().next().unwrap();
    let job_id = record["id"].as_str().unwrap();
    assert_eq!(record["customer_id"], "ABC-001");
    assert_eq!(record["source_url"], "https://example.com/raw.csv");
    assert_eq!(record["status"], "delivered");
    assert!(
        record["pipeline"]
            .as_str()
            .unwrap()
            .contains("copy-pipeline")
    );
    assert!(record["raw_path"].as_str().unwrap().ends_with("raw.csv"));
    assert!(
        record["output_path"]
            .as_str()
            .unwrap()
            .ends_with("final.csv")
    );
    assert!(
        record["link_path"]
            .as_str()
            .unwrap()
            .ends_with("share-link.txt")
    );
    assert!(std::path::Path::new(record["log_path"].as_str().unwrap()).is_file());
    assert!(!content.contains("access_token=secret"));

    let registry_content = std::fs::read_to_string(catalog_dir.join("registry.json")).unwrap();
    let registry: serde_json::Value = serde_json::from_str(&registry_content).unwrap();
    let volumes = registry.as_object().unwrap();
    assert_eq!(volumes.len(), 1);

    let volume = volumes.values().next().unwrap();
    assert_eq!(volume["provider"], "process");
    assert_eq!(volume["source"], format!("process:{job_id}"));
    assert_eq!(volume["status"], "delivered");
    assert!(volume["path"].as_str().unwrap().ends_with("final.csv"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_process_writes_failed_job_record_when_pipeline_fails() {
    let root = std::env::temp_dir().join(format!(
        "qtcloud-process-failed-record-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let fake_qtdata = script_path(&root, "fake-qtdata");
    let pipeline = script_path(&root, "failing-pipeline");
    write_script(&fake_qtdata, fake_qtdata_script());
    write_script(&pipeline, failing_pipeline_script());

    let catalog_dir = root.join("catalog");
    let work_dir = root.join("work");

    let output = cli()
        .env("QTDATA_CLI", &fake_qtdata)
        .env("CATALOG_DIR", &catalog_dir)
        .env("WORKDIR", &work_dir)
        .arg("process")
        .arg("ABC-002")
        .arg("https://example.com/raw.csv?access_token=secret")
        .arg("--pipeline")
        .arg(&pipeline)
        .output()
        .unwrap();

    assert!(!output.status.success());

    let content = std::fs::read_to_string(catalog_dir.join("jobs.json")).unwrap();
    let jobs: serde_json::Value = serde_json::from_str(&content).unwrap();
    let record = jobs.as_object().unwrap().values().next().unwrap();
    assert_eq!(record["customer_id"], "ABC-002");
    assert_eq!(record["status"], "failed");
    assert_eq!(record["source_url"], "https://example.com/raw.csv");
    assert!(!content.contains("access_token=secret"));

    let log = std::fs::read_to_string(record["log_path"].as_str().unwrap()).unwrap();
    assert!(log.contains("pipeline failed"));

    std::fs::remove_dir_all(&root).ok();
}
