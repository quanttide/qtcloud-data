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

### Added
- [ ] 定义 review 决策文件格式（与 CLI manifest 契约对齐，参考 `../cli/docs/dev/`）。
- [ ] 新增 `merge_review` builtin resource：将审核结果合并回匹配明细。
- [ ] 新增 `export` builtin resource：生成最终客户交付文件。
- [ ] 增加真实交付链路的 Provider smoke/e2e 测试。

### Changed
- [ ] 将真实业务资源替换为可配置处理脚本，补齐 resource 绑定工作流（避免交付项目长期停留在 `resource: builtin:copy`）。
- [ ] 扩充失败 job 的 stdout/stderr 摘要和失败步骤上下文（当前 `executeScript` 直接把 stdout/stderr 接到进程输出，需改为 buffer 并写入 job 记录）。
- [ ] 稳定 Provider run 与 CLI catalog/job 的字段契约（当前 provider JobRecord 含 input/output/steps/error，CLI jobs.json 含 raw_path/link_path/log_path，两侧字段不一致）。
