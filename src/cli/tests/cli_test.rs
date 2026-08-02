//! CLI 入口集成测试（整体 help）。

mod common;

use common::cli;

#[test]
fn test_cli_help_shows_all_commands() {
    let output = cli().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clarify"));
    assert!(stdout.contains("design"));
    assert!(stdout.contains("review"));
    assert!(stdout.contains("version"));
    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("blueprint"));
    assert!(stdout.contains("spec"));
}
