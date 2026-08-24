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
    assert_eq!(report["ok"], true);
    assert_eq!(report["command"], "pipeline list");
    assert_eq!(report["data"]["items"][0], "normalize");

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
    assert!(report["data"]["output"].is_string());
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
    assert_eq!(report["data"]["items"][0]["name"], "report");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_global_json_formats_pipeline_show_success() {
    let root = std::env::temp_dir().join(format!(
        "qtcloud-global-json-pipeline-show-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("normalize.yaml"), "name: normalize\n").unwrap();

    let output = cli()
        .env("PIPELINE_DIR", &root)
        .arg("--json")
        .arg("pipeline")
        .arg("show")
        .arg("normalize")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["command"], "pipeline show");
    assert_eq!(report["data"]["name"], "normalize");
    assert_eq!(report["data"]["pipeline"]["name"], "normalize");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_global_json_formats_blueprint_show_success() {
    let root = std::env::temp_dir().join(format!(
        "qtcloud-global-json-blueprint-show-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("customer.yaml"), "name: customer\n").unwrap();

    let output = cli()
        .env("BLUEPRINT_DIR", &root)
        .arg("--json")
        .arg("blueprint")
        .arg("show")
        .arg("customer")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["command"], "blueprint show");
    assert_eq!(report["data"]["name"], "customer");
    assert_eq!(report["data"]["blueprint"]["name"], "customer");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_global_json_formats_contract_list_and_show_success() {
    let root = std::env::temp_dir().join(format!(
        "qtcloud-global-json-contract-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("customer.yaml"), "name: customer\n").unwrap();

    let list = cli()
        .env("CONTRACT_DIR", &root)
        .arg("--json")
        .arg("contract")
        .arg("list")
        .output()
        .unwrap();
    assert!(list.status.success());
    assert!(list.stderr.is_empty());
    let list_report: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(list_report["ok"], true);
    assert_eq!(list_report["command"], "contract list");
    assert_eq!(list_report["data"]["items"][0], "customer");

    let show = cli()
        .env("CONTRACT_DIR", &root)
        .arg("--json")
        .arg("contract")
        .arg("show")
        .arg("customer")
        .output()
        .unwrap();
    assert!(show.status.success());
    assert!(show.stderr.is_empty());
    let show_report: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(show_report["ok"], true);
    assert_eq!(show_report["command"], "contract show");
    assert_eq!(show_report["data"]["name"], "customer");
    assert_eq!(show_report["data"]["content"], "name: customer\n");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_global_json_formats_catalog_show_and_add_success() {
    let root = std::env::temp_dir().join(format!(
        "qtcloud-global-json-catalog-write-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let data = root.join("report.csv");
    std::fs::write(&data, "a,b\n1,2\n").unwrap();

    let add = cli()
        .env("CATALOG_DIR", &root)
        .arg("--json")
        .arg("catalog")
        .arg("add")
        .arg(&data)
        .arg("--name")
        .arg("report")
        .output()
        .unwrap();
    assert!(add.status.success());
    assert!(add.stderr.is_empty());
    let add_report: serde_json::Value = serde_json::from_slice(&add.stdout).unwrap();
    assert_eq!(add_report["ok"], true);
    assert_eq!(add_report["command"], "catalog add");
    assert_eq!(add_report["data"]["volume"]["name"], "report");

    let show = cli()
        .env("CATALOG_DIR", &root)
        .arg("--json")
        .arg("catalog")
        .arg("show")
        .arg("report")
        .output()
        .unwrap();
    assert!(show.status.success());
    assert!(show.stderr.is_empty());
    let show_report: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(show_report["ok"], true);
    assert_eq!(show_report["command"], "catalog show");
    assert_eq!(show_report["data"]["volume"]["name"], "report");

    std::fs::remove_dir_all(&root).ok();
}
