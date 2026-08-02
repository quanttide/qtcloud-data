//! clarify 命令集成测试（spawn CLI 二进制）。

mod common;

use common::cli;

#[test]
fn test_clarify_help() {
    let output = cli().arg("clarify").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("from-chat"));
}

#[test]
fn test_clarify_from_chat_help() {
    let output = cli()
        .arg("clarify")
        .arg("from-chat")
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
}
