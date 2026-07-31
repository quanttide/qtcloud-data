# TODO — CLI v0.2.1 真实项目执行闭环

> 对应 ROADMAP：`[0.2.1]`
> v0.2.0 已归档到 `CHANGELOG.md`，本文件只保留下一版本可执行拆解。

## manifest

- [ ] 定义 manifest YAML/JSON 最小字段（`docs/dev/specification.md`）
- [ ] 增加 manifest 解析与校验纯函数（`src/spec.rs`）
- [ ] 增加 manifest 校验命令或合入 `spec validate`（`src/main.rs`、`src/spec.rs`）
- [ ] 补充 manifest 单元测试和错误用例（`tests/blueprint_test.rs`）

## runner

- [ ] 设计 CLI 本地运行入口参数（`src/main.rs`）
- [ ] 实现按 Specification/Blueprint 发起 Provider run 的客户端调用（`src/process.rs`）
- [ ] 支持本地 runner smoke test，默认仍可执行 `builtin:copy`（`src/process.rs`）
- [ ] 补充端到端命令测试，覆盖输入、输出和 job 记录（`tests/blueprint_test.rs`）

## resources

- [ ] 为真实业务脚本约定 `python:<script>` resource 绑定方式（`docs/dev/specification.md`）
- [ ] 设计 huangjian 类项目的 smoke/e2e fixture 目录（`tests/`）
- [ ] 增加 stdout/stderr 摘要记录字段（`src/process.rs`）
- [ ] 更新 catalog/job 记录文档（`README.md`、`docs/user/transfer.md`）

## release

- [ ] 发布前确认 `Cargo.toml`、`CHANGELOG.md`、`ROADMAP.md`、Git tag 一致
- [ ] 创建并推送 `cli/v0.2.0` tag
- [ ] 由 crates.io owner 发布 `qtcloud-data-cli` v0.2.0
- [ ] 基于 `CHANGELOG.md` 创建 GitHub Release
