# deploy — 发布状态

本页记录 `qtcloud-data` CLI 的发布与制品状态，写实为准。

## 当前状态

- 版本目标：`cli/v0.3.0`
- 发布入口：`qtcloud-devops release publish`
- 现在还没有正式发布完成

## 制品矩阵

`src/cli/Cargo.toml` 的 `cargo-dist` 目标是：

- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `aarch64-apple-darwin`
- `x86_64-apple-darwin`

对应的 release workflow 已补齐 Linux、Windows、macOS arm64、macOS Intel 的构建与上传条目。

## 发布审计

2026-08-24 的 `qtcloud-devops release audit -v cli/v0.3.0 --scope cli` 结果：

- 版本号格式：通过
- 配置文件一致性：通过
- CHANGELOG：通过
- 工作区状态：未通过，存在未提交变更
- 标签冲突：未通过，`cli/v0.3.0` 已存在
- GitHub Release：未通过，body 与 CHANGELOG 不同步

