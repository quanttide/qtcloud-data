# CHANGELOG

## [Unreleased]

### Added
- 读取 CLI 生成的 legacy Blueprint YAML 与 Specification envelope YAML。
- 新增 `GET /blueprints` 和 `GET /blueprints/{name}`，供 Studio 浏览蓝图列表和详情。
- 支持 Blueprint `pipeline.start_at` / `pipeline.states` 状态机字段，兼容旧 `pipeline.steps`。
- 新增 `POST /blueprints/{name}/runs`，从 Blueprint 读取 Pipeline 并执行一次本地 run。
- 新增 `GET /process/jobs/{id}`，返回单条 job 的输入、输出、错误和 step 详情。
- Pipeline 执行器支持 `builtin:copy`，以及受 `PIPELINE_SCRIPT_DIR` 限制的 `python:<script>` / `bash:<script>` 脚本资源。
- process job 记录扩展为包含 blueprint、pipeline、input、output、status、started_at、finished_at 和 step 结果。
- process job 支持文件持久化；Provider 启动时通过 `JOB_STORE_PATH` / `CATALOG_DIR` / `DATA_ROOT` 定位 catalog job 文件并加载历史记录。
- Pipeline 某一步失败时，job 会保留已完成步骤和失败步骤的输入、输出、资源与状态，便于 Studio 展开排查。
- 设置 `PIPELINE_WORK_ROOT` 且目录尚不存在时，Provider 会自动创建工作根目录，再按 job id 创建本次 run 的工作目录。

### Security
- Blueprint 执行接口不会直接执行任意 shell 字符串；脚本资源只按参数化方式调用解释器，并限制脚本真实路径位于 `PIPELINE_SCRIPT_DIR` 下。
- 支持用 `PIPELINE_INPUT_DIR` / `PIPELINE_WORK_ROOT` 收紧 run API 的输入和工作目录边界，并解析真实路径以拒绝 symlink/junction 逃逸。
- Pipeline 执行失败时 HTTP 响应只返回泛化错误，详细错误记录在服务端日志。

## [0.0.1] - 2026-07-10

### Added
- Provider 接口定义（对应 Rust StorageProvider trait）。
- Dropbox 传输实现。
- S3 传输实现（stub）。
- HTTP API 骨架（4 个端点）。
- Pipeline 执行引擎（stub）。
- 内存存储（process job 记录）。
- Go 项目结构和模块化架构。
