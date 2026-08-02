# CHANGELOG

## [Unreleased]

### Changed
- 新增 `runtime/` 模块（`Runtime` trait + 注册表）：codegen（implement 用）+ execute（process 用），`from_name`/`from_ext` 注册表驱动；Python codegen 逻辑从 `stage/implement.rs` 分离，`process` 执行分发改注册表（`.py`→python / `.sh`→bash）。旧 `stage::implement::{implement_step_prompt, implement_assemble_prompt, to_snake}` 保留为 deprecated 转发（随 v0.3 移除）。
- `storage/` 统一概念命名：`StorageProvider` → `Storage`、`DropboxProvider` → `DropboxStorage`（等 6 平台）。旧名保留为 deprecated re-export（随 v0.3 移除）。

## [0.2.2] - 2026-08-02

### Changed
- `version` 命令降级为 `spec version` 子命令（`qtcloud-data spec version list/show/diff`）；顶层 `version` 标注废弃，v0.3 移除。
- 源码按域分组：生命周期模块（clarify/design/implement/process/transfer）移入 `stage/`，Specification 域（spec/blueprint/contract/version）移入 `spec/`，存储平台 `providers/` 改名 `storage/`，catalog/pipeline 移入 `implementation/`——lib 公开路径相应变更（破坏性变更）。
- `store` 模块拆分为 `registry`（JSON 注册表读写 + 原子写盘）与 `util`（数据目录解析 / UTC 时间）。lib 公开路径 `qtcloud_data_cli::store::*` 变更为 `registry::*` / `util::*`（破坏性变更）。
- 测试按 src/ 模块拆分 `tests/{module}_test.rs`（cli/clarify/design/review/spec/spec_version/doctor/blueprint/storage/process），共享 helper 收敛到 `tests/common/mod.rs`；`test_support::temp_dir` 改 RAII（`TempDir` Drop 自动清理，panic 也兜底）。
- CI clippy 从严 `-A warnings` → `-D warnings`（与本地检查一致）。
- 新增 `src/cli/AGENTS.md`（AI 代理经验指引：CI 对齐 / 测试对齐 / 模块组织）。

## [0.2.1] - 2026-08-02

### Added
- 统一 `store` 模块：合并 catalog/process/transfer 三份路径解析、UTC 时间格式化和 JSON 注册表读写（`Registry<T>`），写盘原子化（临时文件 + rename），替代三份重复拷贝。
- `catalog` 新增 `VolumeStatus` 状态枚举，替代魔法字符串；`registry.json` 落盘格式保持不变，未知状态字符串安全降级为 `unknown`，兼容旧数据。
- 基线 smoke/e2e：新增 `tests/fixtures/github-activity/` 真实业务 fixture 与 `process` 全链路回归测试（内容级产物断言 + URL 脱敏校验），断言细节见 `tests/fixtures/README.md`。
- 统一错误模型：新增 `CliError` 类型，`process` 命令入口返回 `Result<(), CliError>`，`main` 顶层统一错误格式化。
- `process` 抽取 StepExecutor 状态机（Receive → Pipeline → Send），收敛 5 处重复失败处理，失败统一落 failed job 记录。
- `transfer send` / `transfer receive` 抽为进程内服务函数，`process` 改为进程内组合（替代自我 re-exec）；`QTDATA_CLI` 保留为测试/部署逃生舱。

### Changed
- `pipeline list/show` 与 `blueprint list/show`：cue 输出改 `--out json` 结构化解析，替代文本 grep。
- `process` 的 blueprint pipeline 解析（`resolve_blueprint_pipeline`）：改结构化 JSON 解析，替代文本 trim。
- `contract list/show`：以文件直读为主路径（cue 为可选增强），不再依赖 cue 解析 YAML。
- 全部命令错误处理统一为 `Result<(), CliError>`：`std::process::exit(1)` 迁移到 `main` 顶层统一格式化（`错误: {err}` + 退出码 1），错误路径可测试；`transfer send/receive` 失败时退出码不再为 0。
- `dropbox` 上传失败由 `panic!` 改为返回错误；`blueprint`/`pipeline` 的 cue 缺失/解析失败由 `expect` 改为优雅错误。

## [0.2.0] - 2026-08-01

### Added
- `doctor` 命令：检查本机 DataOps 环境、常用工具、数据目录和传输凭证配置状态。
- `doctor --fix-dirs`：自动创建常用 `.quanttide/data/` 目录。
- `doctor --json`：输出机器可读诊断报告，便于 CI、Studio 或脚本集成。
- `DATA_ROOT`：统一覆盖默认数据根目录，子目录环境变量仍可单独覆盖。
- `process` 成功或失败后写入 job 记录到 `CATALOG_DIR/jobs.json`，并生成对应日志路径，便于交付追踪。
- `process` 成功交付后自动把最终产物登记到 `CATALOG_DIR/registry.json`，provider 为 `process`，状态为 `delivered`。
- `transfer send` 成功后写入交付链接记录到 `CATALOG_DIR/delivery-links.json`；使用 `--output` 时仍会同时写入指定链接文件。
- `spec wrap` / `spec validate`：把已有 Blueprint YAML 包装为稳定 Specification envelope，并校验旧 YAML 或 envelope，供 Provider 后续对齐读取。
- `design blueprint` 生成 Step Functions 风格的 `pipeline.start_at` / `pipeline.states`，并默认给 state/step 写入 `resource: builtin:copy`，供 Provider 做最小可执行 smoke test。

### Fixed
- 修复 CLI release workflow 的 package dry-run：CHANGELOG release notes 预检改写入 runner 临时目录，避免生成未提交的 `release-notes.md` 导致 crates.io dry-run 失败。

## [0.1.16] - 2026-07-27

### Fixed
- Improved crates.io release availability when shared dependency crates were already published.

## [0.1.15] - 2026-07-27

### Fixed
- Fixed binary artifact upload paths for the scoped CLI release.

## [0.1.14] - 2026-07-27

### Fixed
- Restored dependency resolution for the v0.1.x release pipeline.

## [0.1.13] - 2026-07-27

### Changed
- Maintenance-only release for v0.1.x packaging; no command behavior changed.

## [0.1.12] - 2026-07-25

### Fixed
- Kept the CLI publish step independent from already-published shared crates.

## [0.1.11] - 2026-07-25

### Fixed
- Surfaced crates.io publish failures instead of treating them as successful releases.

## [0.1.10] - 2026-07-25

### Changed
- Verified the `implement` command packaging path for the v0.1.x CLI line.

## [0.1.9] - 2026-07-25

### Fixed
- Treated already-published shared dependency crates as non-fatal during CLI release.

## [0.1.8] - 2026-07-25

### Fixed
- Fixed the scoped `src/cli` package path used by release automation.

## [0.1.7] - 2026-07-25

### Fixed
- Fixed crates.io token handling for automated CLI publishing.

## [0.1.6] - 2026-07-25

### Changed
- Maintenance-only release for v0.1.x packaging; no command behavior changed.

## [0.1.5] - 2026-07-25

### Fixed
- Stabilized release checks for the v0.1.x CLI package.

## [0.1.4] - 2026-07-25

### Fixed
- Stabilized release checks for the v0.1.x CLI package.

## [0.1.3] - 2026-07-25

### Fixed
- Removed duplicate data reads in `sftp.rs`.

## [0.1.2] - 2026-07-25

### Fixed
- Stabilized the SFTP implementation for the v0.1.x release line.

## [0.1.1] - 2026-07-25

### Changed
- Maintenance-only release for v0.1.x packaging; no command behavior changed.

## [0.1.0] - 2026-07-24

### Added
- `implement <yaml> --lang python`：从 Blueprint YAML 生成 Python 代码实现。
- CI pipeline：push 自动 build、test、clippy、fmt。
- Release pipeline：tag 触发 crates.io 发布和多平台二进制打包。

### Changed
- `rc.1` 到 `rc.2` 再到 `0.1.0` 的发布过程固定为逐版递增，避免移动 tag。
- Release workflow 拆为 `publish-crate` 和 `build-binary` 两阶段。

### Fixed
- 修复 Release 上传制品路径匹配。

## [0.1.0-beta.1] - 2026-07-24

### Added
- `clarify from-chat <file>`：从聊天记录或上下文生成数据需求文档（DRD）。
- `design contract <drd>`：从 DRD 生成数据契约（Contract: .yaml + .md）。
- `design blueprint <drd>`：从 DRD 生成处理蓝图（Blueprint: .yaml + .md + .html）。
- `review <input>`：提升为顶级命令，审计 Specification。
- `version {list,show,diff}`：提升为顶级命令。

### Changed
- `blueprint` 子命令迁移：review/design/formalize/preview/version -> clarify/design/review/version。
- `design formalize` 保留，支持 md -> YAML 形式化。
- `design preview` 保留，支持 YAML -> HTML 渲染。
- CUE 格式全局替换为 YAML。
- `contract_tables_to_yaml` / `blueprint_table_to_yaml` 改为 LLM 输出 Markdown 表格、代码确定性生成 YAML。
- 目录结构调整为 `.quanttide/data/drd/` 和 `.quanttide/data/spec/`。

### Removed
- 移除 `quanttide-data-core` 的 from_cue.rs / to_cue.rs 手写 CUE 解析器。

## [0.1.0-alpha.1] - 2026-07-17

### Added
- `blueprint review`：审计已有 Blueprint，LLM 输出结构化问题清单。
- `blueprint design new`：生成 .md Blueprint 模板。
- `blueprint design edit`：编辑已有 .md Blueprint。
- `blueprint formalize`：Markdown -> LLM -> CUE 形式化。
- `blueprint preview`：CUE -> HTML 可视化渲染。
- `blueprint version list|show|diff`：git-based 版本管理。

### Changed
- Blueprint 模块拆分为纯逻辑层 `blueprint_core` 与 I/O 薄壳层。
- LLM 调用统一走 `quanttide-agent` 接口。
- Blueprint 数据模型和 LLM 调用开始接入共享工具包。

## [0.0.5] - 2026-07-10

### Changed
- 默认目录改为 `.quanttide/data/` 规范：pipelines / blueprints / contracts。
- WORKDIR 改为系统临时目录 `/tmp/qtcloud-data`，用完自动清理。
- README 更新：添加命令一览表、S3/SFTP 平台、process 示例。

## [0.0.4] - 2026-07-10

### Added
- `process` 命令：编排 receive -> pipeline -> send 完整流程。
- `pipeline list|show` 命令：查看 CUE 定义的管道。
- `blueprint list|show` 命令：查看 CUE 定义的蓝图。
- `contract list|show` 命令：查看独立契约定义。
- 全覆盖测试（9 个用例）。

## [0.0.3] - 2026-07-10

### Added
- S3 provider（`--provider s3`）：PutObject + 预签名 URL + 自动接收。
- SFTP provider（`--provider sftp`）：密钥/密码认证，支持 `sftp://` URL 和自动模式。
- receive 双模式：手动（URL）和自动（路径）。

## [0.0.2] - 2026-07-10

### Added
- StorageProvider trait 架构，统一 send/receive 接口。
- 百度网盘支持（`--provider baidu`）。
- Google Drive 支持（`--provider google`）。
- OneDrive 支持（`--provider onedrive`）。
- wiremock 集成测试（6 个用例覆盖 send/receive/错误处理）。
- 开发者文档（`docs/transfer.md`）。
- 用户文档（`docs/transfer.md`）。

### Changed
- CLI 增加 `--provider` 选项（默认 `dropbox`），receive 时自动从 URL 识别提供商。
- 重构 dropbox 模块为 provider 模式。

### Removed
- 夸克网盘支持（无官方 API）。

## [0.0.1] - 2026-07-10

### Added
- 初始版本：Dropbox 数据传输 CLI。
- `transfer send`：上传文件到网盘并生成分享链接。
- `transfer receive`：从共享链接下载文件。
- `transfer ls`：列出网盘中的客户目录。
- wiremock 集成测试框架。
