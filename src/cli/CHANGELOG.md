# CHANGELOG

## [0.1.0-beta.1] - 2026-07-24

### Added
- `clarify from-chat <file>` — 从聊天记录/上下文生成数据需求文档（DRD）
- `design contract <drd>` — 从 DRD 生成数据契约（Contract: .yaml + .md）
- `design blueprint <drd>` — 从 DRD 生成处理蓝图（Blueprint: .yaml + .md + .html）
- `review <input>` — 提升为顶级命令，审计 Specification
- `version {list,show,diff}` — 提升为顶级命令

### Changed
- `blueprint` 子命令迁移：review/design/formalize/preview/version → clarify/design/review/version
- `design formalize` 保留，md → YAML 形式化
- `design preview` 保留，YAML → HTML 渲染
- CUE 格式全局替换为 YAML
- `contract_tables_to_yaml` / `blueprint_table_to_yaml`：LLM 输出 Markdown 表格，代码确定性生成 YAML
- 目录结构：`.quanttide/data/drd/` + `.quanttide/data/spec/`

### Removed
- `quanttide-data-core` 的 from_cue.rs / to_cue.rs（1716行手写 CUE 解析器）

## [0.1.0-alpha.1] - 2026-07-17

### Added
- `blueprint review` — 审计已有 Blueprint，LLM 输出结构化问题清单
- `blueprint design new` — 生成 .md Blueprint 模板
- `blueprint design edit` — 编辑已有 .md Blueprint
- `blueprint formalize` — Markdown → LLM → CUE 形式化
- `blueprint preview` — CUE → HTML 可视化渲染
- `blueprint version list|show|diff` — git-based 版本管理

### Changed
- Blueprint 模块拆分为纯逻辑层 `blueprint_core` 与 I/O 薄壳层
- LLM 调用统一走 `quanttide-agent` 接口

### Added (dependencies)
- `quanttide-data-core` — Blueprint 数据模型 + CUE 解析器
- `quanttide-agent` — LLM 统一接口

## [0.0.5] - 2026-07-10

### Changed
- 默认目录改为 `.quanttide/data/` 规范：pipelines / blueprints / contracts
- WORKDIR 改为系统临时目录 `/tmp/qtcloud-data`，用完自动清理
- README 更新：添加命令一览表、S3/SFTP 平台、process 示例
- Cargo.toml 版本更新为 0.0.5

## [0.0.4] - 2026-07-10

### Added
- `process` 命令：编排 receive → pipeline → send 完整流程
- `pipeline list|show` 命令：查看 CUE 定义的管道
- `blueprint list|show` 命令：查看 CUE 定义的蓝图
- `contract list|show` 命令：查看独立契约定义
- 全覆盖测试（9 个用例）

### Changed
- Cargo.toml 版本更新为 0.0.4

## [0.0.3] - 2026-07-10

### Added
- S3 provider（`--provider s3`）：PutObject + 预签名 URL + 自动接收
- SFTP provider（`--provider sftp`）：密钥/密码认证，支持 `sftp://` URL 和自动模式
- receive 双模式：手动（URL）和自动（路径）

### Changed
- Cargo.toml 版本更新为 0.0.3

## [0.0.2] - 2026-07-10

### Added
- StorageProvider trait 架构，统一 send/receive 接口
- 百度网盘支持（`--provider baidu`）
- Google Drive 支持（`--provider google`）
- OneDrive 支持（`--provider onedrive`）
- wiremock 集成测试（6 个用例覆盖 send/receive/错误处理）
- 开发者文档（`docs/dev/transfer.md`）
- 用户文档（`docs/user/transfer.md`）

### Changed
- CLI 增加 `--provider` 选项（默认 `dropbox`），receive 时自动从 URL 识别提供商
- 重构 dropbox 模块为 provider 模式
- Cargo.toml 版本从 0.1.0 更新为 0.0.2

### Removed
- 夸克网盘支持（无官方 API）

## [0.0.1] - 2026-07-10

### Added
- 初始版本：Dropbox 数据传输 CLI
- `transfer send`：上传文件到网盘并生成分享链接
- `transfer receive`：从共享链接下载文件
- `transfer ls`：列出网盘中的客户目录
- wiremock 集成测试框架
