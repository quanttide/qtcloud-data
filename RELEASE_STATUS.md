# qtcloud-data 发布状态

> 生成日期: 2026-07-31 | 仓库: [quanttide/qtcloud-data](https://github.com/quanttide/qtcloud-data)

## 总览

| Scope | 语言 | 源码版本 | 已发布制品 | 状态 |
|---|---|---|---|---|
| cli | Rust | v0.2.0 | crates.io v0.1.16 / GitHub tag `cli/v0.1.16` | v0.2.0 源码已合并 main，制品发布待 owner 权限和 `cli/v0.2.0` tag |
| provider | Go | v0.2.0 | GitHub tag `provider/v0.0.1` | v0.2.0 源码已合并 main，Blueprint runner 底座可本地运行 |
| studio | Flutter | 原型阶段 | GitHub tag `studio/v0.1.0-alpha.1` | 页面开发中，本次 CLI + Provider 交付不包含 Studio 发布 |

## v0.2.0 源码交付

PR: [#3](https://github.com/quanttide/qtcloud-data/pull/3)
源码提交: `f0cb741 feat: publish cli provider v0.2.0 source`
合并提交: `16a69c0 Merge pull request #3 from quanttide/codex/cli-provider-source-v020`
合并时间: 2026-07-31 20:46:26 +08:00

### CLI

**Added**
- `doctor`：检查 Git、Rust、Python、CUE、数据目录和常见传输凭证。
- `doctor --fix-dirs`：自动创建 `.quanttide/data/` 常用目录。
- `doctor --json`：输出机器可读诊断报告。
- `spec wrap` / `spec validate`：固化并校验 CLI 与 Provider 共用的 Specification YAML envelope。
- `process`：执行后写入 job 记录，并把最终产物登记到 catalog。
- `transfer send --output`：保留交付链接记录。
- `design blueprint`：生成 Step Functions 风格的 `pipeline.start_at` / `pipeline.states`，并默认写入 `resource: builtin:copy`。

**Changed**
- CLI 内部依赖改为 crates.io 版本依赖，源码交付不依赖开发者本机 `D:\packages`。

### Provider

**Added**
- 读取 CLI 生成的 legacy Blueprint YAML 与 Specification envelope YAML。
- 新增 `GET /blueprints`、`GET /blueprints/{name}`、`POST /blueprints/{name}/runs`。
- 新增 `GET /process/jobs/{id}`，返回单条 job 详情。
- Pipeline 执行器支持 `builtin:copy`、`python:<script>`、`bash:<script>`。
- process job 支持文件持久化，并记录 step 输入、输出、状态和错误。

**Security**
- 脚本资源通过参数化方式调用解释器，不直接执行任意 shell 字符串。
- 支持用 `PIPELINE_INPUT_DIR`、`PIPELINE_WORK_ROOT`、`PIPELINE_SCRIPT_DIR` 收紧本地路径边界。
- HTTP 错误响应不暴露脚本执行细节。

## 验证记录

最后本地验证时间：2026-07-31。

| Scope | 命令 | 结果 |
|---|---|---|
| cli | `cargo fmt --check` | 通过 |
| cli | `cargo test --quiet` | 通过，46 + 19 + 9 个测试 |
| cli | `cargo clippy -- -D warnings` | 通过 |
| provider | `go test ./...` | 通过 |
| provider | `go vet ./...` | 通过 |
| cli/provider | secret 扫描 | 未发现真实 API key/token/password |

## 待完成发布动作

- 创建并推送 `cli/v0.2.0` tag。
- 由 crates.io owner 发布 `qtcloud-data-cli` v0.2.0。
- 基于 CLI v0.2.0 CHANGELOG 创建 GitHub Release。
- 如 Provider 需要单独发布，创建 `provider/v0.2.0` tag，并补充对应 Release。
