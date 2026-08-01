# TODO

> 按发布版本组织：版本范围与条目见 [ROADMAP.md](ROADMAP.md)，本文件为任务级清单。
> 任务完成并在版本发布后迁移到 CHANGELOG。

## release 流程（v0.2.x 每版发布均需执行）

- [ ] 将 feature 分支 push 到远端并创建 Pull Request（`docs/dev/release.md`）
- [ ] 通过 code review 后合并到 `main`（`docs/dev/release.md`）
- [ ] 在 main 合并完成后运行 `qtcloud-devops release publish -v cli/v0.X.Y --registry crates -y`（`docs/dev/release.md`，v0.X.Y 按发布版本替换）
- [ ] 验证 crates.io、GitHub Release 和 Linux/Windows 二进制制品（`docs/dev/release.md`）

## [v0.2.1] 行为不变的内部治理

> 开工顺序：先补基线 e2e 的 fixture（对现有 `process` 命令建立回归保护），再开始 store 模块重构。

### store

- [ ] `src/lib.rs` 新增 store 模块：Registry<T> 合并 catalog/process/transfer 三份 JSON 读写
- [ ] `src/catalog.rs` registry 读写改用 store 模块
- [ ] `src/process.rs` jobs 记录读写改用 store 模块
- [ ] `src/transfer.rs` delivery-links 读写改用 store 模块
- [ ] `src/lib.rs` store 模块统一时间工具，替换三份 chrono_now/days_to_date 拷贝
- [ ] `src/lib.rs` store 模块写盘原子化（临时文件 + rename）

### cue

- [ ] `src/pipeline.rs` list/show 改 cue --out json 结构化解析
- [ ] `src/blueprint.rs` list/show 改 cue --out json 结构化解析
- [ ] `src/contract.rs` 以文件直读为主路径，cue 为可选增强
- [ ] `src/process.rs` resolve_blueprint_pipeline 改 cue --out json 结构化解析，替代文本 trim

### catalog

- [ ] `src/catalog.rs` status 枚举化，替代魔法字符串

### testing

- [x] 基线 smoke/e2e：现有 `process` 命令全链路回归，fixture 落 `tests/fixtures/`
- [x] 保存基线 smoke/e2e 的输入、输出和验证记录（`docs/`）

## [v0.2.2] 错误模型与执行器

### error-model

- [ ] `src/main.rs` 命令入口改为 `Result<(), CliError>`，顶层统一错误格式化
- [ ] `src/process.rs` 抽取 StepExecutor 状态机（Receive → Pipeline → Send），收敛 5 份重复失败处理
- [ ] `src/transfer.rs` send/receive 抽为进程内服务函数，process 库内组合替代自我 re-exec

### testing

- [ ] `src/process.rs` StepExecutor 单元测试（tempfile + 注入式路径）

## [v0.2.3] manifest + Provider 打通

### manifest

- [ ] 定义 manifest 输入契约，声明 raw、map、配置表和 review decisions（`docs/dev/`）
- [ ] 增加 manifest 的 YAML 校验和错误提示（`src/spec.rs`）

### provider-integration

- [ ] 增加 `PROVIDER_URL` 配置 Provider 服务地址（`src/`、`docs/dev/`）
- [ ] 增加调用 Provider run API 的 CLI 入口，按 Specification/Blueprint 发起执行（`src/`）
- [ ] 增加 Provider run 请求参数的 CLI 校验和错误提示（`src/`、`docs/dev/specification.md`）

### catalog

- [ ] catalog/job 记录区分预审核产物、审核决策文件和最终交付产物（status 枚举已在 v0.2.1 落地）

### testing

- [ ] 业务 e2e：raw + `map.dta` → `review_master` 全链路（依赖 Provider merge_review / export）
- [ ] 保存业务 e2e 的输入、输出和验证记录（`docs/`）

## [v0.3.0]

### distribution

- [ ] 增加 macOS 二进制构建（`../../.github/workflows/release-cli.yml`）
- [ ] 增加发布后的 deploy、operate、monitor 记录（`../../.github/workflows/`、`docs/`）

### runtimes

- [ ] `src/lib.rs` 注册 runtimes 模块（新增 src/runtimes/）
- [ ] `src/process.rs` run_pipeline 改注册表查表，替代扩展名 if-else
- [ ] `src/implement.rs` implement 支持 --lang r / --lang stata
- [ ] `src/blueprint_core.rs` 新增 R / Stata codegen prompt 模板
- [ ] `src/doctor.rs` 检查表由 RuntimeAdapter 注册表驱动

### structured-output

- [ ] `src/main.rs` 全局 --json 结构化输出
- [ ] `src/transfer.rs` provider 枚举化，替代字符串匹配
- [ ] `src/process.rs` pipeline 引用结构化（Blueprint states），替代逗号分隔字符串

## [v0.5.0]

### usability

- [ ] 新人通过 CLI 和工程规范快速接手历史项目（`docs/`）
- [ ] 将常见内部协调动作封装成可重复命令（`src/`）
- [ ] `review` 自动发现跨项目不一致模式（`src/review.rs`）

## 后续（未分配版本）

### testing

- [ ] `src/clarify.rs` LLM 调用注入 LlmClient trait，支持 fake 测试
- [ ] `src/design.rs` LLM 调用注入 LlmClient trait，支持 fake 测试
- [ ] `src/implement.rs` LLM 调用注入 LlmClient trait，支持 fake 测试
- [ ] `src/doctor.rs` env 注入模式推广到各模块（参考 data_dirs_with）
