# Agent Instructions — qtcloud-data CLI

本文件为 AI Agent 在 `qtcloud-data-cli` crate 中工作提供经验指引。

## 关键文档

| 文档 | 用途 |
|------|------|
| [README.md](README.md) | 快速开始、命令一览、安装 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 贡献指南、提交流程、文档映射 |
| [docs/index.md](docs/index.md) | 命令结构总览 + src 模块 ↔ 文档映射表（事实源） |

## 经验教训

- **CI 对齐**：重构/移动文件后先跑格式检查再提交（`cargo fmt`），并保持本地检查从严与 CI 一致（clippy `-D warnings`、`RUSTFLAGS="-D warnings" cargo test`）——本地双绿（fmt + 严格 lint）约等于 CI 绿。CI 失败先查是 fmt 差异还是逻辑错误（历史上多次 CI 失败均为 `cargo fmt --check` 未跑导致）。
- **发布提交原子性**：改 `Cargo.toml` 版本号时 `Cargo.lock` 必须同一提交——CI `cargo build --locked` 要求 lock 与 manifest 一致，分两个提交会导致中间提交 CI 必挂（v0.2.2 发布踩过）。
- **测试对齐**：集成测试按 `src/` 模块拆分 `tests/{module}_test.rs`，共享 helper 在 `tests/common/mod.rs`；测试文件命名与命令路径一致（如 `spec version` ↔ `tests/spec_version_test.rs`）。
- **模块组织**：`src/` 按域分组——`stage/`（生命周期动词流程）、`spec/`（Specification 域）、`storage/`（存储平台）、`implementation/`（实现资源）；`impl` 是 Rust 关键字不能作模块名，目录与模块名不同时用 `#[path]` 属性。
