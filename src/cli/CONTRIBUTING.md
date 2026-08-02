# qtcloud-data CLI — 贡献指南

欢迎贡献 `qtcloud-data-cli`！本文档提供本 crate 的开发环境、提交流程和发布流程。

## 仓库定位

`src/cli` 是量潮数据云（qtcloud-data）的命令行工具 crate（`qtcloud-data-cli`），位于
`apps/qtcloud-data` 子模块内。CLI 把数据交付里容易卡住的碎片环节串成可重复命令。

命令结构与分类（生命周期 / 横向工具 / 查看）以 [docs/dev/index.md](docs/dev/index.md) 为事实源，
本文件不重复展开——贡献流程请往下看，命令用法见 [README.md](README.md)。

相关文档（文档与 `src/` 模块一一对应，见 [docs/dev/index.md](docs/dev/index.md) 映射表）：

| 文档 | 对应模块 | 用途 |
|------|---------|------|
| [README.md](README.md) | — | 快速开始、命令一览、安装 |
| [ROADMAP.md](ROADMAP.md) | — | 版本里程碑和 checkbox 计划 |
| [TODO.md](TODO.md) | — | 按模块拆解的执行任务 |
| [CHANGELOG.md](CHANGELOG.md) | — | 版本变更记录（发布事实源） |
| [docs/dev/index.md](docs/dev/index.md) | main/lib/error | 命令结构总览 + 文档映射表 + 错误模型 |
| [docs/dev/transfer.md](docs/dev/transfer.md) | transfer/providers | 传输服务与 StorageProvider |
| [docs/dev/data-format.md](docs/dev/data-format.md) | catalog/store | 数据格式（registry/jobs/delivery-links 字段级） |
| [docs/dev/process.md](docs/dev/process.md) | process | StepExecutor 编排 |
| [docs/dev/llm.md](docs/dev/llm.md) | clarify/design/implement/review | LLM 命令与 Handler 注入 |
| [docs/dev/specification.md](docs/dev/specification.md) | spec/blueprint_core | Specification YAML 契约 |
| [docs/dev/view.md](docs/dev/view.md) | blueprint/contract/pipeline | 查看命令与文件直读策略 |
| [docs/dev/tooling.md](docs/dev/tooling.md) | doctor/version | 环境检查与版本管理 |

## 开发环境

### 前置依赖

- Rust（edition 2024，建议 nightly 之外的最新 stable）
- Git（≥ 2.20）
- cue（可选）：`pipeline list/show`、`blueprint list/show` 的增强解析；`doctor` 检查项

### 构建与测试

```bash
cd src/cli
cargo build --locked
cargo test --locked
cargo clippy --locked -- -D warnings   # CI 严格 lint，本地应保持零告警
cargo fmt --check
```

> **注意**：CI（`.github/workflows/test-cli.yml`）在 `RUSTFLAGS=-D warnings` 下运行，
> 本地提交前请用相同参数验证（`RUSTFLAGS="-D warnings" cargo test --locked`）。

### 测试分层

| 层次 | 位置 | 手段 |
|---|---|---|
| 单元测试 | `src/*.rs` 内 `mod tests` | 纯函数 + env 注入 + `test_support::{temp_dir, write_script, fake_llm}` |
| 集成测试 | `tests/blueprint_test.rs`、`tests/integration_test.rs`、`tests/provider_test.rs` | spawn 二进制、wiremock 模拟 HTTP |
| e2e | `tests/e2e_baseline.rs` | 真实 fixture 全链路（`tests/fixtures/github-activity/`） |

覆盖率：`cargo llvm-cov test --workspace`（当前 83.7%）。

## 提交规范

提交消息遵循 `.gitmessage` 格式：

```
<type>(<scope>): <description>
```

- `<type>`：`feat` / `fix` / `docs` / `style` / `refactor` / `test` / `chore` / `init`
- `<scope>`（可选）：本 crate 内通常省略；跨 crate 用 `provider` / `studio` / `qtcloud-data`
- `<description>`：中文祈使句、现在时态、首字母小写、不加句号

示例：

```bash
git commit -m "refactor: 全部命令错误处理统一 CliError"
git commit -m "test: 补测 fake cue 路径"
git commit -m "docs: 更新覆盖率基线"
```

**提交即推送**：提交后默认推送到远端（`git push origin main`），子模块提交后需在父仓库更新子模块指针并推送。

## 发布流程

CLI 发布遵循 `plan → code → build → test → release → deploy → operate → monitor` 生命周期，
自动化覆盖 `plan` 到 `release` 阶段，后续阶段记录在 `ROADMAP.md`。

### 发布原则

- 小而可审查的发布
- SemVer；多组件仓库中 CLI tag 用 `cli/vX.Y.Z`
- tag 不可变，绝不移动已存在的 release tag
- `CHANGELOG.md` 是发布事实源，GitHub Release notes 从对应条目生成
- 不在开发者笔记本上手动 publish crates；以 `qtcloud-devops release publish` 为发布入口

### 操作检查（发布前）

1. `src/cli/Cargo.toml` 包含目标版本
2. `src/cli/CHANGELOG.md` 包含 `## [X.Y.Z] - YYYY-MM-DD`
3. 目标 tag 不存在
4. release-prep 提交后工作树干净
5. 变更已走 feature 分支 → Pull Request → review → `main` 合并流程
6. build / test / clippy / format 全部通过

### 发布步骤

**1. Plan**：更新 `ROADMAP.md` 里程碑，把剩余工作拆分到 `TODO.md`

```bash
qtcloud-devops plan status --scope cli
qtcloud-devops plan audit --scope cli
```

**2. Code**：在 feature 分支上完成实现与文档改动

```bash
git switch -c codex/cli-v0.2.X-release
qtcloud-devops code audit src/cli
```

**3. Build and Test**

```bash
qtcloud-devops build status
qtcloud-devops test status

cd src/cli
cargo fmt --check
cargo build --locked
cargo test --locked
cargo clippy --locked -- -A warnings
```

**4. 更新发布记录**：`Cargo.toml` 版本号 + `CHANGELOG.md` 发布条目 + `ROADMAP.md`/`TODO.md` 勾选

**5. 提交与审查**：release-prep 提交（`chore(cli): prepare v0.2.X release`），
feature 分支推远端 → Pull Request → review → 合并 `main`。release tag 必须指向 `main` 可达提交。
若变更已在 `main` 且 CI 通过（`push: [main]` 触发），可直接 `main` 发布。

**6. 发布预检**（合并后，从干净的 `main` checkout）

```bash
qtcloud-devops release status
qtcloud-devops release audit -v cli/v0.2.X --scope cli
qtcloud-devops release publish -v cli/v0.2.X --registry crates --dry-run
```

dry-run 不得创建 tag、GitHub Release 或 crates.io 版本。

**7. 发布**（maintainer 确认后）

```bash
qtcloud-devops release publish -v cli/v0.2.X --registry crates -y
```

命令创建并推送 `cli/v0.2.X` tag，随后 `release-cli.yml` GitHub Actions 完成：
检查 tag/Cargo 版本/CHANGELOG/clean checkout/main 祖先 → 构建测试 → 发布 crates.io →
构建 Linux/Windows 二进制 → 创建 GitHub Release（notes 取自 CHANGELOG）。

**8. 验证**

```bash
cargo info qtcloud-data-cli --registry crates-io
cargo install qtcloud-data-cli --version <X.Y.Z>
qtcloud-data doctor --no-fail
qtcloud-data spec --help
qtcloud-data process --help
```

已发布版本记录：`cli/v0.2.0`（2026-08-01）、`cli/v0.2.1`（2026-08-02）。

### 本地 Cargo 镜像说明

若 Cargo 配置为用 `rsproxy` 替换 crates.io，本地 dry-run 需显式指定 crates.io。
该命令仅为包预检，不得用于正式发布：

```bash
cd src/cli
cargo publish --locked --dry-run --registry crates-io --allow-dirty
```
