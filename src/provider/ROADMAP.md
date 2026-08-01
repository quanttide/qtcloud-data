# ROADMAP

> 格式：Keep a Changelog + checkbox 任务清单。
> ROADMAP 面向未来计划；发布后将已完成条目迁移到 CHANGELOG。

## [0.0.2]

### Added
- [x] 读取 CLI 生成的 Specification/Blueprint YAML。
- [x] 提供 Blueprint 列表、详情、执行和 process job 查询 API。
- [x] 提供最小 Pipeline 执行器，支持内置复制和受限脚本资源。
- [x] process job 支持文件持久化。

### Security
- [x] 收紧脚本、输入和工作目录边界。
- [x] Pipeline 失败时避免向 HTTP 响应暴露详细内部错误。

## [0.0.3]

### Changed
- [ ] 将真实业务资源替换为可配置处理脚本。
- [ ] 扩充失败 job 的 stdout/stderr 摘要和失败步骤上下文。
- [ ] 稳定 Provider run 与 CLI catalog/job 的字段契约。

### Added
- [ ] 增加真实交付链路的 Provider smoke/e2e 测试。
