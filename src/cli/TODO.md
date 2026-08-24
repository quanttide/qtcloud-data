# TODO

> 按发布版本组织；版本目标见 [ROADMAP.md](ROADMAP.md)。
> `cli/v0.2.2` 已于 2026-08-02 发布，已完成内容已归档到 [CHANGELOG.md](CHANGELOG.md)。
> `0.3.0` 当前仍在开发中；本文件只记录已完成进度和剩余执行任务，不代表已发布。

## 0. release 流程（每版发布均需执行）

- [ ] 将 feature 分支 push 到远端并创建 Pull Request（`CONTRIBUTING.md`）
- [ ] 通过 code review 后合并到 main（`CONTRIBUTING.md`）
- [ ] 在 main 合并完成后运行 `qtcloud-devops release publish -v cli/v0.X.Y --registry crates -y`（`CONTRIBUTING.md`）
- [ ] 验证 crates.io、GitHub Release 和 Linux/Windows/macOS 二进制制品（`CONTRIBUTING.md`）

## 0.3.0 manifest 与 catalog

- [x] 定义 manifest 输入契约：raw、map、配置表和 `review_decisions[]`（`docs/specification.md`）
- [x] 在 `src/spec/mod.rs` 增加 Manifest 数据模型、YAML 解析和字段级错误提示
- [x] 增加 manifest 合法/非法样例及命令级测试（`tests/spec_test.rs`）
- [x] 严格复核 Manifest 路由：Manifest 形状但 `kind` 缺失/错误时仍报告 Manifest kind 错误（`src/spec/mod.rs`、`tests/spec_test.rs`）
- [x] 严格复核审核决策：`review_decisions[].target` 必须引用已声明物料（`src/spec/mod.rs`、`docs/specification.md`）
- [x] 在 `src/implementation/catalog.rs` 区分预审核产物、审核决策文件和最终交付产物
- [x] 补充 catalog 状态设置 API 和 `catalog set-status` 命令，打通 `received → processing → processed → delivered`
- [x] 让 `src/stage/process.rs` 在接收、处理、交付阶段更新对应 Volume 状态
- [x] 更新 catalog 数据格式文档和兼容性约定（`docs/catalog.md`）
- [x] 严格验收阶段三：重跑 catalog 测试、process e2e、旧 registry 兼容和失败补偿验证（`tests/catalog_test.rs`、`tests/process_test.rs`）

## 0.3.0 查看命令与依赖

- [x] `src/spec/blueprint.rs` 的 `list/show` 改为文件直读主路径，cue 作为可选增强
- [x] `src/implementation/pipeline.rs` 的 `list/show` 改为文件直读主路径，cue 作为可选增强
- [x] `src/doctor.rs` 将 cue 检查改为 optional，并补充无 cue 环境测试
- [x] 更新查看命令文档，明确文件直读和 cue 增强两种路径（`docs/blueprint.md`、`docs/pipeline.md`、`docs/doctor.md`）
- [x] 补充 Blueprint/Pipeline 无 cue 环境测试与跨平台 fake cue 回退测试（`tests/blueprint_test.rs`、`tests/pipeline_test.rs`、`tests/common/mod.rs`）

## 0.3.0 runtime 与执行

- [x] 在 `src/runtime/` 增加 R runtime 和 Stata runtime
- [x] 为 R / Stata 增加 codegen prompt、代码提取和函数命名规则（`src/runtime/`、`src/stage/implement.rs`）
- [x] 让 `src/stage/process.rs` 支持 R / Stata 脚本执行，并为未知扩展名提供结构化错误
- [x] 让 `src/doctor.rs` 的工具检查由 `src/runtime/mod.rs` 注册表驱动
- [x] 保留 Python/Bash 现有行为，并补充 runtime 注册表回归测试（`src/runtime/`、`tests/`）

## 0.3.0 structured output

- [x] 在 `src/main.rs` 增加全局 `--json` 错误通道；成功结果模型继续迁移
- [x] 让 `CliError` 携带稳定的结构化错误码（`src/error.rs`）
- [x] 将 `run_command` 分发移入库层，使 `Commands` 和参数注入路径可单测（`src/cli.rs`、`src/lib.rs`）
- [x] 在 `src/stage/transfer.rs` 使用结构化 provider 枚举，替代字符串匹配
- [x] 在 `src/stage/process.rs` 使用结构化 Blueprint states 引用，替代逗号分隔字符串
- [x] 将无副作用查看命令和 catalog/pipeline 命令迁移到统一成功结果模型（`src/cli.rs`、`src/spec/`、`src/implementation/`）

## 0.3.0 传输测试与发布

- [x] 为百度网盘补充 wiremock 或本地模拟测试（`src/storage/baidu_drive.rs`、`tests/storage_test.rs`）
- [x] 为 SFTP 补充本地模拟或 CI sshd 测试（`src/storage/sftp.rs`、`tests/storage_test.rs`）
- [ ] 核对 Cargo dist 目标与父仓库 release workflow，确保 Linux、Windows、macOS 制品一致（`Cargo.toml`、`../../.github/workflows/release-cli.yml`）
- [ ] 补充发布后的 deploy、operate、monitor 记录（`docs/`）

## Provider 依赖项（延期，不阻塞 0.3.0）

- [ ] 增加 `PROVIDER_URL` 配置（`src/`、`docs/specification.md`）
- [ ] 增加按 Specification/Blueprint 调用 Provider run API 的 CLI 入口（`src/`）
- [ ] 增加 run 请求参数校验和错误提示（`src/`、`docs/specification.md`）
- [ ] 完成 raw + map.dta → review_master 业务 e2e（`tests/`）
- [ ] 保存业务 e2e 输入、输出和验证记录（`docs/`）

## 0.5.0

- [ ] 新人通过 CLI 和工程规范快速接手历史项目（`README.md`、`docs/`）
- [ ] 将常见内部协调动作封装成可重复命令（`src/stage/process.rs`、`src/`）
- [ ] review 自动发现跨项目不一致模式（`src/review.rs`）

## 未分配版本

- [ ] 将凭证环境变量名集中为常量表（`src/storage/mod.rs`、`src/doctor.rs`）
- [ ] 将 doctor 的 env 注入模式推广到其他模块（参考 `data_dirs_with`，`src/doctor.rs`）
