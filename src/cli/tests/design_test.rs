//! design 命令集成测试（spawn CLI 二进制）。

mod common;

use common::cli;

#[test]
fn test_design_help() {
    let output = cli().arg("design").arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("contract"));
    assert!(stdout.contains("blueprint"));
    assert!(stdout.contains("formalize"));
    assert!(stdout.contains("preview"));
}

#[test]
fn test_design_subcommands_help() {
    for sub in &["contract", "blueprint", "formalize", "preview"] {
        let output = cli().arg("design").arg(sub).arg("--help").output().unwrap();
        assert!(output.status.success(), "design {sub} --help failed");
    }
}
