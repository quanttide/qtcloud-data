# TODO

## 1. release

- [ ] 将 feature 分支 push 到远端并创建 Pull Request（`docs/dev/release.md`）
- [ ] 通过 code review 后合并到 `main`（`docs/dev/release.md`）
- [ ] 在 main 合并完成后运行 qtcloud-devops release publish -v cli/v0.2.0 --registry crates -y（`docs/dev/release.md`）
- [ ] 验证 crates.io、GitHub Release 和 Linux/Windows 二进制制品（`docs/dev/release.md`）

## 2. manifest

- [ ] 定义 manifest 输入契约，声明 raw、map、配置表和 review decisions（`docs/dev/`）
- [ ] 增加 manifest 的 YAML 校验和错误提示（`src/spec.rs`）

## 3. provider-integration

- [ ] 增加调用 Provider run API 的 CLI 入口，按 Specification/Blueprint 发起执行（`src/`）
- [ ] 增加 Provider run 请求参数的 CLI 校验和错误提示（`src/`、`docs/dev/specification.md`）

## 4. smoke-e2e

- [ ] 增加 raw + `map.dta` 到 `review_master` 的正式 smoke/e2e 测试（`tests/`）
- [ ] 保存 smoke/e2e 的输入、输出和验证记录（`docs/`）

## 5. distribution

- [ ] 增加 macOS 二进制构建（`../../.github/workflows/release-cli.yml`）
- [ ] 增加发布后的 deploy、operate、monitor 记录（`../../.github/workflows/`、`docs/`）

## 6. usability

- [ ] 新人通过 CLI 和工程规范快速接手历史项目（`docs/`）
- [ ] 将常见内部协调动作封装成可重复命令（`src/`）
- [ ] `review` 自动发现跨项目不一致模式（`src/review.rs`）

## 7. error-model

- [ ] `src/main.rs` 命令入口改为 `Result<(), CliError>`，顶层统一错误格式化
- [ ] `src/process.rs` 抽取 StepExecutor 状态机（Receive → Pipeline → Send），收敛 5 份重复失败处理
- [ ] `src/transfer.rs` send/receive 抽为进程内服务函数，process 库内组合替代自我 re-exec

## 8. store

- [ ] `src/lib.rs` 新增 store 模块：Registry<T> 合并 catalog/process/transfer 三份 JSON 读写
- [ ] `src/catalog.rs` registry 读写改用 store 模块
- [ ] `src/process.rs` jobs 记录读写改用 store 模块
- [ ] `src/transfer.rs` delivery-links 读写改用 store 模块
- [ ] `src/lib.rs` store 模块统一时间工具，替换三份 chrono_now/days_to_date 拷贝
- [ ] `src/lib.rs` store 模块写盘原子化（临时文件 + rename）

## 9. cue

- [ ] `src/pipeline.rs` list/show 改 cue --out json 结构化解析
- [ ] `src/blueprint.rs` list/show 改 cue --out json 结构化解析
- [ ] `src/contract.rs` 以文件直读为主路径，cue 为可选增强

## 10. runtimes

- [ ] `src/lib.rs` 注册 runtimes 模块（新增 src/runtimes/）
- [ ] `src/process.rs` run_pipeline 改注册表查表，替代扩展名 if-else
- [ ] `src/implement.rs` implement 支持 --lang r / --lang stata
- [ ] `src/blueprint_core.rs` 新增 R / Stata codegen prompt 模板
- [ ] `src/doctor.rs` 检查表由 RuntimeAdapter 注册表驱动

## 11. structured-output

- [ ] `src/main.rs` 全局 --json 结构化输出
- [ ] `src/transfer.rs` provider 枚举化，替代字符串匹配
- [ ] `src/catalog.rs` status 枚举化，替代魔法字符串
- [ ] `src/process.rs` pipeline 引用结构化（Blueprint states），替代逗号分隔字符串

## 12. testing

- [ ] `src/process.rs` StepExecutor 单元测试（tempfile + 注入式路径）
- [ ] `src/clarify.rs` LLM 调用注入 LlmClient trait，支持 fake 测试
- [ ] `src/design.rs` LLM 调用注入 LlmClient trait，支持 fake 测试
- [ ] `src/implement.rs` LLM 调用注入 LlmClient trait，支持 fake 测试
- [ ] `src/doctor.rs` env 注入模式推广到各模块（参考 data_dirs_with）
