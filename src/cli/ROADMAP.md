# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。
> ROADMAP 只保留未来计划；已发布内容以 CHANGELOG 为事实源。
> `cli/v0.2.2` 已于 2026-08-02 发布。原 0.2.2 中未完成的后续工作已重新归入 0.3.0 或延期项。
> `0.3.0` 仍处于开发中；Manifest、Catalog、查看命令解耦、Runtime 扩展和结构化错误输出首切片已完成，尚未发布。

## [0.3.0]

> 目标：补齐 manifest/catalog 契约、降低查看命令对 cue 的依赖、扩展运行时注册表，并提供可供 Studio/CI 消费的结构化输出。当前剩余重点是成功结果统一模型、传输补测和发布前验收。

### Added

- [x] manifest 契约：定义 raw、map、配置表和 `review_decisions[]` 的输入格式（`docs/specification.md`、`src/spec/mod.rs`）
- [x] manifest 校验：增加 YAML 结构校验、字段级错误提示和命令级测试（`src/spec/mod.rs`、`tests/spec_test.rs`）
- [x] manifest 严格复核补齐：Manifest 形状但 `kind` 缺失/错误时仍进入 Manifest 校验，并校验 `review_decisions[].target` 引用已声明物料（`src/spec/mod.rs`、`tests/spec_test.rs`）
- [x] catalog 产物类型：区分预审核产物、审核决策文件和最终交付产物（`src/implementation/catalog.rs`、`docs/catalog.md`）
- [x] catalog 状态闭环：补齐 `processing` / `processed` 的设置路径，并让 `process` 生命周期更新状态（`src/implementation/catalog.rs`、`src/stage/process.rs`）
- [x] catalog 阶段三严格验收：重跑 catalog/process 命令级测试、库内 catalog/process 单元测试，确认旧 registry 兼容和失败补偿路径通过（`tests/catalog_test.rs`、`tests/process_test.rs`、`src/implementation/catalog.rs`、`src/stage/process.rs`）
- [x] 百度网盘和 SFTP 传输补测，目标覆盖率从当前缺口提升到至少 50%（`src/storage/baidu_drive.rs`、`src/storage/sftp.rs`、`tests/storage_test.rs`）
- [x] R / Stata runtime：补充运行时实现、codegen prompt 和执行注册（`src/runtime/`、`src/stage/implement.rs`、`src/stage/process.rs`）
- [x] doctor 检查表改由 runtime 注册表驱动，并覆盖已注册运行时（`src/doctor.rs`、`src/runtime/mod.rs`）
- [x] 全局 `--json` 错误通道和稳定错误码首切片（`src/main.rs`、`src/cli.rs`、`src/error.rs`）
- [x] 成功命令统一结果模型：优先迁移无副作用查看命令和 catalog/pipeline 命令（`src/cli.rs`、`src/spec/`、`src/implementation/`）

### Changed

- [x] Blueprint/Pipeline `list/show` 以文件直读为主，cue 降为可选增强（`src/spec/blueprint.rs`、`src/implementation/pipeline.rs`）
- [x] doctor 将 cue 检查降为 optional，安装 CLI 后不再暴露 cue 为必需依赖（`src/doctor.rs`、`docs/doctor.md`）
- [x] 传输 provider 使用枚举或结构化解析，替代字符串匹配（`src/stage/transfer.rs`、`src/storage/mod.rs`）
- [x] process 使用结构化 Blueprint states 引用，替代逗号分隔的 pipeline 字符串（`src/stage/process.rs`）
- [x] 将命令分发移入库层，使 `Commands` 和统一参数注入路径可单测（`src/main.rs`、`src/cli.rs`、`src/lib.rs`）
- [x] 补充 deploy、operate、monitor 阶段记录（`docs/`）
- [x] 核对并补齐父仓库各平台 release workflow，确保 Linux、Windows、macOS 制品与发布记录一致（`../../.github/workflows/release-cli.yml`）

## Provider 依赖项（不作为 0.3.0 发布门槛）

> 以下工作依赖 Provider ROADMAP [0.0.3] 的 `merge_review` / `export` 能力，待 Provider 稳定后排期。

- [ ] 增加 `PROVIDER_URL` 配置 Provider 服务地址（`src/`、`docs/specification.md`）
- [ ] 增加按 Specification/Blueprint 调用 Provider run API 的 CLI 入口（`src/`）
- [ ] 增加 run 请求参数校验和错误提示（`src/`、`docs/specification.md`）
- [ ] 完成 raw + map.dta → review_master 业务 e2e（`tests/`）
- [ ] 保存业务 e2e 的输入、输出和验证记录（`docs/`）

## [0.5.0]

### Added

- [ ] 新人通过 CLI 和工程规范快速接手历史项目（`README.md`、`docs/`）
- [ ] 将常见内部协调动作封装成可重复命令（`src/stage/process.rs`、`src/`）

### Changed

- [ ] review 自动发现跨项目不一致模式（`src/review.rs`）

## 未分配版本

- [ ] 将凭证环境变量名集中为常量表，减少 provider 和 doctor 中的魔法字符串（`src/storage/mod.rs`、`src/doctor.rs`）
- [ ] 将 doctor 的 env 注入模式推广到其他模块（参考 `data_dirs_with`，`src/doctor.rs`）
