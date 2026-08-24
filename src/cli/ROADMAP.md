# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。
> ROADMAP 面向未来计划；发布后将已完成条目迁移到 CHANGELOG。
> 覆盖率基线：v0.2.1 变更面行覆盖 >85%；整体行覆盖 83.7%（53.2% 起），0% 存量模块补测目标 ≥70%（见 [0.2.1]）。

## [0.2.2]

> manifest 契约与 catalog 产物类型；不依赖 Provider。Provider run 入口与业务 e2e 待 Provider 侧稳定后另行排期（见下方注记）。
> 实现项已完成；发布动作按 TODO 的 release 流程执行。

### Added

- [x] `docs/` 定义 manifest 输入契约（raw[] / map / review_decisions[]），`src/spec/mod.rs` 增加 manifest YAML 校验和错误提示
- [x] `src/implementation/catalog.rs` catalog/job 记录区分预审核产物、审核决策文件和最终交付产物（status 枚举已在 v0.2.1 落地）
- [x] `src/implementation/pipeline.rs` / `src/spec/blueprint.rs` list/show 改文件直读为主路径（cue 降为可选增强，对齐 contract.rs v0.2.1 先例），`src/doctor.rs` cue 检查降为 optional——装完即用，不暴露 cue 模块概念
- [x] `tests/` 查看类命令补测（第一部分）：contract 67% / version 96% / transfer 80%，fixture + wiremock 已落地
- [x] `tests/` 查看类命令补测（第二部分）：blueprint 85% / pipeline 82%（fake cue 脚本注入 PATH）；main 仍仅子进程流可测
- [x] `src/` LLM 命令注入 Handler 补测：clarify 80% / design 63% / implement 82% / review 66%（复用 quanttide-agent 的 `HttpClient` 抽象，`test_support::fake_llm`）
- [x] `src/providers/` wiremock 补测：google_drive 80% / onedrive 77% / s3 43%
- [x] `src/storage/` baidu/sftp 补测移至 v0.3.0（需真实服务或本地模拟：CI 起 sshd 或本地模拟）——不阻塞 v0.2.2 发布

> **待排期（不阻塞 v0.2.2）**：CLI 发起 Provider run 的执行入口（`PROVIDER_URL` 配置 + run 请求参数校验），以及业务 e2e（raw + map.dta → review_master，依赖 Provider merge_review / export）——待 Provider ROADMAP [0.0.3] 的 merge_review/export 落地后启动。

## [0.3.1]

> v0.3.0 已完成项已迁移至 [CHANGELOG.md](CHANGELOG.md)。

### Added

- [ ] `Cargo.toml` / `.github/workflows/release-cli.yml` 构建并上传 macOS 二进制包
- [ ] 其余命令成功结果逐步迁移为结构化输出，供 Studio/CI 消费

### Changed

- [ ] `docs/` 补充发布后的 deploy、operate、monitor 阶段记录

## [0.5.0]

### Added

- [ ] `README.md` 新人通过 CLI 和工程规范快速接手历史项目
- [ ] `src/process.rs` 将常见内部协调动作封装成可重复命令

### Changed

- [ ] `src/review.rs` review 自动发现跨项目不一致模式
