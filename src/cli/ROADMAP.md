# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。
> 当前源码版本：`qtcloud-data-cli` v0.2.0。
> crates.io 已发布版本仍为 v0.1.16，v0.2.0 待 owner 权限完成发布。

产品目标：把数据交付中简单但高频、容易卡进度的环节封装成命令，让 Git 中的约定文件成为人、AI、CLI、Provider、Studio 共用的事实源。

工作流愿景：

```text
Context -> clarify -> Requirements (DRD) -> design -> Specification -> implement -> execute -> report -> transfer -> Delivery
                                                     \-> Contract + Blueprint
```

## [0.2.1]

### Added
- [ ] 定义 manifest 输入契约，用一个 manifest 文件声明 raw、map、配置表、review decisions 等多输入。
- [ ] 增加 CLI 本地运行入口，支持按 Specification/Blueprint 发起 Provider run 或本地 runner。
- [ ] 补齐真实业务 resource 绑定工作流，避免交付项目长期停留在 `resource: builtin:copy`。
- [ ] 将 huangjian 类项目整理为正式 smoke/e2e 案例：正式 raw + `map.dta` 跑到 `review_master`。

### Changed
- [ ] 增强失败记录，把脚本 stdout/stderr 摘要和失败步骤上下文写入 job/catalog。

## [0.2.0] — 源码已合并 main

### Added
- [x] `doctor`：检查 Git、Rust、Python、CUE、数据目录和常见传输凭证。
- [x] `doctor --fix-dirs`：自动创建 `.quanttide/data/` 下的常用目录。
- [x] `doctor --json`：输出机器可读诊断结果，便于 Studio/CI 调用。
- [x] `spec wrap`：将 Blueprint YAML 包装为稳定 Specification YAML envelope。
- [x] `spec validate`：校验 Blueprint/Specification 结构。
- [x] `process` 执行后写入 job 记录，包括输入、pipeline、输出、状态和日志路径。
- [x] `catalog add` 与 `process` 输出联动，自动登记最终产物。
- [x] `transfer send --output` 默认支持交付链接记录。
- [x] `design blueprint` 默认生成 `pipeline.start_at` / `pipeline.states`，并写入 `resource: builtin:copy` 作为最小可执行 smoke-test 资源。

### Changed
- [x] CLI 内部依赖改为 crates.io 版本依赖，源码交付不依赖开发者本机 `D:\packages`。

## [0.1.16] — 已发布

### Added
- [x] `clarify from-chat <file>`：聊天记录/上下文 -> DRD。
- [x] `design contract <drd>`：DRD -> Contract（YAML + MD）。
- [x] `design blueprint <drd>`：DRD -> Blueprint（YAML + MD + HTML）。
- [x] `implement <yaml> --lang python`：Blueprint YAML -> Python 实现。
- [x] `review <input>`：审计 DRD 或 Specification。
- [x] `version {list,show,diff}`：规格版本管理。
- [x] `transfer {send,receive}`：网盘/对象存储数据传输。
- [x] `process`：串联 receive -> pipeline -> send。
- [x] `catalog`：本地数据目录登记。

### Changed
- [x] crates.io 包发布到 `qtcloud-data-cli` v0.1.16。
- [x] GitHub Release 使用 `cli/v0.1.16`。
- [x] 二进制包覆盖 Linux `x86_64-unknown-linux-gnu` 和 Windows `x86_64-pc-windows-msvc`。

## [0.2.2]

### Added
- [ ] 支持人工/AI review decisions 作为后续步骤输入。
- [ ] 接入 `merge_review` resource，将审核结果合并回匹配明细。
- [ ] 接入 `export` resource，生成最终客户交付文件。
- [ ] 在 catalog/job 记录里区分预审核产物、审核决策文件和最终交付产物。

## [0.3.0]

### Added
- [ ] CI 中构建 Linux、Windows、macOS 二进制包。
- [ ] Release 自动上传各平台产物。
- [ ] 发布前校验 Cargo.toml、CHANGELOG、Git tag 版本一致性。

### Changed
- [x] 将 `main` `[Unreleased]` 内容切分为 v0.2.0 发布准备版本，并同步 crates.io 发布元数据。

## [0.5.0]

### Added
- [ ] 新人通过 CLI + 工程标准快速接手历史项目。
- [ ] `review` 能自动发现跨项目不一致模式。
- [ ] 常见内部协调动作可以用命令串起来，而不是依赖聊天记录来回确认。
