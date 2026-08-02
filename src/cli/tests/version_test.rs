//! version（spec version）命令集成测试（spawn CLI 二进制）。
//! 顶层 `version` 已废弃（v0.3 移除），主入口为 `spec version`。

mod common;

use common::cli;

#[test]
fn test_spec_version_help() {
    let output = cli().arg("spec").arg("version").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("show"));
    assert!(stdout.contains("diff"));
}

#[test]
fn test_legacy_version_command_annotated_deprecated() {
    // v0.3 移除前仍可用，帮助标注废弃与替代入口
    let output = cli().arg("version").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("废弃"));
    assert!(stdout.contains("spec version"));
}
