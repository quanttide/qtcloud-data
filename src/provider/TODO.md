# TODO

## pipeline-runtime

- [ ] 将真实业务资源替换为可配置处理脚本，逐步减少对 `builtin:copy` 的依赖（`src/provider/internal/pipeline/`）。
- [ ] 为脚本资源补充配置示例和边界说明（`src/provider/README.md`、`src/provider/testdata/`）。

## job-observability

- [ ] 扩充失败 job 的 stdout/stderr 摘要（`src/provider/internal/store/`、`src/provider/internal/api/`）。
- [ ] 在 job 详情中保留失败步骤的输入、输出、resource 和错误上下文（`src/provider/internal/store/`、`src/provider/internal/api/`）。

## contract-alignment

- [ ] 稳定 Provider run 与 CLI catalog/job 的字段契约（`src/provider/internal/store/`、`src/cli/docs/dev/specification.md`）。
- [ ] 补充 Specification envelope 与 legacy Blueprint YAML 的兼容性样例（`src/provider/testdata/`）。

## smoke-e2e

- [ ] 增加真实交付链路的 Provider smoke/e2e 测试（`src/provider/internal/`、`src/provider/testdata/`）。
- [ ] 记录 smoke/e2e 的输入、输出和验证方式（`src/provider/README.md`、`src/provider/testdata/`）。
