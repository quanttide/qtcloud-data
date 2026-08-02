//! spec 命令集成测试（spawn CLI 二进制）。

mod common;

use common::{cli, sample_blueprint_yaml};

#[test]
fn test_spec_help() {
    let output = cli().arg("spec").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wrap"));
    assert!(stdout.contains("validate"));
}

#[test]
fn test_spec_wrap_writes_enveloped_yaml() {
    let root = std::env::temp_dir().join(format!("qtcloud-spec-wrap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let input = root.join("sample-blueprint.yaml");
    let output_path = root.join("sample-spec.yaml");
    std::fs::write(&input, sample_blueprint_yaml()).unwrap();

    let output = cli()
        .arg("spec")
        .arg("wrap")
        .arg(&input)
        .arg("--output")
        .arg(&output_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "spec wrap failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read_to_string(&output_path).unwrap();
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
    assert_eq!(
        yaml["api_version"].as_str(),
        Some("qtcloud.quanttide.com/v1alpha1")
    );
    assert_eq!(yaml["kind"].as_str(), Some("Specification"));
    assert_eq!(yaml["metadata"]["name"].as_str(), Some("sample"));
    assert_eq!(
        yaml["spec"]["blueprint"]["pipeline"]["name"].as_str(),
        Some("sample-pipeline")
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn test_spec_validate_accepts_enveloped_yaml() {
    let root = std::env::temp_dir().join(format!("qtcloud-spec-validate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let input = root.join("sample-blueprint.yaml");
    let spec_path = root.join("sample-spec.yaml");
    std::fs::write(&input, sample_blueprint_yaml()).unwrap();

    let wrap = cli()
        .arg("spec")
        .arg("wrap")
        .arg(&input)
        .arg("--output")
        .arg(&spec_path)
        .output()
        .unwrap();
    assert!(wrap.status.success());

    let validate = cli()
        .arg("spec")
        .arg("validate")
        .arg(&spec_path)
        .output()
        .unwrap();

    assert!(
        validate.status.success(),
        "spec validate failed: {}\n{}",
        String::from_utf8_lossy(&validate.stdout),
        String::from_utf8_lossy(&validate.stderr)
    );
    assert!(String::from_utf8_lossy(&validate.stdout).contains("Specification OK: sample"));

    std::fs::remove_dir_all(&root).ok();
}
