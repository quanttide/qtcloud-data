//! 命令级集成测试共享 helper（spawn CLI 二进制）。

use std::process::Command;

#[allow(dead_code)]
/// spawn 编译好的 CLI 二进制（先 `cargo build`）。
pub fn cli() -> Command {
    Command::new("./target/debug/qtcloud-data")
}

#[allow(dead_code)]
/// 供 `spec wrap/validate` 测试使用的合法 Blueprint YAML。
pub fn sample_blueprint_yaml() -> &'static str {
    r#"name: "sample"
description: "稳定规格测试"
contract:
  input:
    schema: "raw: string"
    format: "CSV"
  output:
    schema: "clean: string"
    format: "CSV"
    rules:
      - 字段非空
pipeline:
  name: "sample-pipeline"
  steps:
    - name: "clean"
      from: "raw"
      to: "clean"
      desc: "trim whitespace"
status: draft
created_at: "2026-07-30T00:00:00+00:00"
updated_at: "2026-07-30T00:00:00+00:00"
"#
}
