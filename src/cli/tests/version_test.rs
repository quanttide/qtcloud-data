//! version 命令集成测试（spawn CLI 二进制）。

mod common;

use common::cli;

#[test]
fn test_version_help() {
    let output = cli().arg("version").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("show"));
    assert!(stdout.contains("diff"));
}
