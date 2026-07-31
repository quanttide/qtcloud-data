# ROADMAP

> 当前发布准备版本：`qtcloud-data-cli` v0.2.0
> v0.2.0 已收口 CLI 规格流、交付记录和 Provider Blueprint runner 底座，正在准备发布到 crates.io/GitHub Release。

产品目标：把数据交付中简单但高频、容易卡进度的环节封装成命令和服务，让 Git 中的约定文件成为人、AI、CLI、Provider、Studio 共用的事实源。

## 工作流愿景

```text
Context -> clarify -> Requirements (DRD) -> design -> Specification -> implement -> execute -> report -> transfer -> Delivery
                                                     \-> Contract + Blueprint
```

CLI 优先覆盖的是内部日常交付链路中的碎片环节：

- 客户沟通上下文沉淀为 DRD
- DRD 生成可复用的 Specification YAML
- Specification 生成可执行代码骨架
- 审计、版本、目录、传输、编排统一走命令行
- 每个步骤的结果落到 `.quanttide/data/`，方便 Git diff、AI 读取和同事交接

## 已完成

### v0.1.16：crates.io 发布版

- `clarify from-chat <file>`：聊天记录/上下文 -> DRD
- `design contract <drd>`：DRD -> Contract（YAML + MD）
- `design blueprint <drd>`：DRD -> Blueprint（YAML + MD + HTML）
- `implement <yaml> --lang python`：Blueprint YAML -> Python 实现
- `review <input>`：审计 DRD 或 Specification
- `version {list,show,diff}`：规格版本管理
- `transfer {send,receive}`：网盘/对象存储数据传输
- `process`：串联 receive -> pipeline -> send
- `catalog`：本地数据目录登记

发布状态：

- crates.io 包：`qtcloud-data-cli` v0.1.16
- GitHub Release：`cli/v0.1.16`
- 二进制包：Linux `x86_64-unknown-linux-gnu`、Windows `x86_64-pc-windows-msvc`

真实数据验证：

- SEC 信用协议数据档案
- 化妆品检测报告数据档案

### v0.2.0：CLI 规格流和交付记录

- [x] `doctor`：检查 Git、Rust、Python、CUE、数据目录和常见传输凭证
- [x] `doctor --fix-dirs`：自动创建 `.quanttide/data/` 下的常用目录
- [x] `doctor --json`：输出机器可读诊断结果，便于 Studio/CI 调用
- [x] `spec wrap`：将 Blueprint YAML 包装为稳定 Specification YAML envelope
- [x] `spec validate`：校验 Blueprint/Specification 结构
- [x] `process` 执行后写入 job 记录（输入、pipeline、输出、状态、日志路径）
- [x] `catalog add` 与 `process` 输出联动，自动登记最终产物
- [x] `transfer send --output` 默认支持交付链接记录
- [x] `design blueprint` 默认生成 `pipeline.start_at` / `pipeline.states`，并写入 `resource: builtin:copy` 作为最小可执行 smoke-test 资源

### v0.2.0：Provider Blueprint runner 底座

- [x] 固化 Specification YAML 结构，作为 CLI 和 Provider 的共同契约
- [x] Provider 读取 CLI 生成的 Blueprint YAML，并通过 `GET /blueprints` / `GET /blueprints/{name}` 暴露列表和详情
- [x] Blueprint 增加 Step Functions 风格的 `start_at` / `states` 状态机字段，继续保留 `steps` 兼容旧实现
- [x] Provider 接入最小 Pipeline 执行引擎，通过 `POST /blueprints/{name}/runs` 执行带 `resource` 的状态机/步骤，并写入 process job 记录
- [x] Provider 支持 `builtin:copy`、`python:<script>`、`bash:<script>` 资源
- [x] Provider process job 记录支持文件持久化，并通过 `GET /process/jobs/{id}` 暴露单条详情

## 下一步

### v0.2.1：真实项目执行闭环

- [ ] 定义 manifest 输入契约，用一个 manifest 文件声明 raw、map、配置表、review decisions 等多输入
- [ ] 增加 CLI 本地运行入口，支持从命令行按 Specification/Blueprint 发起 Provider run 或本地 runner
- [ ] 补齐真实业务 resource 绑定工作流，避免交付项目长期停留在 `resource: builtin:copy`
- [ ] 增强 Provider 错误记录，保留脚本 stdout/stderr 摘要和失败步骤上下文
- [ ] 将 huangjian 类项目整理为正式 smoke/e2e 案例：正式 raw + `map.dta` 跑到 `review_master`

### v0.2.x：审核后交付阶段

- [ ] 支持人工/AI review decisions 作为后续步骤输入
- [ ] 接入 `merge_review` resource，将审核结果合并回匹配明细
- [ ] 接入 `export` resource，生成最终客户交付文件
- [ ] 在 catalog/job 记录里区分预审核产物、审核决策文件和最终交付产物

### v0.2.x：Studio 对齐

- [x] Studio 通过 Provider API 浏览 Blueprint 列表、详情和执行记录
- [x] Studio 在 Blueprint 详情页接入 run 表单，调用 Provider `POST /blueprints/{name}/runs`
- [x] Studio 执行记录支持点击进入 job 详情，查看 step 输入、输出和状态

### v0.3.0：多平台发布自动化

- [ ] CI 中构建 Linux、Windows、macOS 二进制包
- [ ] Release 自动上传各平台产物
- [ ] 发布前校验 Cargo.toml、CHANGELOG、Git tag 版本一致性
- [x] 将 `main` `[Unreleased]` 内容切分为 v0.2.0 发布准备版本，并同步 crates.io 发布元数据

### v0.5.0：团队日常好用

- [ ] 新人通过 CLI + 工程标准快速接手历史项目
- [ ] `review` 能自动发现跨项目不一致模式
- [ ] 常见内部协调动作可以用命令串起来，而不是依赖聊天记录来回确认
