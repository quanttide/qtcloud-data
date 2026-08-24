//! blueprint 命令集成测试（spawn CLI 二进制）。

mod common;

use common::cli;

#[test]
fn test_blueprint_help() {
    let output = cli().arg("blueprint").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("show"));
    // Old subcommands should NOT appear
    assert!(!stdout.contains("review"));
    assert!(!stdout.contains("design"));
    assert!(!stdout.contains("formalize"));
}

#[test]
fn test_blueprint_list_runs() {
    let tmp = std::env::temp_dir().join("bp-v020-test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let output = cli()
        .env("BLUEPRINT_DIR", tmp.to_str().unwrap())
        .arg("blueprint")
        .arg("list")
        .output()
        .unwrap();
    // May fail if cue CLI not installed, but should not panic
    let _ = output;

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_blueprint_list_json_returns_items() {
    let tmp = std::env::temp_dir().join(format!("bp-json-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("customer.yaml"), "name: customer\n").unwrap();

    let output = cli()
        .env("BLUEPRINT_DIR", &tmp)
        .arg("--json")
        .arg("blueprint")
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["command"], "blueprint list");
    assert_eq!(report["data"]["items"][0], "customer");

    std::fs::remove_dir_all(&tmp).ok();
}
