# TODO — Provider v0.2.1 执行闭环

> 对应 ROADMAP：`[0.2.1]`
> v0.2.0 已归档到 `CHANGELOG.md`，本文件只保留下一版本可执行拆解。

## pipeline

- [ ] 捕获脚本 stdout/stderr 并写入 step 摘要（`internal/pipeline/pipeline.go`）
- [ ] 增加脚本执行超时配置（`internal/pipeline/pipeline.go`）
- [ ] 限制 stdout/stderr 记录大小，避免 job 文件过大（`internal/store/store.go`）
- [ ] 补充失败脚本和超时脚本测试（`internal/pipeline/pipeline_test.go`）

## manifest

- [ ] 定义 manifest 结构体和 YAML 解析逻辑（`internal/specstore/specstore.go`）
- [ ] 支持 run API 接收 manifest 路径（`internal/api/handler.go`）
- [ ] 将 manifest 多输入映射给 Pipeline 执行器（`internal/pipeline/pipeline.go`）
- [ ] 补充 manifest 解析和 run API 集成测试（`internal/specstore/specstore_test.go`、`internal/api/blueprint_handler_test.go`）

## api

- [ ] 统一成功和失败响应 envelope（`internal/api/handler.go`）
- [ ] 为 run/list/get 接口增加 request id 日志上下文（`internal/api/handler.go`）
- [ ] 收紧 transfer API 的本地路径校验（`internal/api/handler.go`）
- [ ] 更新 Provider README 的 API 响应示例（`README.md`）

## release

- [ ] 发布前确认 `version.go`、`CHANGELOG.md`、`ROADMAP.md`、Git tag 一致
- [ ] 创建并推送 `provider/v0.2.0` tag
- [ ] 基于 `CHANGELOG.md` 创建 GitHub Release
