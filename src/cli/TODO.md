# TODO

## release

- [ ] 将 feature 分支 push 到远端并创建 Pull Request（`src/cli/docs/dev/release.md`）。
- [ ] 通过 code review 后合并到 `main`（`src/cli/docs/dev/release.md`）。
- [ ] 在 `main` 合并完成后运行 `qtcloud-devops release publish -v cli/v0.2.0 --registry crates -y`（`src/cli/docs/dev/release.md`）。
- [ ] 验证 crates.io、GitHub Release 和 Linux/Windows 二进制制品（`src/cli/docs/dev/release.md`）。

## manifest

- [ ] 定义 manifest 输入契约，声明 raw、map、配置表和 review decisions（`src/cli/docs/dev/`）。
- [ ] 增加 manifest 的 YAML 校验和错误提示（`src/cli/src/spec.rs`）。

## provider-integration

- [ ] 增加调用 Provider run API 的 CLI 入口，按 Specification/Blueprint 发起执行（`src/cli/src/`）。
- [ ] 增加 Provider run 请求参数的 CLI 校验和错误提示（`src/cli/src/`、`src/cli/docs/dev/specification.md`）。

## smoke-e2e

- [ ] 增加 raw + `map.dta` 到 `review_master` 的正式 smoke/e2e 测试（`src/cli/tests/`）。
- [ ] 保存 smoke/e2e 的输入、输出和验证记录（`src/cli/docs/`）。

## distribution

- [ ] 增加 macOS 二进制构建（`.github/workflows/release-cli.yml`）。
- [ ] 增加发布后的 deploy、operate、monitor 记录（`.github/workflows/`、`docs/`）。

## usability

- [ ] 新人通过 CLI 和工程规范快速接手历史项目（`src/cli/docs/`）。
- [ ] 将常见内部协调动作封装成可重复命令（`src/cli/src/`）。
- [ ] `review` 自动发现跨项目不一致模式（`src/cli/src/review.rs`）。
