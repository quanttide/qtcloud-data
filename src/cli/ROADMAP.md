# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。
> ROADMAP 面向未来计划；发布后将已完成条目迁移到 CHANGELOG。
> 覆盖率基线：v0.2.1 变更面行覆盖 >85%；整体行覆盖 53.2%，0% 存量模块补测目标 ≥70%（见 [0.2.2]）。

## [0.2.2]

> manifest + Provider 打通；新功能版本，依赖 v0.2.1 的 store/catalog 枚举化，与 Provider ROADMAP [0.0.3] 协同。

### Added

- [ ] `docs/dev/` 定义 manifest 输入契约（raw[] / map / review_decisions[]），`src/spec.rs` 增加 manifest YAML 校验和错误提示
- [ ] `src/process.rs` 增加通过 CLI 发起 Provider run 的执行入口（`PROVIDER_URL` 配置 + run 请求参数校验）
- [ ] `src/catalog.rs` catalog/job 记录区分预审核产物、审核决策文件和最终交付产物（status 枚举已在 v0.2.1 落地）
- [ ] `tests/` 增加业务 e2e：raw + map.dta → review_master 全链路（依赖 Provider merge_review / export）
- [ ] `tests/` 查看类命令补测：blueprint/contract/pipeline/review/version/main fixture 测试（0% → ≥70%）
- [ ] `src/clarify.rs` LLM 命令注入 LlmClient fake 补测（clarify/design/implement 0% → ≥60%）
- [ ] `src/providers/` baidu/google/onedrive/sftp wiremock 补测（0% → ≥50%）

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
