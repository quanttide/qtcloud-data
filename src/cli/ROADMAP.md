# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。
> ROADMAP 面向未来计划；发布后将已完成条目迁移到 CHANGELOG。
> 覆盖率基线：v0.2.1 变更面行覆盖 >85%；整体行覆盖 83.7%（53.2% 起），0% 存量模块补测目标 ≥70%（见 [0.2.1]）。

## [0.2.2]

> manifest 契约与 catalog 产物类型；不依赖 Provider。Provider run 入口与业务 e2e 待 Provider 侧稳定后另行排期（见下方注记）。

### Added

- [ ] `docs/` 定义 manifest 输入契约（raw[] / map / review_decisions[]），`src/spec.rs` 增加 manifest YAML 校验和错误提示
- [ ] `src/catalog.rs` catalog/job 记录区分预审核产物、审核决策文件和最终交付产物（status 枚举已在 v0.2.1 落地）
- [ ] `src/pipeline.rs` / `src/blueprint.rs` list/show 改文件直读为主路径（cue 降为可选增强，对齐 contract.rs v0.2.1 先例），`src/doctor.rs` cue 检查降为 optional——装完即用，不暴露 cue 模块概念
- [x] `tests/` 查看类命令补测（第一部分）：contract 67% / version 96% / transfer 80%，fixture + wiremock 已落地
- [x] `tests/` 查看类命令补测（第二部分）：blueprint 85% / pipeline 82%（fake cue 脚本注入 PATH）；main 仍仅子进程流可测
- [x] `src/` LLM 命令注入 Handler 补测：clarify 80% / design 63% / implement 82% / review 66%（复用 quanttide-agent 的 `HttpClient` 抽象，`test_support::fake_llm`）
- [x] `src/providers/` wiremock 补测：google_drive 80% / onedrive 77% / s3 43%
- [ ] `src/providers/` baidu/sftp 补测（需要真实服务或本地模拟，0% → ≥50%，需 CI 起 sshd 或本地模拟）

> **待排期（不阻塞 v0.2.2）**：CLI 发起 Provider run 的执行入口（`PROVIDER_URL` 配置 + run 请求参数校验），以及业务 e2e（raw + map.dta → review_master，依赖 Provider merge_review / export）——待 Provider ROADMAP [0.0.3] 的 merge_review/export 落地后启动。

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

- [ ] `docs/` 补充发布后的 deploy、operate、monitor 阶段记录

## [0.5.0]

### Added

- [ ] `README.md` 新人通过 CLI 和工程规范快速接手历史项目
- [ ] `src/process.rs` 将常见内部协调动作封装成可重复命令

### Changed

- [ ] `src/review.rs` review 自动发现跨项目不一致模式
