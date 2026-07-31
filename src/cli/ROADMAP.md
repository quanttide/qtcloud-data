# ROADMAP

> 当前发布版本：`qtcloud-data-cli` v0.1.16
> 本地 `[Unreleased]` 正在推进 v0.1.17/v0.1.18 的日常可用性，以及 v0.2.0 的 Provider 对齐。

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

## 下一步

### v0.1.17：日常环境可用性

- [x] `doctor`：检查 Git、Rust、Python、CUE、数据目录和常见传输凭证
- [x] `doctor --fix-dirs`：自动创建 `.quanttide/data/` 下的常用目录
- [x] `doctor --json`：输出机器可读诊断结果，便于 Studio/CI 调用
- [x] README 与 RELEASE_STATUS 持续对齐已发布版本

### v0.1.18：交付记录闭环

- [x] `process` 执行后写入 job 记录（输入、pipeline、输出、状态、日志路径）
- [x] `catalog add` 与 `process` 输出联动，自动登记最终产物
- [x] `transfer send --output` 默认支持交付链接记录

### v0.2.0：Provider 对齐

- [x] 固化 Specification YAML 结构，作为 CLI 和 Provider 的共同契约
- [x] `design blueprint` 默认生成 `pipeline.start_at` / `pipeline.states`，并写入 `resource: builtin:copy` 作为最小可执行 smoke-test 资源
- [x] Provider 读取 CLI 生成的 Blueprint YAML，并通过 `GET /blueprints` / `GET /blueprints/{name}` 暴露列表和详情
- [x] Blueprint 增加 Step Functions 风格的 `start_at` / `states` 状态机字段，继续保留 `steps` 兼容旧实现
- [x] Provider 接入最小 Pipeline 执行引擎，通过 `POST /blueprints/{name}/runs` 执行带 `resource` 的状态机/步骤，并写入 process job 记录
- [x] Studio 通过 Provider API 浏览 Blueprint 列表、详情和执行记录
- [x] Studio 在 Blueprint 详情页接入 run 表单，调用 Provider `POST /blueprints/{name}/runs`
- [x] Provider process job 记录支持文件持久化，并通过 `GET /process/jobs/{id}` 暴露单条详情
- [x] Studio 执行记录支持点击进入 job 详情，查看 step 输入、输出和状态

### v0.3.0：多平台发布自动化

- [ ] CI 中构建 Linux、Windows、macOS 二进制包
- [ ] Release 自动上传各平台产物
- [ ] 发布前校验 Cargo.toml、CHANGELOG、Git tag 版本一致性

### v0.5.0：团队日常好用

- [ ] 新人通过 CLI + 工程标准快速接手历史项目
- [ ] `review` 能自动发现跨项目不一致模式
- [ ] 常见内部协调动作可以用命令串起来，而不是依赖聊天记录来回确认
