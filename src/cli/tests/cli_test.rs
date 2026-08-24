//! CLI 入口集成测试（整体 help）。

mod common;

use common::{cli, sample_blueprint_yaml};

#[test]
fn test_cli_help_shows_all_commands() {
    let output = cli().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("clarify"));
    assert!(stdout.contains("design"));
    assert!(stdout.contains("review"));
    assert!(stdout.contains("version"));
    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("blueprint"));
    assert!(stdout.contains("spec"));
}

#[test]
fn test_global_json_formats_command_errors() {
    let missing = std::env::temp_dir().join(format!(
        "qtcloud-global-json-missing-{}.yaml",
        std::process::id()
    ));
    let output = cli()
        .arg("--json")
        .arg("spec")
        .arg("validate")
        .arg(&missing)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        output.stderr.is_empty(),
        "JSON errors should not use stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["error"]["code"], "cli_error");
    assert!(
        report["error"]["message"]
            .as_str()
            .unwrap()
            .contains("无法读取 YAML")
    );
}

#[test]
fn test_global_json_formats_pipeline_list_success() {
    let root = std::env::temp_dir().join(format!(
        "qtcloud-global-json-pipeline-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("normalize.yaml"), "name: normalize\n").unwrap();

    let output = cli()
        .env("PIPELINE_DIR", &root)
        .arg("--json")
        .arg("pipeline")
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["command"], "pipeline list");
    assert_eq!(report["items"][0], "normalize");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_global_json_formats_spec_wrap_success() {
    let root =
        std::env::temp_dir().join(format!("qtcloud-global-json-wrap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let input = root.join("sample.yaml");
    let output_path = root.join("sample-spec.yaml");
    std::fs::write(&input, sample_blueprint_yaml()).unwrap();

    let output = cli()
        .arg("--json")
        .arg("spec")
        .arg("wrap")
        .arg(&input)
        .arg("--output")
        .arg(&output_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["command"], "spec wrap");
    assert!(output_path.is_file());

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_global_json_formats_catalog_list_success() {
    let root = std::env::temp_dir().join(format!(
        "qtcloud-global-json-catalog-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let data = root.join("report.csv");
    std::fs::write(&data, "a,b\n1,2\n").unwrap();
    let registry = serde_json::json!({
        "report": {
            "name": "report",
            "path": data,
            "size": 8,
            "received_at": "2026-08-24 00:00:00",
            "status": "received",
            "artifact_type": "pre_review"
        }
    });
    std::fs::write(
        root.join("registry.json"),
        serde_json::to_vec_pretty(&registry).unwrap(),
    )
    .unwrap();

    let output = cli()
        .env("CATALOG_DIR", &root)
        .arg("--json")
        .arg("catalog")
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["command"], "catalog list");
    assert_eq!(report["items"][0]["name"], "report");

    std::fs::remove_dir_all(&root).ok();
}
