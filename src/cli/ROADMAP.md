# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。
> ROADMAP 面向未来计划；发布后将已完成条目迁移到 CHANGELOG。

## [0.2.1]

> 行为不变的内部治理；发布前需 `cargo test` 全绿 + 命令输出 diff 零变化。
> Provider 侧资源开发（merge_review / export / resource 绑定工作流）在 `../provider/` ROADMAP [0.0.3] 推进，不阻塞本版本。

### Added

- [x] `src/lib.rs` 新增统一 store 模块：合并 catalog/process/transfer 三份路径、时间、JSON 读写拷贝（含原子写盘与 env 解析统一）
- [x] `src/catalog.rs` status 枚举化替代魔法字符串
- [x] `tests/` 增加基线 smoke/e2e 案例：现有 `process` 命令全链路回归保护（fixture + 验收：`cargo test` 全绿）

### Changed

- [x] `src/pipeline.rs` cue 输出改结构化 JSON 解析，替代文本 grep
- [x] `src/blueprint.rs` cue 输出改结构化 JSON 解析，替代文本 grep
- [x] `src/contract.rs` 以文件直读为主路径（cue 为可选增强），替代 cue grep
- [x] `src/process.rs` resolve_blueprint_pipeline 改结构化 JSON 解析，替代文本 trim

## [0.2.2]

> 错误模型与执行器重构；行为变化集中在本版本（失败输出格式统一、process 不再自我 re-exec）。

### Added

- [ ] `src/process.rs` 统一错误模型（CliError），命令入口返回 `Result<(), CliError>`，`src/main.rs` 顶层统一格式化
- [ ] `src/process.rs` 抽取 StepExecutor 状态机（Receive → Pipeline → Send），收敛重复失败处理
- [ ] `src/process.rs` StepExecutor 单元测试（tempfile + 注入式路径）

### Changed

- [ ] `src/transfer.rs` send/receive 抽为进程内服务函数，process 库内组合替代自我 re-exec

## [0.2.3]

> manifest + Provider 打通；新功能版本，依赖 v0.2.1 的 store/catalog 枚举化，与 Provider ROADMAP [0.0.3] 协同。

### Added

- [ ] `docs/dev/` 定义 manifest 输入契约（raw[] / map / review_decisions[]），`src/spec.rs` 增加 manifest YAML 校验和错误提示
- [ ] `src/process.rs` 增加通过 CLI 发起 Provider run 的执行入口（`PROVIDER_URL` 配置 + run 请求参数校验）
- [ ] `src/catalog.rs` catalog/job 记录区分预审核产物、审核决策文件和最终交付产物（status 枚举已在 v0.2.1 落地）
- [ ] `tests/` 增加业务 e2e：raw + map.dta → review_master 全链路（依赖 Provider merge_review / export）

## [0.3.0]

### Added

- [ ] `Cargo.toml` 构建 Linux、Windows、macOS 二进制包
- [ ] `Cargo.toml` 自动上传各平台 Release 制品
- [ ] `src/lib.rs` 新增 runtimes 模块：RuntimeAdapter trait 与注册表（python / r / stata / matlab / bash / builtin）
- [ ] `src/process.rs` run_pipeline 改为注册表查表，替代扩展名 if-else 分发
- [ ] `src/implement.rs` implement 支持 --lang r / --lang stata
- [ ] `src/blueprint_core.rs` 新增 R / Stata codegen prompt 模板
- [ ] `src/doctor.rs` 检查表由 RuntimeAdapter 注册表驱动
- [ ] `src/main.rs` 全局 --json 结构化输出，供 Studio/CI 消费

### Changed

- [ ] `docs/dev/` 补充发布后的 deploy、operate、monitor 阶段记录

## [0.5.0]

### Added

- [ ] `README.md` 新人通过 CLI 和工程规范快速接手历史项目
- [ ] `src/process.rs` 将常见内部协调动作封装成可重复命令

### Changed

- [ ] `src/review.rs` review 自动发现跨项目不一致模式
