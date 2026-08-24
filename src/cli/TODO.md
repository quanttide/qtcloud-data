# TODO

> 按发布版本组织：版本范围与条目见 [ROADMAP.md](ROADMAP.md)，本文件为任务级清单。
> 任务完成并在版本发布后迁移到 CHANGELOG。

## 0. release 流程（每版发布均需执行）

- [ ] 将 feature 分支 push 到远端并创建 Pull Request（`CONTRIBUTING.md`）
- [ ] 通过 code review 后合并到 main（`CONTRIBUTING.md`）
- [ ] 在 main 合并完成后运行 qtcloud-devops release publish -v cli/v0.X.Y --registry crates -y（`CONTRIBUTING.md`）
- [ ] 验证 crates.io、GitHub Release 和 Linux/Windows 二进制制品（`CONTRIBUTING.md`）

## [0.2.2] manifest 契约与 catalog 产物类型

- [x] manifest：定义输入契约，声明 raw、map、配置表和 review decisions（`docs/`）
- [x] manifest：增加 YAML 校验和错误提示（`src/spec/mod.rs`）
- [x] catalog：记录区分预审核产物、审核决策文件和最终交付产物（`src/implementation/catalog.rs`）
- [x] catalog：状态流转闭环——VolumeStatus 5 态目前只有 add(received) 与 process(delivered) 可达，补 processing/processed 的设置路径（如 `catalog set-status` 或 review/process 自动流转）
- [x] view：pipeline/blueprint list/show 改文件直读为主，cue 降为可选增强（对齐 contract.rs，`src/implementation/pipeline.rs` / `src/spec/blueprint.rs`）
- [x] view：doctor 的 cue 检查降为 optional（`src/doctor.rs`）
- [x] coverage：查看类命令 fixture 补测（第一部分）：contract / version / transfer 已落地（contract 67%、version 96%、transfer 80%，`tests/`）
- [x] coverage：查看类命令补测（第二部分）：blueprint 85% / pipeline 82%（fake cue 注入 PATH，`src/blueprint.rs` / `src/pipeline.rs`）
- [x] coverage：LLM 命令注入 Handler 补测：clarify 80% / design 63% / implement 82% / review 66%（`src/*.rs`，复用 quanttide-agent `HttpClient`，见 `lib.rs test_support`）
- [x] coverage：非 dropbox provider wiremock 补测：google_drive 80% / onedrive 77% / s3 43%（`src/providers/`、`tests/provider_test.rs`）
- [x] coverage：baidu/sftp 补测迁移至 [0.3.0]（需要真实服务或本地模拟，不阻塞 v0.2.2）
- [x] coverage：更新覆盖率基线（当前 83.7%，见 `CONTRIBUTING.md` 测试分层）
- [x] storage：凭证环境变量名集中为常量表迁移至 99. 后续（`DROPBOX_ACCESS_TOKEN` / `GOOGLE_DRIVE_ACCESS_TOKEN` 等魔法字符串去重，低优先，可选）

## [0.3.1]

- [ ] distribution：增加 macOS 二进制构建和 Release 上传（`../../.github/workflows/release-cli.yml`）
- [ ] distribution：增加发布后的 deploy、operate、monitor 记录（`../../.github/workflows/`、`docs/`）
- [ ] structured-output：其余命令成功结果逐步迁移为结构化输出（`src/`）

## [0.5.0]

- [ ] usability：新人通过 CLI 和工程规范快速接手历史项目（`docs/`）
- [ ] usability：将常见内部协调动作封装成可重复命令（`src/`）
- [ ] usability：review 自动发现跨项目不一致模式（`src/review.rs`）

## 99. 后续（未分配版本）

- [ ] `src/doctor.rs` env 注入模式推广到各模块（参考 data_dirs_with）
- [ ] `src/storage/mod.rs` 凭证环境变量名集中为常量表（`DROPBOX_ACCESS_TOKEN` / `GOOGLE_DRIVE_ACCESS_TOKEN` 等魔法字符串去重）
- [ ] provider：增加 `PROVIDER_URL` 配置 Provider 服务地址（`src/`、`docs/`，依赖 Provider ROADMAP [0.0.3]）
- [ ] provider：增加调用 Provider run API 的 CLI 入口，按 Specification/Blueprint 发起执行（`src/`，依赖 Provider merge_review / export）
- [ ] provider：增加 run 请求参数的 CLI 校验和错误提示（`src/`、`docs/specification.md`）
- [ ] testing：业务 e2e raw + map.dta → review_master 全链路（`tests/`，依赖 Provider merge_review / export）
- [ ] testing：保存业务 e2e 的输入、输出和验证记录（`docs/`）
