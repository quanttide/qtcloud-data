//! doctor 命令集成测试（spawn CLI 二进制）。

mod common;

use common::cli;

#[test]
fn test_doctor_help() {
    let output = cli().arg("doctor").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("检查本机 DataOps 环境"));
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("--fix-dirs"));
}

#[test]
fn test_doctor_no_fail_runs_without_printing_secret_values() {
    let output = cli()
        .env("DROPBOX_ACCESS_TOKEN", "top-secret-token")
        .arg("doctor")
        .arg("--no-fail")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Summary:"));
    assert!(!stdout.contains("top-secret-token"));
}

#[test]
fn test_doctor_json_no_fail_outputs_machine_readable_report() {
    let output = cli()
        .env("DROPBOX_ACCESS_TOKEN", "top-secret-token")
        .arg("doctor")
        .arg("--json")
        .arg("--no-fail")
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["command"], "doctor");
    assert!(report["summary"]["warnings"].is_number());
    assert!(report["checks"].as_array().unwrap().len() > 5);
    assert!(!stdout.contains("top-secret-token"));
}

#[test]
fn test_doctor_fix_dirs_creates_configured_data_dirs() {
    let root = std::env::temp_dir().join(format!("qtcloud-doctor-fix-dirs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);

    let drd = root.join("drd");
    let spec = root.join("spec");
    let blueprint = root.join("blueprint");
    let contract = root.join("contract");
    let pipeline = root.join("pipeline");
    let catalog = root.join("catalog");

    let output = cli()
        .env("DATA_ROOT", &root)
        .env("DRD_DIR", &drd)
        .env("SPEC_DIR", &spec)
        .env("BLUEPRINT_DIR", &blueprint)
        .env("CONTRACT_DIR", &contract)
        .env("PIPELINE_DIR", &pipeline)
        .env("CATALOG_DIR", &catalog)
        .arg("doctor")
        .arg("--fix-dirs")
        .arg("--no-fail")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(drd.is_dir());
    assert!(spec.is_dir());
    assert!(blueprint.is_dir());
    assert!(contract.is_dir());
    assert!(pipeline.is_dir());
    assert!(catalog.is_dir());

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_doctor_fix_dirs_uses_data_root_for_default_children() {
    let root =
        std::env::temp_dir().join(format!("qtcloud-doctor-data-root-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);

    let output = cli()
        .env("DATA_ROOT", &root)
        .arg("doctor")
        .arg("--fix-dirs")
        .arg("--no-fail")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(root.join("drd").is_dir());
    assert!(root.join("spec").is_dir());
    assert!(root.join("blueprint").is_dir());
    assert!(root.join("contract").is_dir());
    assert!(root.join("pipeline").is_dir());
    assert!(root.join("catalog").is_dir());

    std::fs::remove_dir_all(&root).ok();
}
