# TODO

> 按发布版本组织：版本范围与条目见 [ROADMAP.md](ROADMAP.md)，本文件为任务级清单。
> 任务完成并在版本发布后迁移到 CHANGELOG。

## 0. release 流程（每版发布均需执行）

- [ ] 将 feature 分支 push 到远端并创建 Pull Request（`CONTRIBUTING.md`）
- [ ] 通过 code review 后合并到 main（`CONTRIBUTING.md`）
- [ ] 在 main 合并完成后运行 qtcloud-devops release publish -v cli/v0.X.Y --registry crates -y（`CONTRIBUTING.md`）
- [ ] 验证 crates.io、GitHub Release 和 Linux/Windows 二进制制品（`CONTRIBUTING.md`）

## [0.2.2] manifest 契约与 catalog 产物类型

- [ ] manifest：定义输入契约，声明 raw、map、配置表和 review decisions（`docs/`）
- [ ] manifest：增加 YAML 校验和错误提示（`src/spec.rs`）
- [ ] catalog：记录区分预审核产物、审核决策文件和最终交付产物（`src/catalog.rs`）
- [ ] view：pipeline/blueprint list/show 改文件直读为主，cue 降为可选增强（对齐 contract.rs，`src/pipeline.rs` / `src/blueprint.rs`）
- [ ] view：doctor 的 cue 检查降为 optional（`src/doctor.rs`）
- [x] coverage：查看类命令 fixture 补测（第一部分）：contract / version / transfer 已落地（contract 67%、version 96%、transfer 80%，`tests/`）
- [x] coverage：查看类命令补测（第二部分）：blueprint 85% / pipeline 82%（fake cue 注入 PATH，`src/blueprint.rs` / `src/pipeline.rs`）
- [x] coverage：LLM 命令注入 Handler 补测：clarify 80% / design 63% / implement 82% / review 66%（`src/*.rs`，复用 quanttide-agent `HttpClient`，见 `lib.rs test_support`）
- [x] coverage：非 dropbox provider wiremock 补测：google_drive 80% / onedrive 77% / s3 43%（`src/providers/`、`tests/provider_test.rs`）
- [ ] coverage：baidu/sftp 补测（需要真实服务或本地模拟，0% → ≥50%，需 CI 起 sshd 或本地模拟，`src/providers/`）
- [x] coverage：更新覆盖率基线（当前 83.7%，见 `CONTRIBUTING.md` 测试分层）
- [ ] providers：凭证环境变量名集中为常量表（`DROPBOX_ACCESS_TOKEN` / `GOOGLE_DRIVE_ACCESS_TOKEN` 等魔法字符串去重，`src/providers/mod.rs`）——低优先，可选

### 待排期（不阻塞 v0.2.2，依赖 Provider ROADMAP [0.0.3]）

- [ ] provider：增加 `PROVIDER_URL` 配置 Provider 服务地址（`src/`、`docs/`）
- [ ] provider：增加调用 Provider run API 的 CLI 入口，按 Specification/Blueprint 发起执行（`src/`）
- [ ] provider：增加 run 请求参数的 CLI 校验和错误提示（`src/`、`docs/specification.md`）
- [ ] testing：业务 e2e raw + map.dta → review_master 全链路（`tests/`，依赖 Provider merge_review / export）
- [ ] testing：保存业务 e2e 的输入、输出和验证记录（`docs/`）

## [0.3.0]

- [ ] distribution：增加 macOS 二进制构建（`../../.github/workflows/release-cli.yml`）
- [ ] distribution：增加发布后的 deploy、operate、monitor 记录（`../../.github/workflows/`、`docs/`）
- [ ] runtimes：`src/lib.rs` 注册 runtimes 模块（新增 src/runtimes/）
- [ ] runtimes：`src/process.rs` run_pipeline 改注册表查表，替代扩展名 if-else
- [ ] runtimes：`src/implement.rs` implement 支持 --lang r / --lang stata
- [ ] runtimes：`src/blueprint_core.rs` 新增 R / Stata codegen prompt 模板
- [ ] runtimes：`src/doctor.rs` 检查表由 RuntimeAdapter 注册表驱动
- [ ] structured-output：`src/main.rs` 全局 --json 结构化输出
- [ ] structured-output：`src/transfer.rs` provider 枚举化，替代字符串匹配
- [ ] structured-output：`src/process.rs` pipeline 引用结构化（Blueprint states），替代逗号分隔字符串

## [0.5.0]

- [ ] usability：新人通过 CLI 和工程规范快速接手历史项目（`docs/`）
- [ ] usability：将常见内部协调动作封装成可重复命令（`src/`）
- [ ] usability：review 自动发现跨项目不一致模式（`src/review.rs`）

## 99. 后续（未分配版本）

- [ ] `src/doctor.rs` env 注入模式推广到各模块（参考 data_dirs_with）
