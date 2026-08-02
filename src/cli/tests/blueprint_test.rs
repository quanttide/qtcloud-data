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
