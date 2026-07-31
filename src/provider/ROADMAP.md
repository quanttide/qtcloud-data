# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。
> 当前源码版本：`qtcloud-provider` v0.2.0。

Provider 目标：承接 CLI 生成的 Specification / Blueprint YAML，把本地 DataOps 流程从命令行推进到可被 Studio 调用的后端执行服务。

## [0.2.1]

### Added
- [ ] 增强 Pipeline run 错误记录，保留 stdout/stderr 摘要和失败步骤上下文。
- [ ] 支持 manifest 输入契约，允许一次 run 声明 raw、map、配置表和 review decisions 等多输入。
- [ ] 增加真实业务 resource smoke/e2e 案例，覆盖 `python:<script>` 执行链路。
- [ ] 为 Provider API 增加统一响应 envelope，便于 Studio 稳定处理错误和数据。

### Security
- [ ] 增加 transfer API 的输入路径校验，避免未配置边界时误暴露本机敏感路径。
- [ ] 增加脚本执行超时和输出大小限制。

## [0.2.0] — 源码已合并 main

### Added
- [x] 读取 CLI 生成的 legacy Blueprint YAML 与 Specification envelope YAML。
- [x] 新增 `GET /blueprints` 和 `GET /blueprints/{name}`。
- [x] 支持 Blueprint `pipeline.start_at` / `pipeline.states` 状态机字段，兼容旧 `pipeline.steps`。
- [x] 新增 `POST /blueprints/{name}/runs`，从 Blueprint 读取 Pipeline 并执行一次本地 run。
- [x] 新增 `GET /process/jobs/{id}`，返回单条 job 的输入、输出、错误和 step 详情。
- [x] Pipeline 执行器支持 `builtin:copy`、`python:<script>`、`bash:<script>`。
- [x] process job 支持文件持久化，并记录 step 结果。

### Security
- [x] 脚本资源只按参数化方式调用解释器，不直接执行任意 shell 字符串。
- [x] 用 `PIPELINE_INPUT_DIR`、`PIPELINE_WORK_ROOT`、`PIPELINE_SCRIPT_DIR` 收紧本地路径边界。
- [x] Pipeline 执行失败时 HTTP 响应只返回泛化错误。

## [0.0.1] — 已发布

### Added
- [x] Provider 接口定义。
- [x] Dropbox 传输实现。
- [x] S3 传输 stub。
- [x] HTTP API 骨架。
- [x] Pipeline 执行引擎 stub。
- [x] 内存 process job store。
