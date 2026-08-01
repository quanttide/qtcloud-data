# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。
> ROADMAP 面向未来计划；发布后将已完成条目迁移到 CHANGELOG。

## [0.2.1]

### Added

- [ ] `src/process.rs` 定义 manifest 输入契约，支持多输入数据流
- [ ] `src/process.rs` 增加通过 CLI 发起 Provider run 的执行入口
- [ ] `tests/` 增加真实项目 smoke/e2e 案例
- [ ] `src/process.rs` 统一错误模型（CliError），命令入口返回 `Result<(), CliError>` 顶层统一格式化
- [ ] `src/process.rs` 抽取 StepExecutor 状态机（Receive → Pipeline → Send），收敛重复失败处理
- [ ] `src/lib.rs` 新增统一 store 模块：合并 catalog/process/transfer 三份路径、时间、JSON 读写拷贝
- [ ] `src/transfer.rs` send/receive 抽为进程内服务函数，process 库内组合替代自我 re-exec
- [ ] `../provider/` 接入 merge_review resource，将审核结果合并回匹配明细
- [ ] `../provider/` 接入 export resource，生成最终客户交付文件
- [ ] `src/catalog.rs` 在 catalog/job 记录里区分预审核产物、审核决策文件和最终交付产物

### Changed

- [ ] `src/pipeline.rs` cue 输出改结构化 JSON 解析，替代文本 grep
- [ ] `src/blueprint.rs` cue 输出改结构化 JSON 解析，替代文本 grep
- [ ] `src/contract.rs` cue 输出改结构化 JSON 解析，替代文本 grep
- [ ] `../provider/` Provider 错误记录增强：保留脚本 stdout/stderr 摘要和失败步骤上下文
- [ ] 补齐真实业务 resource 绑定工作流，避免交付项目长期停留在 `resource: builtin:copy`

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
