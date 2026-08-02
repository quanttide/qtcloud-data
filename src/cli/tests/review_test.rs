//! review 命令集成测试（spawn CLI 二进制）。

mod common;

use common::cli;

#[test]
fn test_review_help() {
    let output = cli().arg("review").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("INPUT"));
}
